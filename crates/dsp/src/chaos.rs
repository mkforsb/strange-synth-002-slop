/// Lorenz attractor (σ=10, ρ=28, β=8/3), integrated with adaptive Euler
/// sub-steps so it stays stable from slow drift up to audio-rate turbulence.
#[derive(Clone, Copy, Debug)]
pub struct Lorenz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Lorenz {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Lorenz { x, y, z }
    }

    #[inline]
    pub fn step(&mut self, dt: f32) {
        let n = (dt * 250.0).ceil().clamp(1.0, 16.0);
        let h = dt / n;
        for _ in 0..n as usize {
            let dx = 10.0 * (self.y - self.x);
            let dy = self.x * (28.0 - self.z) - self.y;
            let dz = self.x * self.y - 2.666_666_7 * self.z;
            self.x += dx * h;
            self.y += dy * h;
            self.z += dz * h;
        }
        if !(self.x.is_finite() && self.y.is_finite() && self.z.is_finite()) || self.x.abs() > 200.0 {
            *self = Lorenz::new(1.0, 1.0, 20.0);
        }
    }

    /// Roughly normalised to -1..1.
    #[inline]
    pub fn out(&self) -> (f32, f32, f32) {
        (self.x * 0.05, self.y * 0.037, (self.z - 24.0) * 0.05)
    }
}

/// Chaos param (0..1) to Lorenz time units per second.
#[inline]
pub fn chaos_speed(c: f32) -> f32 {
    0.2 * 10f32.powf(c * 4.0)
}
