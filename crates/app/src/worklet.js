// OXIDE AudioWorklet: hosts the Rust DSP (bare wasm, C ABI, no imports).
class OxideProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();
    this.ok = false;
    try {
      const opts = options.processorOptions;
      const inst = new WebAssembly.Instance(new WebAssembly.Module(opts.wasm), {});
      this.ex = inst.exports;
      this.ex.oxide_init(sampleRate);
      const p = opts.params;
      for (let i = 0; i < p.length; i++) this.ex.oxide_set_param(i, p[i]);
      this.tlen = this.ex.oxide_telemetry_len();
      this.ok = true;
      this.port.postMessage({ t: "ready", sr: sampleRate, params: this.ex.oxide_param_count() });
    } catch (e) {
      this.port.postMessage({ t: "error", msg: String(e) });
    }
    this.blocks = 0;
    this.port.onmessage = (e) => {
      if (!this.ok) return;
      const m = e.data, x = this.ex;
      switch (m.t) {
        case "p": x.oxide_set_param(m.i, m.v); break;
        case "all": for (let i = 0; i < m.v.length; i++) x.oxide_set_param(i, m.v[i]); break;
        case "on": x.oxide_note_on(m.n, m.v); break;
        case "off": x.oxide_note_off(m.n); break;
        case "alloff": x.oxide_all_notes_off(); break;
        case "panic": x.oxide_panic(); break;
      }
    };
  }

  process(_inputs, outputs) {
    if (!this.ok) return true;
    const out = outputs[0];
    const n = out[0].length;
    const ptr = this.ex.oxide_process(n);
    const mem = this.ex.memory.buffer;
    const buf = new Float32Array(mem, ptr, 256);
    out[0].set(buf.subarray(0, n));
    if (out[1]) out[1].set(buf.subarray(128, 128 + n));
    // ~60 Hz telemetry for the visualisers.
    if (++this.blocks % 6 === 0) {
      const t = new Float32Array(mem, this.ex.oxide_telemetry(), this.tlen);
      this.port.postMessage({ t: "tm", d: t.slice() });
    }
    return true;
  }
}
registerProcessor("oxide", OxideProcessor);
