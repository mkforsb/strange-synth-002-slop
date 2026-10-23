//! On-screen keyboard plus the computer-keyboard mapping.

use dioxus::prelude::*;

use crate::Synth;

const OCTAVES: i32 = 3;
const BLACK: [bool; 12] = [false, true, false, true, false, false, true, false, true, false, true, false];
const NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

/// Computer keyboard, tracker style: A W S E D F T G Y H U J K O L P ; '
pub fn key_to_semitone(key: &str) -> Option<i32> {
    let k = key.to_lowercase();
    let i = ["a", "w", "s", "e", "d", "f", "t", "g", "y", "h", "u", "j", "k", "o", "l", "p", ";", "'"]
        .iter()
        .position(|&c| c == k)?;
    Some(i as i32)
}

/// Lowest note shown on screen for a given octave shift.
pub fn screen_base(octave: i32) -> i32 {
    36 + octave * 12
}

/// Note played by the computer keyboard's "A" key.
pub fn qwerty_base(octave: i32) -> i32 {
    48 + octave * 12
}

#[component]
pub fn Keyboard() -> Element {
    let synth = use_context::<Synth>();
    let octave = (synth.octave)();
    let held = synth.held.read().clone();
    let base = screen_base(octave);
    let qbase = qwerty_base(octave);
    let whites: Vec<i32> = (0..OCTAVES * 12 + 1).filter(|s| !BLACK[(s % 12) as usize]).collect();
    let n_white = whites.len() as f32;

    let key = move |note: i32, black: bool, left: f32, width: f32| {
        let n = note.clamp(0, 127) as u8;
        let down = held.contains(&n);
        let class = format!("{} {}", if black { "key black" } else { "key white" }, if down { "down" } else { "" });
        let label = if note % 12 == 0 { format!("{}{}", NAMES[0], note / 12 - 1) } else { String::new() };
        let qwerty = (note - qbase) >= 0 && (note - qbase) < 18;
        rsx! {
            div {
                key: "{note}",
                class: "{class}",
                "data-qwerty": if qwerty { "1" } else { "0" },
                style: "left: {left}%; width: {width}%",
                onpointerdown: move |e| {
                    e.prevent_default();
                    let h = if black { 70.0 } else { 120.0 };
                    let vel = (0.35 + 0.65 * (e.data().element_coordinates().y / h) as f32).clamp(0.1, 1.0);
                    synth.note_on(n, vel);
                },
                onpointerenter: move |e| {
                    if !e.data().held_buttons().is_empty() {
                        synth.note_on(n, 0.8);
                    }
                },
                onpointerup: move |_| synth.note_off(n),
                onpointerleave: move |_| {
                    if synth.held.peek().contains(&n) {
                        synth.note_off(n);
                    }
                },
                span { "{label}" }
            }
        }
    };

    let ww = 100.0 / n_white;
    rsx! {
        div { class: "keyboard-wrap",
            div { class: "octave-ctl",
                button { onclick: move |_| synth.shift_octave(-1), "◀" }
                span { "OCT {octave:+}" }
                button { onclick: move |_| synth.shift_octave(1), "▶" }
                div { class: "kbd-hint", "Z / X" }
            }
            div { class: "keyboard",
                for (i, s) in whites.iter().enumerate() {
                    {key(base + s, false, i as f32 * ww, ww)}
                }
                for s in (0..OCTAVES * 12).filter(|s| BLACK[(s % 12) as usize]) {
                    {
                        let white_index = whites.iter().filter(|&&w| w < s).count() as f32;
                        key(base + s, true, white_index * ww - ww * 0.32, ww * 0.64)
                    }
                }
            }
        }
    }
}
