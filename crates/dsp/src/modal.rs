//! A bank of two-pole resonators: the metal body.

use crate::util::{t60_coef, TAU};

pub const NMODES: usize = 16;

/// Pseudo-random scatter per mode, used by Shatter.
pub const SHATTER: [f32; NMODES] = [
    0.0, 0.62, -0.41, 0.93, -0.78, 0.15, -0.97, 0.48, 0.81, -0.22, -0.66, 0.37, -0.09, 0.71, -0.53, 0.99,
];
/// Two decorrelated jitter patterns used by Unrest.
pub const JITTER_A: [f32; NMODES] = [
    0.0, -0.8, 0.3, 0.9, -0.5, -0.2, 0.7, -0.9, 0.4, 0.1, -0.6, 0.8, -0.3, 0.5, -0.7, 0.2,
];
pub const JITTER_B: [f32; NMODES] = [
    0.0, 0.5, -0.9, 0.2, 0.8, -0.6, -0.1, 0.4, -0.7, 0.9, 0.3, -0.4, 0.6, -0.8, 0.1, -0.5,
];

#[derive(Clone, Debug)]
pub struct ModalBank {
    y1: [f32; NMODES],
    y2: [f32; NMODES],
    c1: [f32; NMODES],
    c2: [f32; NMODES],
    b: [f32; NMODES],
    cosw: [f32; NMODES],
    gc: [f32; NMODES],
    exc: [f32; NMODES],
    pick: [f32; NMODES],
    side: [f32; NMODES],
    phi: [f32; NMODES],
    on: [bool; NMODES],
}

impl Default for ModalBank {
    fn default() -> Self {
        let mut m = ModalBank {
            y1: [0.0; NMODES],
            y2: [0.0; NMODES],
            c1: [0.0; NMODES],
            c2: [0.0; NMODES],
            b: [0.0; NMODES],
            cosw: [1.0; NMODES],
            gc: [0.0; NMODES],
            exc: [0.0; NMODES],
            pick: [0.0; NMODES],
            side: [0.0; NMODES],
            phi: [0.0; NMODES],
            on: [false; NMODES],
        };
        for k in 0..NMODES {
            let kf = (k + 1) as f32;
            // Strike, pickup and bow positions on the body.
            m.exc[k] = (0.35 + 0.65 * (kf * core::f32::consts::PI * 0.27).sin().abs()) / kf.sqrt();
            m.pick[k] = 0.4 + 0.6 * (kf * core::f32::consts::PI * 0.43).sin().abs();
            m.phi[k] = 0.25 + 0.75 * (kf * core::f32::consts::PI * 0.11).sin().abs();
            m.side[k] = if (k * 7 + 3) % 5 < 2 { -1.0 } else { 1.0 } * if k == 0 { 0.2 } else { 1.0 };
        }
        m
    }
}

impl ModalBank {
    pub fn reset(&mut self) {
        self.y1 = [0.0; NMODES];
        self.y2 = [0.0; NMODES];
    }

    /// Retune all modes. `freqs` in Hz, `t60` in seconds.
    pub fn tune(&mut self, freqs: &[f32; NMODES], t60: &[f32; NMODES], sr: f32) {
        for k in 0..NMODES {
            let f = freqs[k];
            if !(f > 10.0 && f < sr * 0.46) {
                self.on[k] = false;
                self.c1[k] = 0.0;
                self.c2[k] = 0.0;
                self.y1[k] = 0.0;
                self.y2[k] = 0.0;
                continue;
            }
            self.on[k] = true;
            let w = TAU * f / sr;
            let r = t60_coef(t60[k], sr);
            let (s, c) = w.sin_cos();
            self.cosw[k] = c;
            self.c1[k] = 2.0 * r * c;
            self.c2[k] = r * r;
            self.b[k] = s;
            // Normalise continuous excitation so a noise-driven mode has
            // roughly the same RMS whatever its decay time.
            self.gc[k] = (2.0 * (1.0 - r * r)).sqrt();
        }
    }

    /// One sample. `imp` = impulsive excitation, `cont` = continuous
    /// excitation, `force` = bow friction force.
    /// Returns (mono, side, body velocity at the bow point).
    #[inline]
    pub fn process(&mut self, imp: f32, cont: f32, force: f32) -> (f32, f32, f32) {
        let mut mono = 0.0;
        let mut side = 0.0;
        let mut vel = 0.0;
        for k in 0..NMODES {
            if !self.on[k] {
                continue;
            }
            let x = self.exc[k] * (imp + self.gc[k] * cont) + self.phi[k] * force;
            let y = self.c1[k] * self.y1[k] - self.c2[k] * self.y2[k] + self.b[k] * x;
            vel += self.phi[k] * (y - self.y1[k]);
            self.y2[k] = self.y1[k];
            self.y1[k] = y;
            let o = y * self.pick[k];
            mono += o;
            side += o * self.side[k];
        }
        (mono, side, vel)
    }

    /// Kill runaway or denormal states.
    pub fn sanitize(&mut self) {
        for k in 0..NMODES {
            let a = self.y1[k];
            let runaway = !a.is_finite() || a.abs() > 1e4;
            let denormal = a.abs() < 1e-15 && self.y2[k].abs() < 1e-15;
            if runaway || denormal {
                self.y1[k] = 0.0;
                self.y2[k] = 0.0;
            }
        }
    }

    /// Current amplitude of each mode (for the Chladni visualiser).
    pub fn amplitudes(&self, out: &mut [f32]) {
        for k in 0..NMODES.min(out.len()) {
            if !self.on[k] || self.b[k] < 1e-6 {
                out[k] = 0.0;
                continue;
            }
            let q = (self.y1[k] * self.cosw[k] - self.y2[k]) / self.b[k];
            out[k] = (self.y1[k] * self.y1[k] + q * q).sqrt() * self.pick[k];
        }
    }
}
