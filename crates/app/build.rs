//! Compiles `oxide-dsp` to a bare wasm module so the app can embed it and
//! hand it to the AudioWorklet. A separate target dir avoids deadlocking on
//! the outer build's lock.

use std::{env, path::PathBuf, process::Command};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest.parent().unwrap().parent().unwrap().to_path_buf();
    for p in ["crates/dsp/src", "crates/dsp/Cargo.toml", "crates/core/src", "crates/core/Cargo.toml", "Cargo.toml"] {
        println!("cargo:rerun-if-changed={}", root.join(p).display());
    }

    let target_dir = root.join("target").join("dsp-wasm");
    let mut cmd = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
    cmd.current_dir(&root)
        .args(["build", "-p", "oxide-dsp", "--lib", "--release", "--target", "wasm32-unknown-unknown", "--target-dir"])
        .arg(&target_dir);
    // Don't leak the outer build's configuration (dx sets its own flags).
    for (k, _) in env::vars() {
        if k.starts_with("CARGO_TARGET_")
            || k.starts_with("CARGO_PROFILE_")
            || k.starts_with("CARGO_BUILD_")
            || k.starts_with("CARGO_ENCODED_")
            || k.starts_with("RUSTC")
            || k == "RUSTFLAGS"
            || k == "RUSTDOCFLAGS"
            || k == "CARGO_MAKEFLAGS"
        {
            cmd.env_remove(k);
        }
    }
    let status = cmd.status().expect("failed to spawn cargo for oxide-dsp");
    assert!(status.success(), "building oxide-dsp for wasm32-unknown-unknown failed (is the target installed?)");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    std::fs::copy(target_dir.join("wasm32-unknown-unknown/release/oxide_dsp.wasm"), out.join("oxide_dsp.wasm"))
        .expect("copy oxide_dsp.wasm");
}
