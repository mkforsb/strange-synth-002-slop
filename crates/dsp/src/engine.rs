//! The polyphonic engine and global chain:
//! voices → hum → sympathy → corrode → ladder → EQ → Dattorro plate → volume.

use crate::chaos::Lorenz;
use crate::filters::{Biquad, Ladder, LadderMode};
use crate::modal::{ModalBank, NMODES};
use crate::reverb::Dattorro;
use crate::util::*;
use crate::voice::{Voice, VoiceCtl, CTRL};
use oxide_core::telemetry as tm;
use oxide_core::{NUM_PARAMS, P, PARAMS};

pub const MAX_VOICES: usize = 8;
pub const BLOCK: usize = 128;

/// The hidden metal frame that Sympathy excites: an inharmonic set of
/// tube and bar frequencies.
const FRAME_FREQS: [f32; NMODES] = [
    97.0, 131.7, 187.1, 233.8, 291.9, 346.2, 471.0, 622.8, 845.4, 1123.9, 1514.2, 2047.7, 2731.3, 3612.0,
    4890.5, 6203.1,
];

pub struct Engine {
    sr: f32,
    ctl: VoiceCtl,
    dirty: bool,
    voices: Vec<Voice>,
    age: u64,
    last_pitch: f32,
    lorenz: Lorenz,
    noise: Noise,
    hum_ph: f32,
    activity: f32,
    symp: ModalBank,
    crush_cnt: f32,
    crush_hold: [f32; 2],
    ladder: [Ladder; 2],
    eq: [[Biquad; 5]; 2],
    reverb: Dattorro,
    sm_cut: f32,
    sm_vol: f32,
    sm_mix: f32,
    sm_drive: f32,
    ctrl_count: usize,
    trail_count: u32,
    ticks: u32,
    buf_l: [f32; BLOCK],
    buf_r: [f32; BLOCK],
    pub out: [f32; 2 * BLOCK],
    pub telemetry: [f32; tm::LEN],
}

impl Engine {
    pub fn new(sr: f32) -> Self {
        let sr = sr.clamp(8000.0, 192_000.0);
        let mut symp = ModalBank::default();
        symp.tune(&FRAME_FREQS, &[5.0; NMODES], sr);
        let mut e = Engine {
            sr,
            ctl: VoiceCtl::new(sr),
            dirty: true,
            voices: (0..MAX_VOICES).map(|i| Voice::new(sr, 1234 + i as u32 * 7919)).collect(),
            age: 0,
            last_pitch: 60.0,
            lorenz: Lorenz::new(1.0, 1.0, 20.0),
            noise: Noise::new(99),
            hum_ph: 0.0,
            activity: 0.0,
            symp,
            crush_cnt: 0.0,
            crush_hold: [0.0; 2],
            ladder: [Ladder::default(); 2],
            eq: [[Biquad::default(); 5]; 2],
            reverb: Dattorro::new(sr),
            sm_cut: 12000.0,
            sm_vol: 0.5,
            sm_mix: 0.25,
            sm_drive: 1.0,
            ctrl_count: 0,
            trail_count: 0,
            ticks: 0,
            buf_l: [0.0; BLOCK],
            buf_r: [0.0; BLOCK],
            out: [0.0; 2 * BLOCK],
            telemetry: [0.0; tm::LEN],
        };
        e.update_globals();
        e
    }

    pub fn sample_rate(&self) -> f32 {
        self.sr
    }

    pub fn set_param(&mut self, i: usize, v: f32) {
        if i < NUM_PARAMS {
            self.ctl.p[i] = PARAMS[i].clamp(v);
            self.dirty = true;
        }
    }

    pub fn set_all(&mut self, vals: &[f32]) {
        for (i, &v) in vals.iter().enumerate().take(NUM_PARAMS) {
            self.ctl.p[i] = PARAMS[i].clamp(v);
        }
        self.dirty = true;
    }

    pub fn param(&self, p: P) -> f32 {
        self.ctl.get(p)
    }

    pub fn active_voices(&self) -> usize {
        self.voices.iter().filter(|v| v.active).count()
    }

    pub fn note_on(&mut self, note: u8, vel: f32) {
        if self.dirty {
            self.update_globals();
        }
        self.age += 1;
        let idx = self
            .voices
            .iter()
            .position(|v| v.active && v.note == note)
            .or_else(|| self.voices.iter().position(|v| !v.active))
            .unwrap_or_else(|| {
                // Steal: oldest released voice, otherwise the oldest voice.
                let released = self
                    .voices
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| !v.held)
                    .min_by_key(|(_, v)| v.age)
                    .map(|(i, _)| i);
                released.unwrap_or_else(|| {
                    self.voices.iter().enumerate().min_by_key(|(_, v)| v.age).map(|(i, _)| i).unwrap_or(0)
                })
            });
        let from = self.last_pitch;
        let v = &mut self.voices[idx];
        if v.active && v.note != note {
            v.kill();
        }
        v.note_on(note, vel, from, &self.ctl, &self.lorenz, self.age);
        self.last_pitch = note as f32;
    }

    pub fn note_off(&mut self, note: u8) {
        for v in self.voices.iter_mut().filter(|v| v.active && v.held && v.note == note) {
            v.note_off();
        }
    }

    pub fn all_notes_off(&mut self) {
        for v in &mut self.voices {
            v.note_off();
        }
    }

    pub fn panic(&mut self) {
        for v in &mut self.voices {
            v.kill();
        }
    }

    fn update_globals(&mut self) {
        self.ctl.update();
        let sr = self.sr;
        let g = |p: P| self.ctl.get(p);
        for ch in 0..2 {
            let eq = &mut self.eq[ch];
            eq[0].highpass(g(P::LowCut), 0.707, sr);
            eq[1].low_shelf(150.0, g(P::Low), sr);
            eq[2].peak(g(P::MidFreq), 0.9, g(P::Mid), sr);
            eq[3].high_shelf(5000.0, g(P::High), sr);
            eq[4].lowpass(g(P::HighCut), 0.707, sr);
        }
        self.reverb.decay = g(P::RvDecay);
        self.reverb.damping = g(P::RvDamp);
        self.reverb.predelay = g(P::RvPre) * 0.001 * sr;
        self.reverb.excursion = g(P::RvMod) * 32.0;
        self.reverb.set_size(g(P::RvSize));
        self.dirty = false;
    }

    /// Render `n` (≤ BLOCK) frames into `self.out` (L in 0..BLOCK, R in BLOCK..).
    pub fn process(&mut self, n: usize) {
        let n = n.min(BLOCK);
        if self.dirty {
            self.update_globals();
        }
        let sr = self.sr;
        self.buf_l[..n].fill(0.0);
        self.buf_r[..n].fill(0.0);
        let mut act = 0.0;
        for v in self.voices.iter_mut().filter(|v| v.active) {
            v.render(&self.ctl, &mut self.buf_l[..n], &mut self.buf_r[..n]);
            act += v.env.max(v.gate);
        }

        let p = &self.ctl;
        let hum = p.get(P::Hum);
        let symp = p.get(P::Sympathy);
        let corrode = p.get(P::Corrode);
        let mode = LadderMode::from_index(p.get(P::FilterType));
        let cut_target = p.get(P::Cutoff);
        let reso = p.get(P::Reso);
        let drive = p.get(P::Drive);
        let drive_target = 1.0 + drive * drive * 12.0;
        let vol_target = db_to_gain(p.get(P::Volume));
        let mix_target = p.get(P::RvMix);
        let crush_len = 1.0 + corrode * corrode * 24.0;
        let levels = 2f32.powf(15.0 - corrode * 12.5);
        let act_target = act.min(1.0);

        let mut sum_l = 0.0;
        let mut sum_r = 0.0;
        let mut peak = 0.0f32;
        for i in 0..n {
            if self.ctrl_count == 0 {
                self.sm_cut += (cut_target - self.sm_cut) * 0.25;
                self.ladder[0].set(self.sm_cut, reso, sr);
                self.ladder[1].set(self.sm_cut, reso, sr);
                self.lorenz.step(self.ctl.chaos_dt);
                self.ticks = self.ticks.wrapping_add(1);
                if self.ticks.is_multiple_of(4) {
                    let (x, y, z) = self.lorenz.out();
                    let w = tm::TRAIL + (self.trail_count as usize % tm::TRAIL_LEN) * 3;
                    self.telemetry[w] = x;
                    self.telemetry[w + 1] = y;
                    self.telemetry[w + 2] = z;
                    self.trail_count = self.trail_count.wrapping_add(1);
                }
            }
            self.ctrl_count = (self.ctrl_count + 1) % CTRL;
            self.activity += (act_target - self.activity) * 0.002;

            let mut l = self.buf_l[i];
            let mut r = self.buf_r[i];

            if hum > 0.0 {
                self.hum_ph = (self.hum_ph + 50.0 / sr).fract();
                let h = 0.6 * ftanh(3.0 * fsin(self.hum_ph))
                    + 0.3 * fsin(self.hum_ph * 3.0) * fsin(self.hum_ph * 0.5 + 0.1)
                    + 0.05 * self.noise.bi();
                let h = h * hum * 0.18 * self.activity;
                l += h;
                r += h;
            }

            if symp > 0.0 {
                let (m, s, _) = self.symp.process(0.0, (l + r) * 0.5 * symp, 0.0);
                l += symp * 0.5 * (m + s);
                r += symp * 0.5 * (m - s);
            }

            if corrode > 0.0 {
                self.crush_cnt += 1.0;
                if self.crush_cnt >= crush_len {
                    self.crush_cnt -= crush_len;
                    self.crush_hold = [(l * levels).round() / levels, (r * levels).round() / levels];
                }
                l = lerp(l, self.crush_hold[0], (corrode * 4.0).min(1.0));
                r = lerp(r, self.crush_hold[1], (corrode * 4.0).min(1.0));
            }

            self.sm_drive += (drive_target - self.sm_drive) * 0.001;
            l = self.ladder[0].process(l, self.sm_drive, mode);
            r = self.ladder[1].process(r, self.sm_drive, mode);

            for b in &mut self.eq[0] {
                l = b.process(l);
            }
            for b in &mut self.eq[1] {
                r = b.process(r);
            }

            self.sm_mix += (mix_target - self.sm_mix) * 0.001;
            let (wl, wr) = self.reverb.process((l + r) * 0.5 + 1e-20);
            let dry = (1.0 - self.sm_mix).sqrt();
            let wet = self.sm_mix.sqrt() * 0.9;
            l = l * dry + wl * wet;
            r = r * dry + wr * wet;

            self.sm_vol += (vol_target - self.sm_vol) * 0.002;
            l = ftanh(l * self.sm_vol);
            r = ftanh(r * self.sm_vol);
            if !l.is_finite() || !r.is_finite() {
                l = 0.0;
                r = 0.0;
            }
            sum_l += l * l;
            sum_r += r * r;
            peak = peak.max(l.abs()).max(r.abs());
            self.out[i] = l;
            self.out[BLOCK + i] = r;
        }
        self.symp.sanitize();

        // Telemetry.
        let t = &mut self.telemetry;
        let (x, y, z) = self.lorenz.out();
        t[tm::LORENZ] = x;
        t[tm::LORENZ + 1] = y;
        t[tm::LORENZ + 2] = z;
        t[tm::RMS_L] = (sum_l / n.max(1) as f32).sqrt();
        t[tm::RMS_R] = (sum_r / n.max(1) as f32).sqrt();
        t[tm::PEAK] = peak;
        t[tm::VOICES] = self.voices.iter().filter(|v| v.active).count() as f32;
        t[tm::ACTIVITY] = self.activity;
        t[tm::TRAIL_COUNT] = self.trail_count as f32;
        let loudest = self
            .voices
            .iter()
            .filter(|v| v.active)
            .max_by(|a, b| a.gate.max(a.env).total_cmp(&b.gate.max(b.env)).then(a.age.cmp(&b.age)));
        if let Some(v) = loudest {
            v.modal.amplitudes(&mut t[tm::MODE_AMP..tm::MODE_AMP + NMODES]);
            t[tm::F0] = v.f0();
            t[tm::LOGISTIC] = v.logistic;
        } else {
            for a in &mut t[tm::MODE_AMP..tm::MODE_AMP + NMODES] {
                *a *= 0.9;
            }
        }
        t[tm::MODE_RATIO..tm::MODE_RATIO + NMODES].copy_from_slice(&self.ctl.ratios);
    }
}
