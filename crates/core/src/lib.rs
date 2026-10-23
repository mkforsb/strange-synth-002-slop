//! OXIDE shared definitions: parameters, factory presets and mutation.
//! Used by both the DSP engine (in the AudioWorklet) and the Dioxus UI.

pub mod mutate;
pub mod params;
pub mod presets;
pub mod telemetry;

pub use mutate::{mutate, Rng};
pub use params::{default_values, Curve, Group, ParamInfo, Unit, NUM_PARAMS, P, PARAMS};
pub use presets::{Preset, CATEGORIES, PRESETS};
