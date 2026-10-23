//! Parameter widgets. Every widget supports shift-click to lock its
//! parameter against RND.

use dioxus::prelude::*;
use oxide_core::{Group, Unit, PARAMS};

use crate::{Drag, Synth};

pub fn group_color(g: Group) -> &'static str {
    match g {
        Group::Excite => "#ff7a3d",
        Group::Body => "#f2b134",
        Group::Throat => "#ff5266",
        Group::Mind => "#6be3ff",
        Group::Chaos => "#b98cff",
        Group::Motion => "#7dff9a",
        Group::Dirt => "#d49a6a",
        Group::Voice => "#c9d3de",
        Group::Filter => "#ffd166",
        Group::Eq => "#9ad1ff",
        Group::Reverb => "#a5b4ff",
        Group::Out => "#ffffff",
    }
}

const KNOB_HINT: &str = "Drag or scroll to adjust · Alt-drag for fine · Double-click to reset · Shift-click to lock";

/// Knob angle in degrees (0 = up) for a normalised value.
fn angle(n: f32) -> f32 {
    -135.0 + 270.0 * n
}

fn polar(cx: f32, cy: f32, r: f32, deg: f32) -> (f32, f32) {
    let a = deg.to_radians();
    (cx + r * a.sin(), cy - r * a.cos())
}

fn arc(cx: f32, cy: f32, r: f32, from: f32, to: f32) -> String {
    let (x0, y0) = polar(cx, cy, r, from);
    let (x1, y1) = polar(cx, cy, r, to.max(from + 0.01));
    let large = if to - from > 180.0 { 1 } else { 0 };
    format!("M {x0:.2} {y0:.2} A {r} {r} 0 {large} 1 {x1:.2} {y1:.2}")
}

/// Small padlock used in labels.
#[component]
pub fn LockIcon() -> Element {
    rsx! {
        svg { class: "lock-icon", view_box: "0 0 12 14",
            path { d: "M3 6.5 V4.5 a3 3 0 0 1 6 0 V6.5" }
            rect { x: "1.5", y: "6.5", width: "9", height: "6.5", rx: "1" }
        }
    }
}

const LOCK_PATH: &str = "M26 29 v-3 a4 4 0 0 1 8 0 v3 M24.5 29 h11 v8 h-11 z";

#[component]
pub fn Knob(idx: usize, value: f32, locked: bool) -> Element {
    let synth = use_context::<Synth>();
    let info = &PARAMS[idx];
    let norm = info.to_norm(value);
    let color = group_color(info.group);
    let bipolar = info.min < 0.0 && info.max > 0.0 && info.curve == oxide_core::Curve::Lin;
    let origin = if bipolar { info.to_norm(0.0) } else { 0.0 };
    let track = arc(30.0, 30.0, 23.0, angle(0.0), angle(1.0));
    let value_arc = arc(30.0, 30.0, 23.0, angle(origin.min(norm)), angle(origin.max(norm)));
    let (px, py) = polar(30.0, 30.0, 15.0, angle(norm));
    let (qx, qy) = polar(30.0, 30.0, 6.0, angle(norm));
    let class = if locked { "knob locked" } else { "knob" };
    let step = if info.is_discrete() { 1.0 / (info.max - info.min) } else { 0.01 };
    // Double-click is detected here: the drag overlay that appears on the
    // first press would otherwise swallow the browser's dblclick event.
    let mut last_press = use_signal(|| 0.0f64);

    rsx! {
        div {
            class: "{class}",
            style: "--c: {color}",
            title: "{info.name}: {info.help}\n{KNOB_HINT}",
            onpointerdown: move |e| {
                e.prevent_default();
                let d = e.data();
                let now = js_sys::Date::now();
                let double = now - *last_press.peek() < 350.0;
                last_press.set(now);
                if d.modifiers().contains(Modifiers::SHIFT) {
                    synth.toggle_lock(idx);
                } else if double {
                    synth.set_param(idx, info.default);
                } else {
                    let mut drag = synth.drag;
                    drag.set(Some(Drag { idx, start_y: d.client_coordinates().y, start_norm: norm }));
                }
            },
            onwheel: move |e| {
                e.prevent_default();
                let dy = e.data().delta().strip_units().y;
                if dy != 0.0 {
                    synth.set_param(idx, info.from_norm(norm - dy.signum() as f32 * step));
                }
            },
            onmouseenter: move |_| {
                let mut h = synth.hover;
                h.set(Some(idx));
            },
            svg {
                view_box: "0 0 60 60",
                circle { class: "knob-body", cx: "30", cy: "30", r: "18" }
                path { class: "knob-track", d: "{track}" }
                path { class: "knob-arc", d: "{value_arc}" }
                line { class: "knob-pointer", x1: "{qx}", y1: "{qy}", x2: "{px}", y2: "{py}" }
                if locked {
                    path { class: "knob-lock", d: LOCK_PATH }
                }
            }
            div { class: "knob-name", "{info.name}" }
            div { class: "knob-value", "{info.format(value)}" }
        }
    }
}

/// Segmented switch for choice parameters (the filter type).
#[component]
pub fn Choice(idx: usize, value: f32, locked: bool) -> Element {
    let synth = use_context::<Synth>();
    let info = &PARAMS[idx];
    let Unit::Choice(labels) = info.unit else { return rsx! {} };
    let cur = value.round() as usize;
    let class = if locked { "choice locked" } else { "choice" };
    rsx! {
        div {
            class: "{class}",
            style: "--c: {group_color(info.group)}",
            title: "{info.name}: {info.help}\nClick to select · Shift-click to lock",
            onmouseenter: move |_| {
                let mut h = synth.hover;
                h.set(Some(idx));
            },
            div { class: "choice-name",
                "{info.name}"
                if locked { LockIcon {} }
            }
            div { class: "choice-buttons",
                for (i, label) in labels.iter().enumerate() {
                    button {
                        key: "{i}",
                        class: if i == cur { "on" } else { "" },
                        onpointerdown: move |e| {
                            e.prevent_default();
                            if e.data().modifiers().contains(Modifiers::SHIFT) {
                                synth.toggle_lock(idx);
                            } else {
                                synth.set_param(idx, i as f32);
                            }
                        },
                        "{label}"
                    }
                }
            }
        }
    }
}

/// Full-screen layer that captures the pointer while a knob is dragged.
#[component]
pub fn DragOverlay(drag: Drag) -> Element {
    let synth = use_context::<Synth>();
    let end = move || {
        let mut d = synth.drag;
        d.set(None);
    };
    rsx! {
        div {
            class: "drag-overlay",
            onpointermove: move |e| {
                let d = e.data();
                let y = d.client_coordinates().y;
                let fine = d.modifiers().contains(Modifiers::ALT);
                let span = if fine { 1200.0 } else { 220.0 };
                let n = drag.start_norm + ((drag.start_y - y) / span) as f32;
                synth.set_param(drag.idx, PARAMS[drag.idx].from_norm(n));
            },
            onpointerup: move |_| end(),
            onpointerleave: move |_| end(),
        }
    }
}
