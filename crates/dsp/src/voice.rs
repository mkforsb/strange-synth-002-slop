//! One OXIDE voice.
//!
//! ```text
//!  strike ─┐                       ┌─ modal metal body (16 modes) ─┐
//!  grit  ──┼─ impulse ─────────────┤                               │
//!  breath ─┼─ continuous ──────────┼─ waveguide string ────────────┼─ body ─┐
//!  bow  ◄──┴─ stick-slip friction ◄┘   (× damper gate)             │        │
//!                                                                  │        ├─ pipe ─ throat ─ alien ─ spin ─► L/R
//!  growl/swarm ─┐                                                  │        │
//!  scream ──────┼─ fold ─┬─ (× ADSR) ── src ───────────────────────┼────────┘
//!  glass ───────┤        └─ couple ──► resonators
//!  thoughts ────┘
//!  Lorenz ─► drift (pitch), unrest (modes, vowel, bow)
//! ```

use crate::chaos::Lorenz;
use crate::filters::{DcBlock, OnePole, Svf};
use crate::modal::{ModalBank, JITTER_A, JITTER_B, NMODES};
use crate::util::*;
use oxide_core::{NUM_PARAMS, P};

/// Control-rate period in samples.
pub const CTRL: usize = 16;

const GLASS_RATIOS: [f32; 5] = [1.0, 2.32, 4.25, 6.63, 9.38];
const GLASS_AMPS: [f32; 5] = [1.0, 0.45, 0.25, 0.13, 0.07];
const SWARM_DETUNE: [f32; 4] = [-1.0, 0.63, -0.37, 0.89];
const ALIEN_RATIO: f32 = 2.645_751_3; // √7

/// (F1, F2, F3) and gains for A E I O U.
const VOWELS: [([f32; 3], [f32; 3]); 5] = [
    ([800.0, 1150.0, 2900.0], [1.0, 0.5, 0.25]),
    ([400.0, 1700.0, 2600.0], [1.0, 0.35, 0.3]),
    ([290.0, 2200.0, 3000.0], [1.0, 0.25, 0.25]),
    ([450.0, 800.0, 2830.0], [1.0, 0.55, 0.1]),
    ([325.0, 700.0, 2530.0], [1.0, 0.3, 0.06]),
];
const FORMANT_Q: [f32; 3] = [7.0, 10.0, 14.0];

/// Block-rate snapshot of parameters plus derived coefficients, shared by
/// all voices.
pub struct VoiceCtl {
    pub p: [f32; NUM_PARAMS],
    pub sr: f32,
    pub att_inc: f32,
    pub dec_coef: f32,
    pub rel_coef: f32,
    pub glide_coef: f32,
    pub zap_coef: f32,
    pub elev_step: f32,
    pub spin_inc: f32,
    pub tick_inc: f32,
    pub logistic_r: f32,
    pub blip_coef: f32,
    pub strike_len: f32,
    pub strike_lp: f32,
    pub breath_lp: f32,
    pub grit_prob: f32,
    pub bow_slope: f32,
    pub chaos_dt: f32,
    pub ratios: [f32; NMODES],
    pub t60_rel: [f32; NMODES],
}

impl VoiceCtl {
    pub fn new(sr: f32) -> Self {
        let mut c = VoiceCtl {
            p: oxide_core::default_values(),
            sr,
            att_inc: 0.0,
            dec_coef: 0.0,
            rel_coef: 0.0,
            glide_coef: 1.0,
            zap_coef: 0.0,
            elev_step: 0.0,
            spin_inc: 0.0,
            tick_inc: 0.0,
            logistic_r: 3.7,
            blip_coef: 0.0,
            strike_len: 1.0,
            strike_lp: 0.0,
            breath_lp: 0.0,
            grit_prob: 0.0,
            bow_slope: 3.0,
            chaos_dt: 0.0,
            ratios: [1.0; NMODES],
            t60_rel: [1.0; NMODES],
        };
        c.update();
        c
    }

    #[inline]
    pub fn get(&self, p: P) -> f32 {
        self.p[p as usize]
    }

    pub fn update(&mut self) {
        let sr = self.sr;
        let g = |p: P| self.p[p as usize];
        let ctrl_dt = CTRL as f32 / sr;

        self.att_inc = 1.0 / (g(P::Attack) * sr);
        self.dec_coef = (-4.6 / (g(P::Decay) * sr)).exp();
        self.rel_coef = t60_coef(g(P::Release), sr);
        let glide = g(P::Glide);
        self.glide_coef = if glide < 1e-3 { 1.0 } else { 1.0 - (-ctrl_dt * 3.0 / glide).exp() };
        self.zap_coef = (-ctrl_dt * 4.6 / g(P::ZapTime)).exp();
        let e = g(P::Elevator);
        self.elev_step = e * e.abs() * 24.0 * ctrl_dt;
        self.spin_inc = g(P::SpinRate) / sr;
        self.tick_inc = g(P::Tick) / sr;
        self.logistic_r = 3.45 + 0.55 * g(P::Logic);
        self.blip_coef = (-4.6 * g(P::Tick) / (0.8 * sr)).exp();

        let hard = g(P::Hardness);
        // Soft felt: 9 ms contact, steel: 0.15 ms.
        self.strike_len = ((0.15e-3 + (1.0 - hard).powi(2) * 9e-3) * sr).max(1.0);
        self.strike_lp = crate::filters::onepole_coef(250.0 * 2f32.powf(hard * 6.5), sr);
        self.breath_lp = crate::filters::onepole_coef(300.0 * 2f32.powf(hard * 5.0 + 0.5), sr);
        let grit = g(P::Grit);
        self.grit_prob = grit * grit * 0.03;
        self.bow_slope = 1.2 + 7.0 * (1.0 - g(P::Pressure));
        self.chaos_dt = crate::chaos::chaos_speed(g(P::Chaos)) * ctrl_dt;

        // Mode ratios: stiff-string stretch k·sqrt(1 + B·k²) morphs from
        // harmonic through bars and plates to bells; Shatter scatters them.
        let stiff = g(P::Stiffness);
        let b = stiff * stiff * 0.12;
        let shatter = g(P::Shatter);
        let rust = g(P::Rust);
        for k in 0..NMODES {
            let kf = (k + 1) as f32;
            let mut r = kf * (1.0 + b * kf * kf).sqrt() / (1.0 + b).sqrt();
            if k > 0 {
                r *= (shatter * 0.45 * crate::modal::SHATTER[k]).exp();
                r = r.max(1.01);
            }
            self.ratios[k] = r;
            self.t60_rel[k] = 1.0 / (1.0 + rust * 5.0 * (r - 1.0).max(0.0).powf(0.7));
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Attack,
    Decay,
    Release,
    Idle,
}

pub struct Voice {
    pub note: u8,
    pub active: bool,
    pub held: bool,
    pub age: u64,
    vel: f32,
    sr: f32,

    pitch: f32,
    target: f32,
    f0: f32,

    stage: Stage,
    pub env: f32,
    pub gate: f32,

    lorenz: Lorenz,
    noise: Noise,
    jitter: f32,

    // Exciters
    burst_pos: f32,
    burst_len: f32,
    burst_amp: f32,
    strike_lp: OnePole,
    disp_lp: OnePole,
    breath_lp: OnePole,
    bow_v: f32,

    // Resonators
    pub modal: ModalBank,
    string: Delay,
    string_lp: OnePole,
    string_len: f32,
    string_gain: f32,
    string_a: f32,
    pipe: Delay,
    pipe_lp: OnePole,
    pipe_len: f32,
    formant: [Svf; 3],
    formant_g: [f32; 3],

    // Oscillators
    ph_saw: [f32; 5],
    ph_sub: f32,
    ph_fm: f32,
    fm_prev: [f32; 2],
    ph_glass: [f32; 10],
    ph_ring: f32,
    ph_spin: f32,
    ph_blip: f32,
    blip_freq: f32,
    blip_env: f32,
    blip_timer: f32,
    pub logistic: f32,

    zap_env: f32,
    elev: f32,
    pan: f32,
    ctrl_count: usize,
    quiet: u32,
    dc: [DcBlock; 2],
}

impl Voice {
    pub fn new(sr: f32, seed: u32) -> Self {
        let max_delay = (sr / 8.0) as usize + 8;
        let mut noise = Noise::new(seed);
        Voice {
            note: 0,
            active: false,
            held: false,
            age: 0,
            vel: 1.0,
            sr,
            pitch: 60.0,
            target: 60.0,
            f0: 261.6,
            stage: Stage::Idle,
            env: 0.0,
            gate: 0.0,
            lorenz: Lorenz::new(1.0, 1.0, 20.0),
            jitter: 0.0,
            burst_pos: 1e9,
            burst_len: 1.0,
            burst_amp: 0.0,
            strike_lp: OnePole::default(),
            disp_lp: OnePole::default(),
            breath_lp: OnePole::default(),
            bow_v: 0.0,
            modal: ModalBank::default(),
            string: Delay::new(max_delay),
            string_lp: OnePole::default(),
            string_len: 100.0,
            string_gain: 0.0,
            string_a: 0.0,
            pipe: Delay::new(max_delay),
            pipe_lp: OnePole::default(),
            pipe_len: 50.0,
            formant: [Svf::default(); 3],
            formant_g: [0.0; 3],
            ph_saw: [noise.uni(), noise.uni(), noise.uni(), noise.uni(), noise.uni()],
            ph_sub: 0.0,
            ph_fm: 0.0,
            fm_prev: [0.0; 2],
            ph_glass: [0.0; 10],
            ph_ring: 0.0,
            ph_spin: noise.uni(),
            ph_blip: 0.0,
            blip_freq: 440.0,
            blip_env: 0.0,
            blip_timer: 1.0,
            logistic: 0.37,
            zap_env: 0.0,
            elev: 0.0,
            pan: 0.0,
            ctrl_count: 0,
            quiet: 0,
            dc: [DcBlock::default(); 2],
            noise,
        }
    }

    pub fn note_on(&mut self, note: u8, vel: f32, from_pitch: f32, ctl: &VoiceCtl, seed_chaos: &Lorenz, age: u64) {
        let retrigger = self.active && self.note == note;
        if !self.active {
            // Fresh voice: clear everything that rings.
            self.modal.reset();
            self.string.clear();
            self.pipe.clear();
            self.formant.iter_mut().for_each(|f| f.reset());
            self.fm_prev = [0.0; 2];
            self.bow_v = 0.0;
            self.env = 0.0;
            self.gate = 0.0;
            for (i, ph) in self.ph_glass.iter_mut().enumerate() {
                *ph = (i as f32 * 0.137) % 1.0;
            }
        }
        self.note = note;
        self.vel = vel.clamp(0.0, 1.0);
        self.active = true;
        self.held = true;
        self.age = age;
        self.target = note as f32;
        if !retrigger {
            self.pitch = if ctl.get(P::Glide) > 1e-3 { from_pitch } else { self.target };
            self.elev = 0.0;
        }
        self.stage = Stage::Attack;
        self.zap_env = 1.0;
        self.burst_pos = 0.0;
        self.burst_len = ctl.strike_len;
        let strike = ctl.get(P::Strike);
        // Unit area pulse scaled by strike·velocity: same momentum, longer
        // contact for soft mallets.
        self.burst_amp = strike * (0.25 + 0.75 * self.vel) * PI / (2.0 * self.burst_len);
        // Each voice gets its own chaotic trajectory, diverging from the
        // global attractor by a tiny nudge.
        self.lorenz = *seed_chaos;
        self.lorenz.x += self.noise.bi() * 0.01;
        self.logistic = 0.2 + 0.6 * self.noise.uni();
        self.blip_timer = 1.0;
        let width = ctl.get(P::Width);
        self.pan = ((note as f32 - 60.0) / 24.0 * width * 0.6).clamp(-0.8, 0.8);
        self.quiet = 0;
        self.ctrl_count = 0;
    }

    pub fn note_off(&mut self) {
        self.held = false;
        if self.stage != Stage::Idle {
            self.stage = Stage::Release;
        }
    }

    pub fn kill(&mut self) {
        self.active = false;
        self.held = false;
        self.stage = Stage::Idle;
        self.env = 0.0;
        self.gate = 0.0;
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn f0(&self) -> f32 {
        self.f0
    }

    fn control(&mut self, c: &VoiceCtl) {
        let sr = self.sr;
        self.lorenz.step(c.chaos_dt);
        let (lx, ly, lz) = self.lorenz.out();

        self.pitch += (self.target - self.pitch) * c.glide_coef;
        self.zap_env *= c.zap_coef;
        self.elev = (self.elev + c.elev_step).clamp(-48.0, 48.0);
        self.jitter = self.jitter * 0.8 + 0.2 * self.noise.bi();

        let drift = c.get(P::Drift);
        let zap = c.get(P::Zap);
        let spin = c.get(P::Spin);
        let p = self.pitch
            + c.get(P::Octave) * 12.0
            + c.get(P::Tune)
            + drift * drift * 12.0 * lx
            + zap * zap.abs() * 60.0 * self.zap_env
            + self.elev
            + spin * 0.5 * fsin(self.ph_spin);
        let f0 = midi_to_hz(p).clamp(8.0, sr * 0.45);
        self.f0 = f0;

        let unrest = c.get(P::Unrest);
        if c.get(P::Metal) > 0.0 || c.get(P::Bow) > 0.0 {
            let ring = c.get(P::Ring);
            let mut freqs = [0.0; NMODES];
            let mut t60 = [0.0; NMODES];
            for k in 0..NMODES {
                let j = if k > 0 { unrest * 0.06 * (ly * JITTER_A[k] + lz * JITTER_B[k]) } else { 0.0 };
                freqs[k] = f0 * c.ratios[k] * (1.0 + j);
                t60[k] = ring * c.t60_rel[k];
            }
            self.modal.tune(&freqs, &t60, sr);
        }

        if c.get(P::String) > 0.0 {
            let mute = c.get(P::Mute);
            self.string_a = 0.05 + 0.88 * mute;
            let lp_delay = self.string_a / (1.0 - self.string_a);
            self.string_len = (sr / f0 - lp_delay).max(1.0);
            let t = c.get(P::Ring) * (1.0 - mute) * (1.0 - mute) + 0.03;
            self.string_gain = 10f32.powf(-3.0 / (t * f0));
        }

        if c.get(P::Pipe) > 0.0 {
            self.pipe_len = (sr / (2.0 * f0)).max(1.0);
        }

        if c.get(P::Throat) > 0.0 {
            let v = (c.get(P::Vowel) + unrest * 0.35 * lz).clamp(0.0, 1.0) * 4.0;
            let i = (v as usize).min(3);
            let t = v - i as f32;
            let scale = 2f32.powf((0.5 - c.get(P::Size)) * 2.4);
            let (fa, ga) = VOWELS[i];
            let (fb, gb) = VOWELS[i + 1];
            for n in 0..3 {
                let f = lerp(fa[n], fb[n], t) * scale;
                self.formant[n].set(f.clamp(40.0, sr * 0.45), FORMANT_Q[n], sr);
                self.formant_g[n] = lerp(ga[n], gb[n], t);
            }
        }
    }

    /// Render and add into `out_l` / `out_r`.
    pub fn render(&mut self, c: &VoiceCtl, out_l: &mut [f32], out_r: &mut [f32]) {
        let sr = self.sr;
        let inv_sr = 1.0 / sr;
        let strike_on = c.get(P::Strike) > 0.0;
        let hard = c.get(P::Hardness);
        let grit = c.get(P::Grit);
        let breath = c.get(P::Breath);
        let bow = c.get(P::Bow);
        let metal = c.get(P::Metal);
        let string = c.get(P::String);
        let pipe = c.get(P::Pipe);
        let pipe_fb = 0.55 + 0.42 * pipe;
        let pipe_norm = (1.0 - pipe_fb).sqrt();
        let growl = c.get(P::Growl);
        let swarm = c.get(P::Swarm);
        let fold = c.get(P::Fold);
        let scream = c.get(P::Scream);
        let throat = c.get(P::Throat);
        let glass = c.get(P::Glass);
        let wobble = c.get(P::Wobble);
        let thought = c.get(P::Thought);
        let couple = c.get(P::Couple);
        let unrest = c.get(P::Unrest);
        let alien = c.get(P::Alien);
        let spin = c.get(P::Spin);
        let width = c.get(P::Width);
        let sus = c.get(P::Sustain);
        let velgain = 0.35 + 0.65 * self.vel;
        let resonate = metal > 0.0 || bow > 0.0;
        let side_gain = width * (1.0 - pipe) * (1.0 - throat);

        for i in 0..out_l.len() {
            if self.ctrl_count == 0 {
                self.control(c);
            }
            self.ctrl_count = (self.ctrl_count + 1) % CTRL;
            let f0 = self.f0;
            let dt = f0 * inv_sr;

            // ── Envelopes ──────────────────────────────────────────────
            match self.stage {
                Stage::Attack => {
                    self.env += c.att_inc;
                    if self.env >= 1.0 {
                        self.env = 1.0;
                        self.stage = Stage::Decay;
                    }
                }
                Stage::Decay => self.env = sus + (self.env - sus) * c.dec_coef,
                Stage::Release => {
                    self.env *= c.rel_coef;
                    if self.env < 1e-5 {
                        self.env = 0.0;
                        self.stage = Stage::Idle;
                    }
                }
                Stage::Idle => {}
            }
            if self.held {
                self.gate += (1.0 - self.gate) * 0.01;
            } else {
                self.gate *= c.rel_coef;
            }
            let env = self.env;

            // ── Exciters ───────────────────────────────────────────────
            // `burst` is area-normalised (momentum) for the modal body;
            // strings and tubes want displacement, so they get a
            // peak-normalised copy.
            let mut burst = 0.0;
            if strike_on && self.burst_pos < self.burst_len {
                let w = fsin(self.burst_pos / self.burst_len * 0.5);
                burst = w * self.burst_amp * (1.0 + hard * hard * 0.8 * self.noise.bi());
                self.burst_pos += 1.0;
            }
            let mut speck = 0.0;
            if grit > 0.0 && self.gate > 0.01 && self.noise.uni() < c.grit_prob {
                let a = self.noise.uni();
                speck = a * a * self.noise.bi() * (0.1 + 0.25 * grit) * self.gate;
            }
            let imp = self.strike_lp.lp(burst + speck, c.strike_lp);
            let disp = if string > 0.0 {
                self.disp_lp.lp(burst * self.burst_len * 0.6 + speck * 2.0, c.strike_lp)
            } else {
                0.0
            };

            let br = if breath > 0.0 {
                self.breath_lp.lp(self.noise.bi(), c.breath_lp) * breath * env
            } else {
                0.0
            };

            // ── Oscillators ────────────────────────────────────────────
            let mut src = 0.0;
            if growl > 0.0 {
                let gdt = dt * (1.0 + growl * 0.02 * self.jitter);
                let mut g = saw(self.ph_saw[0], gdt);
                self.ph_saw[0] = (self.ph_saw[0] + gdt).fract();
                // Period doubling: the sub-octave gives the roar its rasp.
                let sub = saw(self.ph_sub, gdt * 0.5);
                self.ph_sub = (self.ph_sub + gdt * 0.5).fract();
                g += 0.6 * sub * (0.5 + 0.5 * fsin(self.ph_sub * 0.5 + self.jitter * 0.1));
                if swarm > 0.0 {
                    for k in 0..4 {
                        let sdt = gdt * (1.0 + swarm * 0.035 * SWARM_DETUNE[k]);
                        g += swarm * saw(self.ph_saw[k + 1], sdt);
                        self.ph_saw[k + 1] = (self.ph_saw[k + 1] + sdt).fract();
                    }
                    g /= 1.0 + swarm * 1.5;
                }
                src += growl * g * 0.45;
            }
            if scream > 0.0 {
                let fb = scream * 0.4 * (self.fm_prev[0] + self.fm_prev[1]);
                let s = fsin(self.ph_fm + fb);
                self.fm_prev[1] = self.fm_prev[0];
                self.fm_prev[0] = s;
                self.ph_fm = (self.ph_fm + dt).fract();
                src += scream.sqrt() * 0.45 * s;
            }
            if fold > 0.0 {
                let folded = fsin(src * (0.25 + fold * 2.5));
                src = lerp(src, folded * 0.6, (fold * 3.0).min(1.0));
            }
            if glass > 0.0 {
                let beat = wobble * 7.0 * inv_sr;
                let mut s = 0.0;
                for k in 0..5 {
                    let inc = dt * GLASS_RATIOS[k];
                    if inc > 0.45 {
                        break;
                    }
                    let b = beat * (1.0 + k as f32 * 0.37);
                    s += GLASS_AMPS[k] * (fsin(self.ph_glass[2 * k]) + fsin(self.ph_glass[2 * k + 1]));
                    self.ph_glass[2 * k] = (self.ph_glass[2 * k] + inc).fract();
                    self.ph_glass[2 * k + 1] = (self.ph_glass[2 * k + 1] + inc + b).fract();
                }
                src += glass * s * 0.22;
            }
            if thought > 0.0 {
                self.blip_timer += c.tick_inc;
                if self.blip_timer >= 1.0 {
                    self.blip_timer -= 1.0;
                    let x = c.logistic_r * self.logistic * (1.0 - self.logistic);
                    self.logistic = if x.is_finite() && x > 1e-4 && x < 0.9999 { x } else { 0.1 + 0.8 * self.noise.uni() };
                    let x = self.logistic;
                    if x > 0.12 {
                        // Whole-tone quantised: a slightly alien scale.
                        let st = ((x * 15.0).floor() * 2.0 - 12.0) + 12.0;
                        self.blip_freq = f0 * semis_to_ratio(st);
                        self.blip_env = 1.0;
                    }
                }
                self.blip_env *= c.blip_coef;
                let s = fsin(self.ph_blip);
                self.ph_blip = (self.ph_blip + (self.blip_freq * inv_sr).min(0.45)).fract();
                let q = self.logistic;
                src += thought * self.blip_env * (s * (1.0 - q * 0.5) + q * 0.5 * ftanh(s * 5.0)) * 0.6;
            }
            let src = src * env * velgain;

            // ── Resonators ─────────────────────────────────────────────
            let cont = br + couple * src * 0.2;
            let mut body = 0.0;
            let mut side = 0.0;
            if resonate {
                let mut force = 0.0;
                if bow > 0.0 {
                    let vb = bow * 0.5 * env.max(self.gate * sus) * (1.0 + 0.04 * self.noise.bi() + unrest * 0.3 * self.lorenz.out().1);
                    let dv = vb - self.bow_v;
                    let t = ((dv * c.bow_slope).abs() + 0.75).powi(-4).min(1.0);
                    force = dv * t * 0.5;
                }
                let (m, s, v) = self.modal.process(imp * 6.0, cont, force);
                self.bow_v = v * 0.5;
                body += metal * m * 0.35;
                side += metal * s * 0.35;
            }
            if string > 0.0 {
                let rd = self.string.read(self.string_len);
                let s = self.string_lp.lp(rd, self.string_a) * self.string_gain;
                let s = sane(s);
                self.string.write(s + disp + cont * 0.2);
                body += string * s * 0.8;
            }
            let body = ftanh(body * self.gate) ;
            let mut v = body + src + br * 0.15;

            if pipe > 0.0 {
                let d = self.pipe.read(self.pipe_len);
                let y = v - pipe_fb * self.pipe_lp.lp(d, 0.2);
                let y = sane(y);
                self.pipe.write(y);
                v = lerp(v, y * pipe_norm, pipe);
            }
            if throat > 0.0 {
                let mut f = 0.0;
                for n in 0..3 {
                    f += self.formant[n].process(v).1 * self.formant_g[n];
                }
                v = lerp(v, f * 2.5, throat);
            }
            if alien > 0.0 {
                let carrier = fsin(self.ph_ring);
                self.ph_ring = (self.ph_ring + (f0 * ALIEN_RATIO * inv_sr).min(0.45)).fract();
                v = lerp(v, v * carrier * 1.4, alien);
            }

            // ── Spin & stereo ──────────────────────────────────────────
            let mut pan = self.pan;
            if spin > 0.0 {
                let s = fsin(self.ph_spin);
                let c4 = fsin(self.ph_spin + 0.25);
                self.ph_spin = (self.ph_spin + c.spin_inc).fract();
                v *= 1.0 - spin * 0.35 * (1.0 - c4);
                pan = (pan + s * spin * 0.9).clamp(-1.0, 1.0);
            }
            let gl = (0.5 * (1.0 - pan)).sqrt();
            let gr = (0.5 * (1.0 + pan)).sqrt();
            let side = side * self.gate * side_gain;
            let l = self.dc[0].process(v * gl + side);
            let r = self.dc[1].process(v * gr - side);
            out_l[i] += l;
            out_r[i] += r;

            if l.abs() + r.abs() < 1e-5 {
                self.quiet += 1;
            } else {
                self.quiet = 0;
            }
        }

        self.modal.sanitize();
        if !self.held && ((self.stage == Stage::Idle && self.gate < 1e-4) || (self.quiet > (sr * 0.25) as u32 && self.env < 1e-3)) {
            self.kill();
        }
        if !(self.f0.is_finite()) {
            self.kill();
        }
    }
}
