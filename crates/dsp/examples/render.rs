//! Offline renderer: plays a short phrase through each preset, writes WAVs
//! to `renders/` and prints level statistics.
//!
//! ```text
//! cargo run -p oxide-dsp --release --example render                  # all presets
//! cargo run -p oxide-dsp --release --example render -- "T-Rex"       # name filter
//! cargo run -p oxide-dsp --release --example render -- Init bow=0.8 press=0.9
//! ```

use oxide_core::{Preset, P, PRESETS};
use oxide_dsp::{Engine, BLOCK};
use std::io::Write;

const SR: f32 = 48000.0;
const TARGET_ST: f32 = -16.0;

pub struct Stats {
    pub st_db: f32,
    pub rms_db: f32,
    pub peak: f32,
    pub tail_db: f32,
    pub centroid: f32,
    pub realtime: f32,
}

fn render(values: &[f32], seconds: f32) -> (Vec<f32>, Vec<f32>, f32) {
    let mut e = Engine::new(SR);
    e.set_all(values);
    // Pick a register that suits the patch: low for octave-shifted creatures.
    let base: u8 = 48;
    let events: Vec<(f32, bool, u8)> = vec![
        (0.05, true, base),
        (1.6, false, base),
        (2.0, true, base + 12),
        (2.0, true, base + 19),
        (3.6, false, base + 12),
        (3.6, false, base + 19),
    ];
    let total = (seconds * SR) as usize;
    let mut l = Vec::with_capacity(total);
    let mut r = Vec::with_capacity(total);
    let mut ev = 0;
    let t0 = std::time::Instant::now();
    while l.len() < total {
        let now = l.len() as f32 / SR;
        while ev < events.len() && events[ev].0 <= now {
            let (_, on, n) = events[ev];
            if on { e.note_on(n, 0.85) } else { e.note_off(n) }
            ev += 1;
        }
        e.process(BLOCK);
        l.extend_from_slice(&e.out[..BLOCK]);
        r.extend_from_slice(&e.out[BLOCK..]);
    }
    let rt = seconds / t0.elapsed().as_secs_f32();
    (l, r, rt)
}

fn stats(l: &[f32], r: &[f32], rt: f32) -> Stats {
    let n = l.len();
    let mut sum = 0.0f64;
    let mut peak = 0.0f32;
    for i in 0..n {
        sum += (l[i] as f64).powi(2) + (r[i] as f64).powi(2);
        peak = peak.max(l[i].abs()).max(r[i].abs());
        assert!(l[i].is_finite() && r[i].is_finite(), "non-finite sample");
    }
    let rms = (sum / (2 * n) as f64).sqrt() as f32;
    let tail = &l[n - n / 10..];
    let tail_rms = (tail.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / tail.len() as f64).sqrt() as f32;
    // Zero-crossing based brightness proxy (Hz).
    let zc = l.windows(2).filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0)).count() as f32;
    // Short-term loudness: max RMS over 400 ms windows.
    let win = (0.4 * SR) as usize;
    let mut st = 0.0f32;
    for c in (0..n.saturating_sub(win)).step_by(win / 4) {
        let e: f64 = (c..c + win).map(|i| (l[i] as f64).powi(2) + (r[i] as f64).powi(2)).sum();
        st = st.max((e / (2 * win) as f64).sqrt() as f32);
    }
    Stats {
        st_db: 20.0 * st.max(1e-9).log10(),
        rms_db: 20.0 * rms.max(1e-9).log10(),
        peak,
        tail_db: 20.0 * tail_rms.max(1e-9).log10(),
        centroid: zc / 2.0 / (n as f32 / SR),
        realtime: rt,
    }
}

fn write_wav(path: &str, l: &[f32], r: &[f32]) -> std::io::Result<()> {
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    let n = l.len() as u32;
    let data_len = n * 4;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_len).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&2u16.to_le_bytes())?;
    f.write_all(&(SR as u32).to_le_bytes())?;
    f.write_all(&(SR as u32 * 4).to_le_bytes())?;
    f.write_all(&4u16.to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&data_len.to_le_bytes())?;
    for i in 0..l.len() {
        for x in [l[i], r[i]] {
            f.write_all(&((x.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes())?;
        }
    }
    Ok(())
}

fn slug(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let filter = args.iter().find(|a| !a.contains('=')).cloned();
    let overrides: Vec<(P, f32)> = args
        .iter()
        .filter_map(|a| a.split_once('='))
        .map(|(k, v)| (P::from_key(k).unwrap_or_else(|| panic!("unknown param {k}")), v.parse().unwrap()))
        .collect();
    let calibrate = args.iter().any(|a| a == "--calibrate");
    let filter = filter.filter(|f| f != "--calibrate");
    std::fs::create_dir_all("renders").unwrap();
    let presets: Vec<&Preset> = PRESETS
        .iter()
        .filter(|p| filter.as_ref().is_none_or(|f| p.name.to_lowercase().contains(&f.to_lowercase())))
        .collect();
    if !calibrate { println!("{:<52} {:>7} {:>8} {:>6} {:>8} {:>7} {:>6}", "preset", "st dB", "rms dB", "peak", "tail dB", "zc Hz", "xRT"); }
    for p in presets {
        let mut v = p.full_values();
        for &(k, x) in &overrides {
            v[k.idx()] = x;
        }
        let (l, r, rt) = render(&v, 6.0);
        let s = stats(&l, &r, rt);
        write_wav(&format!("renders/{}.wav", slug(p.name)), &l, &r).unwrap();
        if calibrate {
            // Suggested volume so the short-term loudness lands on TARGET_ST.
            let vol = v[P::Volume.idx()];
            println!("{}|{:.1}", p.name, (vol + TARGET_ST - s.st_db).clamp(-40.0, 6.0));
            continue;
        }
        let flag = if s.st_db < TARGET_ST - 4.0 {
            "  QUIET"
        } else if s.st_db > TARGET_ST + 4.0 || s.peak > 0.97 {
            "  HOT"
        } else {
            ""
        };
        println!(
            "{:<52} {:>7.1} {:>8.1} {:>6.2} {:>8.1} {:>7.0} {:>6.0}{}",
            format!("[{}] {}", p.category, p.name),
            s.st_db,
            s.rms_db,
            s.peak,
            s.tail_db,
            s.centroid,
            s.realtime,
            flag
        );
    }
}
