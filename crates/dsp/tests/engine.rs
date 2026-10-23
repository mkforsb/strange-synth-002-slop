use oxide_core::{mutate, Rng, NUM_PARAMS, P, PARAMS, PRESETS};
use oxide_dsp::{Engine, BLOCK};

const SR: f32 = 48000.0;

fn run(e: &mut Engine, seconds: f32) -> (f32, f32) {
    let mut peak = 0.0f32;
    let mut energy = 0.0f64;
    let blocks = (seconds * SR / BLOCK as f32) as usize;
    for _ in 0..blocks {
        e.process(BLOCK);
        for &x in &e.out {
            assert!(x.is_finite(), "non-finite output");
            peak = peak.max(x.abs());
            energy += (x as f64) * (x as f64);
        }
    }
    (peak, (energy / (blocks * 2 * BLOCK) as f64).sqrt() as f32)
}

#[test]
fn every_preset_makes_sound_and_stays_bounded() {
    for p in PRESETS {
        let mut e = Engine::new(SR);
        e.set_all(&p.full_values());
        e.note_on(48, 0.9);
        e.note_on(55, 0.9);
        let (peak, rms) = run(&mut e, 1.5);
        assert!(rms > 1e-3, "{} is silent (rms {rms})", p.name);
        assert!(peak <= 1.0, "{} peak {peak}", p.name);
    }
}

#[test]
fn voices_are_freed_after_release() {
    let mut e = Engine::new(SR);
    e.set_param(P::Release.idx(), 0.1);
    for n in 40..52 {
        e.note_on(n, 1.0);
    }
    run(&mut e, 0.2);
    assert_eq!(e.active_voices(), oxide_dsp::MAX_VOICES);
    for n in 40..52 {
        e.note_off(n);
    }
    run(&mut e, 1.5);
    assert_eq!(e.active_voices(), 0);
}

#[test]
fn random_mutations_never_blow_up() {
    let mut rng = Rng::new(2024);
    let mut v = oxide_core::default_values();
    for round in 0..25 {
        mutate(&mut v, &[false; NUM_PARAMS], 1.0, &mut rng);
        let mut e = Engine::new(44100.0);
        e.set_all(&v);
        e.note_on(36 + round as u8 * 2, 1.0);
        e.note_on(70, 0.5);
        let (peak, _) = run(&mut e, 0.5);
        assert!(peak <= 1.0, "round {round}");
    }
}

#[test]
fn extreme_params_are_stable() {
    // Everything at max, then everything at min.
    for pick in [|p: &oxide_core::ParamInfo| p.max, |p: &oxide_core::ParamInfo| p.min] {
        let vals: Vec<f32> = PARAMS.iter().map(pick).collect();
        let mut e = Engine::new(48000.0);
        e.set_all(&vals);
        for n in [0u8, 24, 60, 96, 127] {
            e.note_on(n, 1.0);
        }
        run(&mut e, 1.0);
    }
}
