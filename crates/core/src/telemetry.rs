//! Layout of the telemetry block the DSP publishes for the visualisers.

pub const LORENZ: usize = 0; // x, y, z (normalised, -1..1)
pub const RMS_L: usize = 3;
pub const RMS_R: usize = 4;
pub const VOICES: usize = 5;
pub const PEAK: usize = 6;
pub const F0: usize = 7;
pub const LOGISTIC: usize = 8;
pub const ACTIVITY: usize = 9;
pub const TRAIL_COUNT: usize = 10;
pub const MODE_AMP: usize = 16; // NMODES values
pub const MODE_RATIO: usize = 32; // NMODES values
pub const TRAIL: usize = 48; // TRAIL_LEN × (x, y, z)

pub const NMODES: usize = 16;
pub const TRAIL_LEN: usize = 64;
pub const LEN: usize = TRAIL + TRAIL_LEN * 3;
