//! Canvas visualisers, driven by a requestAnimationFrame loop:
//!
//! * Chladni plate: sand settles on the nodal lines of the modes that are
//!   currently ringing in the metal body (energies come from the DSP).
//! * Lorenz attractor: the engine's own chaotic state, rotating in 3D.
//! * Phosphor XY scope (mid/side) with persistence and a readout.
//! * Scrolling log-frequency spectrograph.

use oxide_core::telemetry as tm;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::f64::consts::{FRAC_1_SQRT_2, PI};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d as Ctx, HtmlCanvasElement, ImageData};

use crate::audio;

const CH: usize = 120; // Chladni resolution
const MAXN: usize = 12;
const TRAIL_MAX: usize = 1100;

struct Canvas {
    el: HtmlCanvasElement,
    ctx: Ctx,
    w: f64,
    h: f64,
}

fn canvas(id: &str) -> Option<Canvas> {
    let el: HtmlCanvasElement = web_sys::window()?.document()?.get_element_by_id(id)?.dyn_into().ok()?;
    let ctx: Ctx = el.get_context("2d").ok()??.dyn_into().ok()?;
    Some(Canvas { w: el.width() as f64, h: el.height() as f64, el, ctx })
}

fn lerp_rgb(stops: &[(f32, [f32; 3])], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    for w in stops.windows(2) {
        let (a, ca) = w[0];
        let (b, cb) = w[1];
        if t <= b {
            let u = (t - a) / (b - a);
            return [0, 1, 2].map(|i| (ca[i] + (cb[i] - ca[i]) * u) as u8);
        }
    }
    stops.last().unwrap().1.map(|x| x as u8)
}

struct Vis {
    chladni: Canvas,
    lorenz: Canvas,
    scope: Canvas,
    spectro: Canvas,
    sand: Vec<f32>,
    field: Vec<f32>,
    phase: [f32; 16],
    pixels: Vec<u8>,
    cos_tab: Vec<f32>,
    plate: Vec<[f32; 3]>,
    trail: VecDeque<[f32; 3]>,
    last_count: u32,
    idle: [f64; 3],
    rot: f64,
    tbuf_l: Vec<f32>,
    tbuf_r: Vec<f32>,
    fbuf: Vec<u8>,
    col: Vec<u8>,
    heat: [[u8; 3]; 256],
    frame: u64,
}

impl Vis {
    fn new() -> Option<Self> {
        let mut cos_tab = vec![0.0; (MAXN + 1) * CH];
        for n in 0..=MAXN {
            for x in 0..CH {
                cos_tab[n * CH + x] = (n as f64 * PI * (x as f64 + 0.5) / CH as f64).cos() as f32;
            }
        }
        // Brushed, slightly corroded plate.
        let mut plate = vec![[0.0; 3]; CH * CH];
        let mut seed = 0x1234_5678u32;
        for y in 0..CH {
            let streak = ((y as f32 * 0.7).sin() * 0.5 + 0.5) * 6.0;
            for x in 0..CH {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                let n = (seed >> 24) as f32 / 255.0;
                let dx = x as f32 / CH as f32 - 0.5;
                let dy = y as f32 / CH as f32 - 0.5;
                let v = 1.0 - (dx * dx + dy * dy) * 1.2;
                let base = (18.0 + streak + n * 5.0) * v;
                plate[y * CH + x] = [base * 1.05, base * 1.1, base * 1.25];
            }
        }
        let stops: [(f32, [f32; 3]); 6] = [
            (0.0, [4.0, 5.0, 10.0]),
            (0.2, [40.0, 12.0, 60.0]),
            (0.45, [150.0, 30.0, 70.0]),
            (0.65, [235.0, 90.0, 40.0]),
            (0.85, [255.0, 190.0, 80.0]),
            (1.0, [255.0, 250.0, 210.0]),
        ];
        let heat = std::array::from_fn(|i| lerp_rgb(&stops, i as f32 / 255.0));
        let spectro = canvas("cv-spectro")?;
        let col = vec![0u8; 2 * spectro.h as usize * 4];
        Some(Vis {
            chladni: canvas("cv-chladni")?,
            lorenz: canvas("cv-lorenz")?,
            scope: canvas("cv-scope")?,
            spectro,
            sand: vec![0.0; CH * CH],
            field: vec![0.0; CH * CH],
            phase: [0.0; 16],
            pixels: vec![0; CH * CH * 4],
            cos_tab,
            plate,
            trail: VecDeque::with_capacity(TRAIL_MAX),
            last_count: 0,
            idle: [1.0, 1.0, 20.0],
            rot: 0.0,
            tbuf_l: vec![0.0; 2048],
            tbuf_r: vec![0.0; 2048],
            fbuf: vec![0; 1024],
            col,
            heat,
            frame: 0,
        })
    }

    fn frame(&mut self) {
        self.frame += 1;
        let audio = audio::get();
        let tele = audio.as_ref().map(|a| a.telemetry.borrow().clone());
        self.draw_chladni(tele.as_deref());
        self.draw_lorenz(tele.as_deref());
        self.draw_scope(audio.as_deref(), tele.as_deref());
        self.draw_spectro(audio.as_deref());
    }

    fn draw_chladni(&mut self, tele: Option<&[f32]>) {
        // Each body mode becomes a plate mode (n, m) whose frequency ratio
        // (n² + m²)/5 is closest to the mode's ratio. The plate shape is
        // their superposition with slowly drifting phases; sand gathers on
        // the nodal lines.
        let mut modes: Vec<(usize, usize, f32, f32)> = Vec::new();
        if let Some(t) = tele {
            for k in 0..tm::NMODES {
                let a = t[tm::MODE_AMP + k];
                if a < 1e-5 {
                    continue;
                }
                let r = t[tm::MODE_RATIO + k].max(1.0);
                let mut best = (1, 2, f32::MAX);
                for n in 1..=MAXN {
                    for m in (n + 1)..=MAXN {
                        let d = (((n * n + m * m) as f32 / 5.0) / r).ln().abs();
                        if d < best.2 {
                            best = (n, m, d);
                        }
                    }
                }
                let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
                modes.push((best.0, best.1, a, sign));
            }
            // No metal body ringing (voices, glass, monsters...): let the
            // chaos state and loudness shape a plate instead.
            let level = t[tm::RMS_L] + t[tm::RMS_R];
            if modes.is_empty() && level > 1e-3 {
                let (x, y, z) = (t[tm::LORENZ], t[tm::LORENZ + 1], t[tm::LORENZ + 2]);
                let f0 = t[tm::F0].max(20.0);
                let o = ((f0 / 55.0).log2().clamp(0.0, 6.0)) as usize;
                for (i, w) in [x, y, z].iter().enumerate() {
                    let n = 1 + (o + i) % 5;
                    modes.push((n, n + 1 + i, level * (0.4 + 0.6 * w.abs()), if i % 2 == 0 { 1.0 } else { -1.0 }));
                }
            }
        }
        let live = !modes.is_empty();
        for (k, ph) in self.phase.iter_mut().enumerate() {
            *ph += 0.011 * (1.0 + k as f32 * 0.37);
        }
        let weights: Vec<f32> =
            modes.iter().enumerate().map(|(k, m)| m.2 * (0.6 + 0.4 * self.phase[k % 16].cos())).collect();
        let ct = &self.cos_tab;
        if live {
            let mut ms = 0.0;
            for y in 0..CH {
                for x in 0..CH {
                    let mut v = 0.0;
                    for (&(n, m, _, s), &w) in modes.iter().zip(&weights) {
                        v += w * (ct[n * CH + x] * ct[m * CH + y] + s * ct[m * CH + x] * ct[n * CH + y]);
                    }
                    self.field[y * CH + x] = v;
                    ms += v * v;
                }
            }
            let inv = 1.0 / (ms / (CH * CH) as f32).max(1e-12);
            for i in 0..CH * CH {
                let target = (-self.field[i] * self.field[i] * inv * 9.0).exp();
                self.sand[i] += (target - self.sand[i]) * 0.2;
            }
        } else {
            for v in &mut self.sand {
                *v *= 0.985;
            }
        }
        for i in 0..CH * CH {
            let s = self.sand[i];
            let p = self.plate[i];
            let px = &mut self.pixels[i * 4..i * 4 + 4];
            px[0] = (p[0] + s * 235.0).min(255.0) as u8;
            px[1] = (p[1] + s * 150.0).min(255.0) as u8;
            px[2] = (p[2] + s * 70.0).min(255.0) as u8;
            px[3] = 255;
        }
        if let Ok(img) = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&self.pixels), CH as u32, CH as u32) {
            let _ = self.chladni.ctx.put_image_data(&img, 0.0, 0.0);
        }
    }

    fn draw_lorenz(&mut self, tele: Option<&[f32]>) {
        match tele {
            Some(t) => {
                let count = t[tm::TRAIL_COUNT] as u32;
                let new = count.wrapping_sub(self.last_count).min(tm::TRAIL_LEN as u32);
                for j in (0..new).rev() {
                    let idx = (count.wrapping_sub(1 + j) as usize) % tm::TRAIL_LEN;
                    let o = tm::TRAIL + idx * 3;
                    self.push_trail([t[o], t[o + 1], t[o + 2]]);
                }
                self.last_count = count;
            }
            None => {
                // Idle: integrate our own attractor so the screen is alive.
                for _ in 0..4 {
                    let [x, y, z] = self.idle;
                    let h = 0.004;
                    self.idle = [
                        x + 10.0 * (y - x) * h,
                        y + (x * (28.0 - z) - y) * h,
                        z + (x * y - 8.0 / 3.0 * z) * h,
                    ];
                    let [x, y, z] = self.idle;
                    self.push_trail([(x * 0.05) as f32, (y * 0.037) as f32, ((z - 24.0) * 0.05) as f32]);
                }
            }
        }
        let c = &self.lorenz;
        let ctx = &c.ctx;
        ctx.set_fill_style_str("rgba(6,8,12,1)");
        ctx.fill_rect(0.0, 0.0, c.w, c.h);
        self.rot += 0.004;
        let (s, co) = self.rot.sin_cos();
        let scale = c.h * 0.42;
        let (cx, cy) = (c.w * 0.5, c.h * 0.5);
        let proj = |p: &[f32; 3]| {
            let (x, y, z) = (p[0] as f64, p[1] as f64, p[2] as f64);
            let rx = x * co - y * s;
            let depth = x * s + y * co;
            (cx + rx * scale, cy - z * scale * 0.95 + depth * scale * 0.18, depth)
        };
        let n = self.trail.len();
        if n > 2 {
            ctx.set_line_width(1.2);
            let chunk = 40;
            let mut i = 0;
            while i + 1 < n {
                let end = (i + chunk + 1).min(n);
                let age = i as f64 / n as f64;
                let hue = 20.0 + 170.0 * age;
                ctx.set_stroke_style_str(&format!("hsla({hue:.0},90%,{:.0}%,{:.2})", 40.0 + age * 25.0, 0.08 + age * 0.85));
                ctx.begin_path();
                let (x0, y0, _) = proj(&self.trail[i]);
                ctx.move_to(x0, y0);
                for p in self.trail.range(i + 1..end) {
                    let (x, y, _) = proj(p);
                    ctx.line_to(x, y);
                }
                ctx.stroke();
                i = end - 1;
            }
            let (hx, hy, _) = proj(self.trail.back().unwrap());
            ctx.set_shadow_blur(14.0);
            ctx.set_shadow_color("#7ff");
            ctx.set_fill_style_str("#dff");
            ctx.begin_path();
            let _ = ctx.arc(hx, hy, 3.0, 0.0, 2.0 * PI);
            ctx.fill();
            ctx.set_shadow_blur(0.0);
        }
    }

    fn push_trail(&mut self, p: [f32; 3]) {
        if let Some(q) = self.trail.back() {
            let d = (p[0] - q[0]).abs() + (p[1] - q[1]).abs() + (p[2] - q[2]).abs();
            if d < 0.012 {
                return;
            }
        }
        if self.trail.len() >= TRAIL_MAX {
            self.trail.pop_front();
        }
        self.trail.push_back(p);
    }

    fn draw_scope(&mut self, audio: Option<&audio::Audio>, tele: Option<&[f32]>) {
        let c = &self.scope;
        let ctx = &c.ctx;
        ctx.set_fill_style_str("rgba(3,8,6,0.28)");
        ctx.fill_rect(0.0, 0.0, c.w, c.h);
        let (cx, cy) = (c.w * 0.5, c.h * 0.5);
        // Graticule.
        ctx.set_stroke_style_str("rgba(90,255,170,0.06)");
        ctx.set_line_width(1.0);
        ctx.begin_path();
        for i in 1..8 {
            let x = c.w * i as f64 / 8.0;
            ctx.move_to(x, 0.0);
            ctx.line_to(x, c.h);
        }
        for i in 1..6 {
            let y = c.h * i as f64 / 6.0;
            ctx.move_to(0.0, y);
            ctx.line_to(c.w, y);
        }
        ctx.stroke();

        let Some(a) = audio else { return };
        a.analyser_l.get_float_time_domain_data(&mut self.tbuf_l);
        a.analyser_r.get_float_time_domain_data(&mut self.tbuf_r);
        let s = c.h * 0.62;
        ctx.set_line_width(1.3);
        ctx.set_stroke_style_str("rgba(120,255,190,0.85)");
        ctx.set_shadow_blur(10.0);
        ctx.set_shadow_color("rgba(60,255,160,0.9)");
        ctx.begin_path();
        for i in 0..self.tbuf_l.len() {
            let (l, r) = (self.tbuf_l[i] as f64, self.tbuf_r[i] as f64);
            let x = cx + (l - r) * s * FRAC_1_SQRT_2;
            let y = cy - (l + r) * s * FRAC_1_SQRT_2;
            if i == 0 { ctx.move_to(x, y) } else { ctx.line_to(x, y) }
        }
        ctx.stroke();
        ctx.set_shadow_blur(0.0);

        if let Some(t) = tele {
            let peak = t[tm::PEAK] as f64;
            ctx.set_fill_style_str(if peak > 0.95 { "#ff4040" } else { "rgba(120,255,190,0.8)" });
            ctx.fill_rect(6.0, c.h - 8.0, (c.w - 12.0) * peak.min(1.0), 3.0);
            ctx.set_font("10px ui-monospace, monospace");
            ctx.set_fill_style_str("rgba(150,255,200,0.75)");
            let _ = ctx.fill_text(&format!("VOICES {}", t[tm::VOICES] as u32), 6.0, 13.0);
            let _ = ctx.fill_text(&format!("F0 {:>7.1} Hz", t[tm::F0]), 6.0, 25.0);
            let _ = ctx.fill_text(&format!("x(n) {:.3}", t[tm::LOGISTIC]), c.w - 70.0, 13.0);
        }
    }

    fn draw_spectro(&mut self, audio: Option<&audio::Audio>) {
        let Some(a) = audio else { return };
        let c = &self.spectro;
        let _ = c.ctx.draw_image_with_html_canvas_element(&c.el, -2.0, 0.0);
        a.analyser_l.get_byte_frequency_data(&mut self.fbuf);
        let nyq = a.ctx.sample_rate() as f64 / 2.0;
        let bins = self.fbuf.len() as f64;
        let h = c.h as usize;
        for y in 0..h {
            let f = 30.0 * (18000.0f64 / 30.0).powf(1.0 - y as f64 / h as f64);
            let b = ((f / nyq) * bins) as usize;
            let v = self.fbuf[b.min(self.fbuf.len() - 1)] as usize;
            let rgb = self.heat[((v as f32 / 255.0).powf(1.8) * 255.0) as usize];
            for x in 0..2 {
                let o = (y * 2 + x) * 4;
                self.col[o..o + 3].copy_from_slice(&rgb);
                self.col[o + 3] = 255;
            }
        }
        if let Ok(img) = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&self.col), 2, h as u32) {
            let _ = c.ctx.put_image_data(&img, c.w - 2.0, 0.0);
        }
    }
}

thread_local! {
    static STARTED: Cell<bool> = const { Cell::new(false) };
}

fn request_frame(f: &Closure<dyn FnMut()>) {
    if let Some(w) = web_sys::window() {
        let _ = w.request_animation_frame(f.as_ref().unchecked_ref());
    }
}

/// Start the animation loop once; canvases must already be in the DOM.
pub fn start() {
    if STARTED.with(|s| s.replace(true)) {
        return;
    }
    let vis: Rc<RefCell<Option<Vis>>> = Rc::new(RefCell::new(None));
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        {
            let mut v = vis.borrow_mut();
            if v.is_none() {
                *v = Vis::new();
            }
            if let Some(v) = v.as_mut() {
                v.frame();
            }
        }
        if let Some(cb) = f.borrow().as_ref() {
            request_frame(cb);
        }
    }));
    if let Some(cb) = g.borrow().as_ref() {
        request_frame(cb);
    }
}
