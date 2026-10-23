pub const PI: f32 = core::f32::consts::PI;
pub const TAU: f32 = core::f32::consts::TAU;

/// xorshift32 audio-rate noise.
#[derive(Clone, Copy, Debug)]
pub struct Noise(u32);

impl Noise {
    pub fn new(seed: u32) -> Self {
        Noise(seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453) | 1)
    }
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    /// Uniform in [0, 1).
    #[inline]
    pub fn uni(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 * (1.0 / 16_777_216.0)
    }
    /// Uniform in [-1, 1).
    #[inline]
    pub fn bi(&mut self) -> f32 {
        self.uni() * 2.0 - 1.0
    }
}

/// sin(2π·phase) for any phase, via a refined parabola (~0.1% error).
#[inline]
pub fn fsin(phase: f32) -> f32 {
    let x = phase - phase.floor(); // 0..1
    let t = 1.0 - 2.0 * x; // sin(2πx) = sin(π·t) for t in -1..1
    let y = 4.0 * t * (1.0 - t.abs());
    0.225 * (y * y.abs() - y) + y
}

#[inline]
pub fn ftanh(x: f32) -> f32 {
    let x = x.clamp(-3.0, 3.0);
    x * (27.0 + x * x) / (27.0 + 9.0 * x * x)
}

#[inline]
pub fn db_to_gain(db: f32) -> f32 {
    10f32.powf(db * 0.05)
}

#[inline]
pub fn semis_to_ratio(s: f32) -> f32 {
    (s * (core::f32::consts::LN_2 / 12.0)).exp()
}

#[inline]
pub fn midi_to_hz(n: f32) -> f32 {
    440.0 * semis_to_ratio(n - 69.0)
}

#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Per-sample decay coefficient so that a signal drops 60 dB in `t60` seconds.
#[inline]
pub fn t60_coef(t60: f32, sr: f32) -> f32 {
    (-6.907_755 / (t60.max(1e-4) * sr)).exp()
}

/// Flush denormals / garbage in feedback paths.
#[inline]
pub fn sane(x: f32) -> f32 {
    if x.is_finite() && x.abs() > 1e-18 { x } else { 0.0 }
}

/// PolyBLEP residual for band-limited saws.
#[inline]
pub fn polyblep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        t + t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + t + t + 1.0
    } else {
        0.0
    }
}

#[inline]
pub fn saw(phase: f32, dt: f32) -> f32 {
    2.0 * phase - 1.0 - polyblep(phase, dt)
}

/// Fractional delay line with linear interpolation.
#[derive(Clone, Debug)]
pub struct Delay {
    buf: Vec<f32>,
    w: usize,
}

impl Delay {
    pub fn new(len: usize) -> Self {
        Delay { buf: vec![0.0; len.max(4)], w: 0 }
    }
    pub fn clear(&mut self) {
        self.buf.iter_mut().for_each(|x| *x = 0.0);
    }
    #[inline]
    pub fn write(&mut self, x: f32) {
        self.buf[self.w] = x;
        self.w += 1;
        if self.w == self.buf.len() {
            self.w = 0;
        }
    }
    /// Read `d` samples behind the most recent write (d >= 1).
    #[inline]
    pub fn read(&self, d: f32) -> f32 {
        let n = self.buf.len();
        let d = d.clamp(1.0, (n - 2) as f32);
        let di = d as usize;
        let frac = d - di as f32;
        let i0 = (self.w + n - di) % n;
        let i1 = (i0 + n - 1) % n;
        self.buf[i0] + (self.buf[i1] - self.buf[i0]) * frac
    }
    #[inline]
    pub fn read_int(&self, d: usize) -> f32 {
        let n = self.buf.len();
        self.buf[(self.w + n - d.clamp(1, n - 1)) % n]
    }
}
