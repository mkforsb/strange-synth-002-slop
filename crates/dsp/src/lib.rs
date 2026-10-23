//! OXIDE DSP engine.
//!
//! Built natively for tests and offline rendering, and as a bare
//! `wasm32-unknown-unknown` module (no wasm-bindgen) that the AudioWorklet
//! instantiates directly through the C ABI below.

pub mod chaos;
pub mod engine;
pub mod filters;
pub mod modal;
pub mod reverb;
pub mod util;
pub mod voice;

pub use engine::{Engine, BLOCK, MAX_VOICES};

#[cfg(target_arch = "wasm32")]
mod ffi {
    use super::Engine;
    use core::cell::UnsafeCell;

    struct Global(UnsafeCell<Option<Engine>>);
    // The AudioWorklet runs on a single thread.
    unsafe impl Sync for Global {}
    static ENGINE: Global = Global(UnsafeCell::new(None));

    fn engine() -> Option<&'static mut Engine> {
        unsafe { (*ENGINE.0.get()).as_mut() }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn oxide_init(sr: f32) {
        unsafe { *ENGINE.0.get() = Some(Engine::new(sr)) }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn oxide_param_count() -> u32 {
        oxide_core::NUM_PARAMS as u32
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn oxide_set_param(i: u32, v: f32) {
        if let Some(e) = engine() {
            e.set_param(i as usize, v);
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn oxide_note_on(note: u32, vel: f32) {
        if let Some(e) = engine() {
            e.note_on(note.min(127) as u8, vel);
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn oxide_note_off(note: u32) {
        if let Some(e) = engine() {
            e.note_off(note.min(127) as u8);
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn oxide_all_notes_off() {
        if let Some(e) = engine() {
            e.all_notes_off();
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn oxide_panic() {
        if let Some(e) = engine() {
            e.panic();
        }
    }

    /// Render `n` frames; returns a pointer to `[L; 128][R; 128]`.
    #[unsafe(no_mangle)]
    pub extern "C" fn oxide_process(n: u32) -> *const f32 {
        match engine() {
            Some(e) => {
                e.process(n as usize);
                e.out.as_ptr()
            }
            None => core::ptr::null(),
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn oxide_telemetry() -> *const f32 {
        engine().map(|e| e.telemetry.as_ptr()).unwrap_or(core::ptr::null())
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn oxide_telemetry_len() -> u32 {
        oxide_core::telemetry::LEN as u32
    }
}
