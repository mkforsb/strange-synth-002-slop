//! Jon Dattorro's plate reverb ("Effect Design Part 1", JAES 1997).
//! Delay lengths are given at the paper's 29761 Hz and scaled.

use crate::filters::OnePole;
use crate::util::{fsin, sane, Delay};

const BASE_SR: f32 = 29761.0;
const IN_AP: [(f32, f32); 4] = [(142.0, 0.75), (107.0, 0.75), (379.0, 0.625), (277.0, 0.625)];
const MAX_SIZE: f32 = 2.0;

struct Allpass {
    buf: Delay,
}

impl Allpass {
    fn new(max_len: f32) -> Self {
        Allpass { buf: Delay::new(max_len as usize + 4) }
    }
    #[inline]
    fn process(&mut self, x: f32, len: f32, g: f32) -> f32 {
        let d = self.buf.read(len);
        let v = x - g * d;
        self.buf.write(v);
        d + g * v
    }
    #[inline]
    fn tap(&self, d: f32) -> f32 {
        self.buf.read(d)
    }
}

pub struct Dattorro {
    k: f32,
    pre: Delay,
    bw: OnePole,
    input: [Allpass; 4],
    mod_ap: [Allpass; 2],
    d1: [Delay; 2],
    ap2: [Allpass; 2],
    d2: [Delay; 2],
    damp: [OnePole; 2],
    fb: [f32; 2],
    lfo: f32,
    lfo_inc: f32,
    size: f32,
    size_target: f32,
    pub decay: f32,
    pub damping: f32,
    pub predelay: f32,
    pub excursion: f32,
}

impl Dattorro {
    pub fn new(sr: f32) -> Self {
        let k = sr / BASE_SR;
        let m = k * MAX_SIZE;
        let exc_margin = 40.0 * k;
        Dattorro {
            k,
            pre: Delay::new((sr * 0.26) as usize),
            bw: OnePole::default(),
            input: IN_AP.map(|(l, _)| Allpass::new(l * k + 4.0)),
            mod_ap: [Allpass::new(672.0 * m + exc_margin), Allpass::new(908.0 * m + exc_margin)],
            d1: [Delay::new((4453.0 * m) as usize + 4), Delay::new((4217.0 * m) as usize + 4)],
            ap2: [Allpass::new(1800.0 * m), Allpass::new(2656.0 * m)],
            d2: [Delay::new((3720.0 * m) as usize + 4), Delay::new((3163.0 * m) as usize + 4)],
            damp: [OnePole::default(); 2],
            fb: [0.0; 2],
            lfo: 0.0,
            lfo_inc: 0.9 / sr,
            size: 1.0,
            size_target: 1.0,
            decay: 0.7,
            damping: 0.3,
            predelay: 0.0,
            excursion: 16.0,
        }
    }

    pub fn set_size(&mut self, s: f32) {
        self.size_target = s.clamp(0.4, MAX_SIZE);
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> (f32, f32) {
        self.size += (self.size_target - self.size) * 0.0002;
        let s = self.k * self.size;
        let k = self.k;

        let mut v = if self.predelay >= 1.0 {
            let d = self.pre.read(self.predelay);
            self.pre.write(x);
            d
        } else {
            self.pre.write(x);
            x
        };
        v = self.bw.lp(v, 0.05 + 0.3 * self.damping);
        for (ap, &(len, g)) in self.input.iter_mut().zip(IN_AP.iter()) {
            v = ap.process(v, len * k, g);
        }

        self.lfo += self.lfo_inc;
        if self.lfo >= 1.0 {
            self.lfo -= 1.0;
        }
        let exc = self.excursion * k;
        let m0 = fsin(self.lfo) * exc;
        let m1 = fsin(self.lfo + 0.25) * exc;

        let decay = self.decay;
        let damp = self.damping * 0.85;
        let fb_l = self.fb[1];
        let fb_r = self.fb[0];

        // Left half of the tank.
        let a = self.mod_ap[0].process(v + decay * fb_l, 672.0 * s + exc + m0, -0.7);
        let t = self.d1[0].read(4453.0 * s);
        self.d1[0].write(a);
        let t = self.damp[0].lp(t, damp) * decay;
        let b = self.ap2[0].process(t, 1800.0 * s, 0.5);
        let out_l = self.d2[0].read(3720.0 * s);
        self.d2[0].write(b);

        // Right half.
        let a = self.mod_ap[1].process(v + decay * fb_r, 908.0 * s + exc + m1, -0.7);
        let t = self.d1[1].read(4217.0 * s);
        self.d1[1].write(a);
        let t = self.damp[1].lp(t, damp) * decay;
        let b = self.ap2[1].process(t, 2656.0 * s, 0.5);
        let out_r = self.d2[1].read(3163.0 * s);
        self.d2[1].write(b);

        self.fb[0] = sane(out_l);
        self.fb[1] = sane(out_r);
        self.damp[0].y = sane(self.damp[0].y);
        self.damp[1].y = sane(self.damp[1].y);

        let yl = self.d1[1].read(266.0 * s) + self.d1[1].read(2974.0 * s) - self.ap2[1].tap(1913.0 * s)
            + self.d2[1].read(1996.0 * s)
            - self.d1[0].read(1990.0 * s)
            - self.ap2[0].tap(187.0 * s)
            - self.d2[0].read(1066.0 * s);
        let yr = self.d1[0].read(353.0 * s) + self.d1[0].read(3627.0 * s) - self.ap2[0].tap(1228.0 * s)
            + self.d2[0].read(2673.0 * s)
            - self.d1[1].read(2111.0 * s)
            - self.ap2[1].tap(335.0 * s)
            - self.d2[1].read(121.0 * s);
        (yl * 0.6, yr * 0.6)
    }
}
