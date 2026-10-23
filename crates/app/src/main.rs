//! OXIDE: a very strange synthesizer. Dioxus UI.

mod audio;
mod keyboard;
mod knob;
mod visuals;

use dioxus::prelude::*;
use oxide_core::{mutate, Group, Rng, Unit, CATEGORIES, NUM_PARAMS, P, PARAMS, PRESETS};
use std::collections::HashMap;

use keyboard::Keyboard;
use knob::{group_color, Choice, DragOverlay, Knob, LockIcon};

const CSS: Asset = asset!("/assets/style.css");
const FAVICON: Asset = asset!("/assets/favicon.svg");
const FIRST_PRESET: usize = 1;
const HISTORY_MAX: usize = 100;

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Drag {
    pub idx: usize,
    pub start_y: f64,
    pub start_norm: f32,
}

#[derive(Clone, Copy, PartialEq)]
struct Snapshot {
    values: [f32; NUM_PARAMS],
    preset: usize,
    modified: bool,
}

/// All UI state. Every field is a signal, so the struct is `Copy` and can be
/// moved freely into event handlers.
#[derive(Clone, Copy)]
pub struct Synth {
    pub values: Signal<[f32; NUM_PARAMS]>,
    pub locks: Signal<[bool; NUM_PARAMS]>,
    pub preset: Signal<usize>,
    pub modified: Signal<bool>,
    history: Signal<Vec<Snapshot>>,
    pub rnd_amount: Signal<f32>,
    pub drag: Signal<Option<Drag>>,
    pub started: Signal<bool>,
    pub error: Signal<Option<String>>,
    pub octave: Signal<i32>,
    pub held: Signal<Vec<u8>>,
    pub hover: Signal<Option<usize>>,
    rng: Signal<Rng>,
}

impl Synth {
    fn new() -> Self {
        let mut locks = [false; NUM_PARAMS];
        // Keep the monitoring level stable by default; shift-click to unlock.
        locks[P::Volume.idx()] = true;
        let seed = (js_sys::Math::random() * u32::MAX as f64) as u64 ^ (js_sys::Date::now() as u64);
        Synth {
            values: Signal::new(PRESETS[FIRST_PRESET].full_values()),
            locks: Signal::new(locks),
            preset: Signal::new(FIRST_PRESET),
            modified: Signal::new(false),
            history: Signal::new(Vec::new()),
            rnd_amount: Signal::new(0.35),
            drag: Signal::new(None),
            started: Signal::new(false),
            error: Signal::new(None),
            octave: Signal::new(0),
            held: Signal::new(Vec::new()),
            hover: Signal::new(None),
            rng: Signal::new(Rng::new(seed)),
        }
    }

    pub fn set_param(mut self, i: usize, v: f32) {
        let v = PARAMS[i].clamp(v);
        if self.values.peek()[i] == v {
            return;
        }
        self.values.write()[i] = v;
        if !*self.modified.peek() {
            self.modified.set(true);
        }
        audio::set_param(i, v);
    }

    fn snapshot(mut self) {
        let s = Snapshot { values: *self.values.peek(), preset: *self.preset.peek(), modified: *self.modified.peek() };
        let mut h = self.history.write();
        h.push(s);
        if h.len() > HISTORY_MAX {
            h.remove(0);
        }
    }

    fn apply(mut self, values: [f32; NUM_PARAMS], preset: usize, modified: bool) {
        self.values.set(values);
        self.preset.set(preset);
        self.modified.set(modified);
        audio::set_all(&values);
    }

    pub fn load_preset(self, idx: usize) {
        let idx = idx % PRESETS.len();
        self.snapshot();
        self.apply(PRESETS[idx].full_values(), idx, false);
    }

    pub fn step_preset(self, delta: i32) {
        let n = PRESETS.len() as i32;
        let cur = *self.preset.peek() as i32;
        self.load_preset(((cur + delta).rem_euclid(n)) as usize);
    }

    pub fn randomize(mut self) {
        self.snapshot();
        let mut v = *self.values.peek();
        let locks = *self.locks.peek();
        let amount = *self.rnd_amount.peek();
        mutate(&mut v, &locks, amount, &mut self.rng.write());
        let preset = *self.preset.peek();
        self.apply(v, preset, true);
    }

    pub fn undo(mut self) {
        let last = self.history.write().pop();
        if let Some(s) = last {
            self.apply(s.values, s.preset, s.modified);
        }
    }

    pub fn toggle_lock(mut self, i: usize) {
        self.locks.write()[i] ^= true;
    }

    pub fn set_group_lock(mut self, g: Group, on: bool) {
        let mut l = self.locks.write();
        for (i, p) in PARAMS.iter().enumerate() {
            if p.group == g {
                l[i] = on;
            }
        }
    }

    pub fn set_all_locks(mut self, on: bool) {
        self.locks.set([on; NUM_PARAMS]);
    }

    pub fn note_on(mut self, n: u8, vel: f32) {
        audio::note_on(n, vel);
        let mut h = self.held.write();
        if !h.contains(&n) {
            h.push(n);
        }
    }

    pub fn note_off(mut self, n: u8) {
        audio::note_off(n);
        self.held.write().retain(|&x| x != n);
    }

    pub fn panic(mut self) {
        audio::panic();
        self.held.write().clear();
    }

    pub fn shift_octave(mut self, d: i32) {
        let o = (*self.octave.peek() + d).clamp(-3, 3);
        self.octave.set(o);
    }
}

#[component]
fn App() -> Element {
    let synth = use_context_provider(Synth::new);
    use_keyboard_and_midi(synth);
    let drag = (synth.drag)();
    let started = (synth.started)();

    rsx! {
        document::Stylesheet { href: CSS }
        document::Title { "OXIDE · a very strange synth" }
        document::Link { rel: "icon", href: FAVICON }
        div { class: "app",
            TopBar {}
            VisualStrip {}
            main { class: "panels",
                for g in Group::ALL {
                    Panel { key: "{g.title()}", group: g }
                }
            }
            Keyboard {}
            Footer {}
            if let Some(d) = drag {
                DragOverlay { drag: d }
            }
            if !started {
                Ignite {}
            }
        }
    }
}

#[component]
fn Ignite() -> Element {
    let synth = use_context::<Synth>();
    let strange = PARAMS.iter().filter(|p| p.group.is_strange()).count();
    let factory = PRESETS.len() - 1;
    let error = (synth.error)();
    rsx! {
        div { class: "ignite",
            div { class: "ignite-inner",
                h1 { class: "ignite-title", "OXIDE" }
                p { class: "ignite-sub", "a very strange synthesizer" }
                p { class: "ignite-blurb",
                    "Friction-squealing metal bodies, Lorenz chaos, logistic-map thoughts, formant monsters and a Dattorro plate. {strange} strange parameters, {factory} factory presets."
                }
                button {
                    class: "ignite-btn",
                    onclick: move |_| {
                        let mut s = synth;
                        let initial = *synth.values.peek();
                        spawn(async move {
                            match audio::start(&initial).await {
                                Ok(()) => s.started.set(true),
                                Err(e) => s.error.set(Some(format!("{e:?}"))),
                            }
                        });
                    },
                    "IGNITE"
                }
                if let Some(e) = error {
                    p { class: "ignite-error", "Audio failed to start: {e}" }
                }
            }
        }
    }
}

#[component]
fn TopBar() -> Element {
    let synth = use_context::<Synth>();
    let cur = (synth.preset)();
    let modified = (synth.modified)();
    let amount = (synth.rnd_amount)();
    let n_locked = synth.locks.read().iter().filter(|&&l| l).count();
    let can_undo = !synth.history.read().is_empty();
    let factory = PRESETS.len() - 1;

    rsx! {
        header { class: "topbar",
            div { class: "logo",
                span { class: "logo-main", "OXIDE" }
                span { class: "logo-sub", "ferro-chaotic synthesizer" }
            }
            div { class: "presets",
                button { class: "btn", title: "Previous preset", onclick: move |_| synth.step_preset(-1), "◀" }
                div { class: "preset-display",
                    select {
                        onchange: move |e| {
                            if let Ok(i) = e.value().parse::<usize>() {
                                synth.load_preset(i);
                            }
                            release_focus();
                        },
                        for cat in CATEGORIES.iter() {
                            optgroup { key: "{cat}", label: "{cat}",
                                for (i, p) in PRESETS.iter().enumerate().filter(|(_, p)| p.category == *cat) {
                                    option { key: "{i}", value: "{i}", selected: i == cur, "{p.name}" }
                                }
                            }
                        }
                    }
                    div { class: "preset-meta",
                        span { class: "preset-cat", "{PRESETS[cur].category}" }
                        span { "{cur} / {factory}" }
                        if modified { span { class: "preset-mod", "● modified" } }
                    }
                }
                button { class: "btn", title: "Next preset", onclick: move |_| synth.step_preset(1), "▶" }
            }
            div { class: "rnd-box",
                button {
                    class: "rnd",
                    title: "Mutate every unlocked parameter",
                    onclick: move |_| synth.randomize(),
                    "RND"
                }
                div { class: "rnd-amount",
                    label { "MUTATION {amount * 100.0:.0}%" }
                    input {
                        r#type: "range",
                        min: "0.02",
                        max: "1",
                        step: "0.01",
                        value: "{amount}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f32>() {
                                let mut a = synth.rnd_amount;
                                a.set(v);
                            }
                        },
                        onchange: move |_| release_focus(),
                    }
                }
                button { class: "btn", disabled: !can_undo, title: "Undo preset change / RND", onclick: move |_| synth.undo(), "UNDO" }
                div { class: "lock-ctl",
                    LockIcon {}
                    span { "{n_locked}" }
                    button { class: "btn small", title: "Lock every parameter", onclick: move |_| synth.set_all_locks(true), "ALL" }
                    button { class: "btn small", title: "Unlock every parameter", onclick: move |_| synth.set_all_locks(false), "NONE" }
                }
            }
            button { class: "btn panic", title: "Silence all voices", onclick: move |_| synth.panic(), "PANIC" }
        }
    }
}

#[component]
fn VisualStrip() -> Element {
    use_effect(visuals::start);
    rsx! {
        section { class: "visuals",
            figure { class: "vis",
                canvas { id: "cv-chladni", width: "120", height: "120", class: "pixelated-soft" }
                figcaption { "CHLADNI PLATE" }
            }
            figure { class: "vis",
                canvas { id: "cv-lorenz", width: "420", height: "240" }
                figcaption { "LORENZ ATTRACTOR" }
            }
            figure { class: "vis",
                canvas { id: "cv-scope", width: "420", height: "240" }
                figcaption { "PHOSPHOR X/Y" }
            }
            figure { class: "vis",
                canvas { id: "cv-spectro", width: "420", height: "240" }
                figcaption { "SPECTROGRAPH" }
            }
        }
    }
}

#[component]
fn Panel(group: Group) -> Element {
    let synth = use_context::<Synth>();
    let values = synth.values.read();
    let locks = synth.locks.read();
    let params: Vec<P> = P::ALL.iter().copied().filter(|p| p.info().group == group).collect();
    let all_locked = params.iter().all(|p| locks[p.idx()]);
    let class = if group.is_strange() { "panel strange" } else { "panel" };
    rsx! {
        section { class: "{class}", style: "--c: {group_color(group)}",
            h2 {
                title: "Shift-click to lock / unlock the whole section",
                onpointerdown: move |e| {
                    if e.data().modifiers().contains(Modifiers::SHIFT) {
                        e.prevent_default();
                        synth.set_group_lock(group, !all_locked);
                    }
                },
                "{group.title()}"
                if all_locked { LockIcon {} }
            }
            div { class: "knobs",
                for p in params.iter() {
                    if matches!(p.info().unit, Unit::Choice(_)) {
                        Choice { key: "{p.idx()}", idx: p.idx(), value: values[p.idx()], locked: locks[p.idx()] }
                    } else {
                        Knob { key: "{p.idx()}", idx: p.idx(), value: values[p.idx()], locked: locks[p.idx()] }
                    }
                }
            }
        }
    }
}

#[component]
fn Footer() -> Element {
    let synth = use_context::<Synth>();
    let hover = (synth.hover)();
    rsx! {
        footer { class: "footer",
            match hover {
                Some(i) => {
                    let p = &PARAMS[i];
                    let v = synth.values.read()[i];
                    let locked = synth.locks.read()[i];
                    rsx! {
                        span { class: "f-group", style: "color: {group_color(p.group)}", "{p.group.title()}" }
                        span { class: "f-name", "{p.name}" }
                        span { class: "f-val", "{p.format(v)}" }
                        span { class: "f-help", "{p.help}" }
                        if locked { span { class: "f-locked", "LOCKED" } }
                    }
                }
                None => rsx! {
                    span { class: "f-help",
                        "Drag or scroll knobs · Alt-drag: fine · Double-click: reset · Shift-click: lock against RND · Keys A–' play, Z/X octave · MIDI input supported"
                    }
                },
            }
        }
    }
}

/// Hand keyboard focus back to the page so the computer keyboard keeps
/// playing notes after using a form control.
fn release_focus() {
    document::eval("document.activeElement && document.activeElement.blur()");
}

/// Global computer-keyboard and Web MIDI input via a small JS bridge.
fn use_keyboard_and_midi(synth: Synth) {
    use_future(move || async move {
        let mut ev = document::eval(
            r#"
            const skip = (e) => e.ctrlKey || e.metaKey || e.altKey || ["TEXTAREA", "SELECT"].includes(e.target.tagName) ||
                (e.target.tagName === "INPUT" && !["range", "button", "checkbox"].includes(e.target.type));
            window.addEventListener("keydown", (e) => {
                if (e.repeat || skip(e)) return;
                dioxus.send(["down", e.key]);
            });
            window.addEventListener("keyup", (e) => dioxus.send(["up", e.key]));
            window.addEventListener("blur", () => dioxus.send(["blur", ""]));
            if (navigator.requestMIDIAccess) {
                navigator.requestMIDIAccess().then((midi) => {
                    const hook = () => midi.inputs.forEach((input) => {
                        input.onmidimessage = (m) => {
                            const [s, d1, d2] = m.data;
                            const c = s & 0xf0;
                            if (c === 0x90 && d2 > 0) dioxus.send(["mon", String(d1), String(d2)]);
                            else if (c === 0x80 || c === 0x90) dioxus.send(["moff", String(d1)]);
                        };
                    });
                    hook();
                    midi.onstatechange = hook;
                }).catch(() => {});
            }
            await new Promise(() => {});
            "#,
        );
        let mut pressed: HashMap<String, u8> = HashMap::new();
        while let Ok(msg) = ev.recv::<Vec<String>>().await {
            let kind = msg.first().map(String::as_str).unwrap_or("");
            let arg = msg.get(1).cloned().unwrap_or_default();
            if !*synth.started.peek() {
                continue;
            }
            match kind {
                "down" => {
                    let key = arg.to_lowercase();
                    if key == "z" {
                        synth.shift_octave(-1);
                    } else if key == "x" {
                        synth.shift_octave(1);
                    } else if let Some(st) = keyboard::key_to_semitone(&key) {
                        let note = (keyboard::qwerty_base(*synth.octave.peek()) + st).clamp(0, 127) as u8;
                        pressed.insert(key, note);
                        synth.note_on(note, 0.85);
                    }
                }
                "up" => {
                    if let Some(note) = pressed.remove(&arg.to_lowercase()) {
                        synth.note_off(note);
                    }
                }
                "blur" => {
                    for (_, note) in pressed.drain() {
                        synth.note_off(note);
                    }
                }
                "mon" => {
                    let n: u8 = arg.parse().unwrap_or(60);
                    let v: f32 = msg.get(2).and_then(|v| v.parse().ok()).unwrap_or(100.0);
                    synth.note_on(n, v / 127.0);
                }
                "moff" => {
                    if let Ok(n) = arg.parse::<u8>() {
                        synth.note_off(n);
                    }
                }
                _ => {}
            }
        }
    });
}
