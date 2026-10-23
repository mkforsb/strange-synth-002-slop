//! WebAudio glue: AudioContext → AudioWorkletNode (Rust DSP) → destination,
//! plus a channel splitter feeding two analysers for the visualisers.

use js_sys::{Array, Float32Array, Object, Reflect, Uint8Array};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AnalyserNode, AudioContext, AudioContextState, AudioWorkletNode, AudioWorkletNodeOptions, Blob, BlobPropertyBag,
    MessageEvent, MessagePort, Url,
};

const DSP_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/oxide_dsp.wasm"));
const WORKLET_JS: &str = include_str!("worklet.js");

pub struct Audio {
    pub ctx: AudioContext,
    pub port: MessagePort,
    pub analyser_l: AnalyserNode,
    pub analyser_r: AnalyserNode,
    pub telemetry: Rc<RefCell<Vec<f32>>>,
    _node: AudioWorkletNode,
    _onmessage: Closure<dyn FnMut(MessageEvent)>,
}

thread_local! {
    static AUDIO: RefCell<Option<Rc<Audio>>> = const { RefCell::new(None) };
}

pub fn get() -> Option<Rc<Audio>> {
    AUDIO.with(|a| a.borrow().clone())
}

pub async fn start(initial: &[f32]) -> Result<(), JsValue> {
    if let Some(a) = get() {
        let _ = a.ctx.resume();
        return Ok(());
    }
    let ctx = AudioContext::new()?;

    let opts = BlobPropertyBag::new();
    opts.set_type("application/javascript");
    let blob = Blob::new_with_str_sequence_and_options(&Array::of1(&JsValue::from_str(WORKLET_JS)), &opts)?;
    let url = Url::create_object_url_with_blob(&blob)?;
    JsFuture::from(ctx.audio_worklet()?.add_module(&url)?).await?;
    let _ = Url::revoke_object_url(&url);

    let proc_opts = Object::new();
    Reflect::set(&proc_opts, &"wasm".into(), &Uint8Array::from(DSP_WASM).buffer())?;
    Reflect::set(&proc_opts, &"params".into(), &Float32Array::from(initial))?;
    let o = AudioWorkletNodeOptions::new();
    o.set_number_of_inputs(0);
    o.set_number_of_outputs(1);
    o.set_output_channel_count(&Array::of1(&2.into()));
    o.set_processor_options(Some(&proc_opts));
    let node = AudioWorkletNode::new_with_options(&ctx, "oxide", &o)?;
    node.connect_with_audio_node(&ctx.destination())?;

    let split = ctx.create_channel_splitter_with_number_of_outputs(2)?;
    node.connect_with_audio_node(&split)?;
    let mk = || -> Result<AnalyserNode, JsValue> {
        let a = ctx.create_analyser()?;
        a.set_fft_size(2048);
        a.set_smoothing_time_constant(0.0);
        a.set_min_decibels(-110.0);
        a.set_max_decibels(-15.0);
        Ok(a)
    };
    let analyser_l = mk()?;
    let analyser_r = mk()?;
    split.connect_with_audio_node_and_output(&analyser_l, 0)?;
    split.connect_with_audio_node_and_output(&analyser_r, 1)?;

    let port = node.port()?;
    let telemetry = Rc::new(RefCell::new(vec![0.0f32; oxide_core::telemetry::LEN]));
    let tm = telemetry.clone();
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        let data = e.data();
        let kind = Reflect::get(&data, &"t".into()).ok().and_then(|v| v.as_string());
        match kind.as_deref() {
            Some("tm") => {
                if let Ok(d) = Reflect::get(&data, &"d".into()) {
                    let arr = Float32Array::from(d);
                    let mut t = tm.borrow_mut();
                    if arr.length() as usize == t.len() {
                        arr.copy_to(&mut t[..]);
                    }
                }
            }
            Some("error") => {
                let msg = Reflect::get(&data, &"msg".into()).unwrap_or(JsValue::NULL);
                web_sys::console::error_2(&"OXIDE worklet error:".into(), &msg);
            }
            Some("ready") => {
                let n = Reflect::get(&data, &"params".into()).ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
                if n as usize != oxide_core::NUM_PARAMS {
                    web_sys::console::error_1(&"OXIDE: DSP/UI parameter count mismatch".into());
                }
            }
            _ => {}
        }
    });
    port.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

    let _ = ctx.resume();
    AUDIO.with(|a| {
        *a.borrow_mut() = Some(Rc::new(Audio {
            ctx,
            port,
            analyser_l,
            analyser_r,
            telemetry,
            _node: node,
            _onmessage: onmessage,
        }))
    });
    Ok(())
}

fn post(fields: &[(&str, JsValue)]) {
    if let Some(a) = get() {
        let o = Object::new();
        for (k, v) in fields {
            let _ = Reflect::set(&o, &(*k).into(), v);
        }
        let _ = a.port.post_message(&o);
    }
}

pub fn set_param(i: usize, v: f32) {
    post(&[("t", "p".into()), ("i", (i as u32).into()), ("v", v.into())]);
}

pub fn set_all(values: &[f32]) {
    post(&[("t", "all".into()), ("v", Float32Array::from(values).into())]);
}

pub fn note_on(note: u8, vel: f32) {
    if let Some(a) = get()
        && a.ctx.state() == AudioContextState::Suspended {
            let _ = a.ctx.resume();
        }
    post(&[("t", "on".into()), ("n", (note as u32).into()), ("v", vel.into())]);
}

pub fn note_off(note: u8) {
    post(&[("t", "off".into()), ("n", (note as u32).into())]);
}

pub fn panic() {
    post(&[("t", "panic".into())]);
}
