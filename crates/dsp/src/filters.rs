use crate::util::{ftanh, PI};

/// Topology-preserving state variable filter (Zavalishin / Simper).
#[derive(Clone, Copy, Debug, Default)]
pub struct Svf {
    ic1: f32,
    ic2: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    k: f32,
}

impl Svf {
    pub fn set(&mut self, freq: f32, q: f32, sr: f32) {
        let g = (PI * (freq / sr).clamp(1e-5, 0.49)).tan();
        self.k = 1.0 / q.max(0.05);
        self.a1 = 1.0 / (1.0 + g * (g + self.k));
        self.a2 = g * self.a1;
        self.a3 = g * self.a2;
    }

    /// Returns (low, band, high). Band is normalised to unity peak gain.
    #[inline]
    pub fn process(&mut self, x: f32) -> (f32, f32, f32) {
        let v3 = x - self.ic2;
        let v1 = self.a1 * self.ic1 + self.a2 * v3;
        let v2 = self.ic2 + self.a2 * self.ic1 + self.a3 * v3;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        (v2, v1 * self.k, x - self.k * v1 - v2)
    }

    pub fn reset(&mut self) {
        self.ic1 = 0.0;
        self.ic2 = 0.0;
    }
}

/// One-pole low-pass, `a` = feedback coefficient (0 = bypass).
#[derive(Clone, Copy, Debug, Default)]
pub struct OnePole {
    pub y: f32,
}

impl OnePole {
    #[inline]
    pub fn lp(&mut self, x: f32, a: f32) -> f32 {
        self.y = x + (self.y - x) * a;
        self.y
    }
}

/// One-pole coefficient for a cutoff frequency.
#[inline]
pub fn onepole_coef(freq: f32, sr: f32) -> f32 {
    (-2.0 * PI * freq / sr).exp()
}

/// DC blocker.
#[derive(Clone, Copy, Debug, Default)]
pub struct DcBlock {
    x1: f32,
    y1: f32,
}

impl DcBlock {
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = x - self.x1 + 0.9985 * self.y1;
        self.x1 = x;
        self.y1 = if y.abs() < 1e-20 { 0.0 } else { y };
        y
    }
}

/// Zero-delay-feedback Moog ladder with a tanh input stage and
/// Xpander-style mode mixing.
#[derive(Clone, Copy, Debug, Default)]
pub struct Ladder {
    s: [f32; 4],
    big_g: f32,
    k: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LadderMode {
    Lp12,
    Lp24,
    Bp,
    Hp,
}

impl LadderMode {
    pub fn from_index(i: f32) -> Self {
        match i.round() as i32 {
            0 => LadderMode::Lp12,
            2 => LadderMode::Bp,
            3 => LadderMode::Hp,
            _ => LadderMode::Lp24,
        }
    }
}

impl Ladder {
    pub fn set(&mut self, cutoff: f32, reso: f32, sr: f32) {
        let g = (PI * (cutoff / sr).clamp(1e-4, 0.45)).tan();
        self.big_g = g / (1.0 + g);
        self.k = reso.clamp(0.0, 1.0) * 3.95;
    }

    #[inline]
    pub fn process(&mut self, x: f32, drive: f32, mode: LadderMode) -> f32 {
        let gg = self.big_g;
        let one_minus = 1.0 - gg; // = 1/(1+g)
        let sig = gg * gg * gg * self.s[0] * one_minus
            + gg * gg * self.s[1] * one_minus
            + gg * self.s[2] * one_minus
            + self.s[3] * one_minus;
        let g4 = gg * gg * gg * gg;
        // Bass loss compensation for the low-pass modes.
        let comp = if matches!(mode, LadderMode::Lp12 | LadderMode::Lp24) { 1.0 + 0.5 * self.k } else { 1.0 };
        let u = (x * drive * comp - self.k * sig) / (1.0 + self.k * g4);
        let u = ftanh(u);
        let mut inp = u;
        let mut y = [0.0f32; 4];
        for i in 0..4 {
            let v = (inp - self.s[i]) * gg;
            y[i] = v + self.s[i];
            self.s[i] = y[i] + v;
            inp = y[i];
        }
        for s in &mut self.s {
            if !s.is_finite() || s.abs() < 1e-20 {
                *s = 0.0;
            }
        }
        let out = match mode {
            LadderMode::Lp12 => y[1],
            LadderMode::Lp24 => y[3],
            LadderMode::Bp => 4.0 * (y[1] - 2.0 * y[2] + y[3]),
            LadderMode::Hp => u - 2.0 * y[0] + y[1],
        };
        out / drive.max(1.0).sqrt()
    }
}

/// RBJ biquad, transposed direct form II.
#[derive(Clone, Copy, Debug)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Default for Biquad {
    fn default() -> Self {
        Biquad { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, z1: 0.0, z2: 0.0 }
    }
}

impl Biquad {
    fn set_raw(&mut self, b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) {
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    fn w(freq: f32, sr: f32) -> (f32, f32) {
        let w = 2.0 * PI * (freq / sr).clamp(1e-5, 0.49);
        (w.cos(), w.sin())
    }

    pub fn lowpass(&mut self, f: f32, q: f32, sr: f32) {
        let (c, s) = Self::w(f, sr);
        let al = s / (2.0 * q);
        self.set_raw((1.0 - c) / 2.0, 1.0 - c, (1.0 - c) / 2.0, 1.0 + al, -2.0 * c, 1.0 - al);
    }

    pub fn highpass(&mut self, f: f32, q: f32, sr: f32) {
        let (c, s) = Self::w(f, sr);
        let al = s / (2.0 * q);
        self.set_raw((1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0, 1.0 + al, -2.0 * c, 1.0 - al);
    }

    pub fn peak(&mut self, f: f32, q: f32, db: f32, sr: f32) {
        let a = 10f32.powf(db / 40.0);
        let (c, s) = Self::w(f, sr);
        let al = s / (2.0 * q);
        self.set_raw(1.0 + al * a, -2.0 * c, 1.0 - al * a, 1.0 + al / a, -2.0 * c, 1.0 - al / a);
    }

    pub fn low_shelf(&mut self, f: f32, db: f32, sr: f32) {
        let a = 10f32.powf(db / 40.0);
        let (c, s) = Self::w(f, sr);
        let al = s / 2.0 * 2f32.sqrt(); // shelf slope 1
        let sa = 2.0 * a.sqrt() * al;
        self.set_raw(
            a * ((a + 1.0) - (a - 1.0) * c + sa),
            2.0 * a * ((a - 1.0) - (a + 1.0) * c),
            a * ((a + 1.0) - (a - 1.0) * c - sa),
            (a + 1.0) + (a - 1.0) * c + sa,
            -2.0 * ((a - 1.0) + (a + 1.0) * c),
            (a + 1.0) + (a - 1.0) * c - sa,
        );
    }

    pub fn high_shelf(&mut self, f: f32, db: f32, sr: f32) {
        let a = 10f32.powf(db / 40.0);
        let (c, s) = Self::w(f, sr);
        let al = s / 2.0 * 2f32.sqrt();
        let sa = 2.0 * a.sqrt() * al;
        self.set_raw(
            a * ((a + 1.0) + (a - 1.0) * c + sa),
            -2.0 * a * ((a - 1.0) + (a + 1.0) * c),
            a * ((a + 1.0) + (a - 1.0) * c - sa),
            (a + 1.0) - (a - 1.0) * c + sa,
            2.0 * ((a - 1.0) - (a + 1.0) * c),
            (a + 1.0) - (a - 1.0) * c - sa,
        );
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        if !self.z1.is_finite() {
            self.z1 = 0.0;
            self.z2 = 0.0;
        }
        y
    }
}
