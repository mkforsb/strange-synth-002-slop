//! The parameter table. Every knob in the UI, every value in a preset and
//! every slot in the DSP engine's parameter array is described here.
//!
//! Values are stored in *plain* units (Hz, seconds, dB, 0..1 amounts...).
//! `ParamInfo::to_norm` / `from_norm` map to the 0..1 knob domain.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Group {
    Excite,
    Body,
    Throat,
    Mind,
    Chaos,
    Motion,
    Dirt,
    Voice,
    Filter,
    Eq,
    Reverb,
    Out,
}

impl Group {
    pub const ALL: [Group; 12] = [
        Group::Excite,
        Group::Body,
        Group::Throat,
        Group::Mind,
        Group::Chaos,
        Group::Motion,
        Group::Dirt,
        Group::Voice,
        Group::Filter,
        Group::Eq,
        Group::Reverb,
        Group::Out,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Group::Excite => "Excite",
            Group::Body => "Body",
            Group::Throat => "Throat",
            Group::Mind => "Glass & Mind",
            Group::Chaos => "Chaos",
            Group::Motion => "Motion",
            Group::Dirt => "Dirt",
            Group::Voice => "Voice",
            Group::Filter => "Master Filter",
            Group::Eq => "EQ",
            Group::Reverb => "Plate Reverb",
            Group::Out => "Out",
        }
    }

    /// The "strange" groups, as opposed to the conventional utility sections.
    pub fn is_strange(self) -> bool {
        matches!(
            self,
            Group::Excite
                | Group::Body
                | Group::Throat
                | Group::Mind
                | Group::Chaos
                | Group::Motion
                | Group::Dirt
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve {
    /// `min + n * (max - min)`
    Lin,
    /// `min * (max / min) ^ n` (min must be > 0)
    Exp,
    /// `min + n^2 * (max - min)`
    Sq,
    /// Integer steps between min and max.
    Step,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Unit {
    Pct,
    /// -1..1 shown as -100..+100%
    Bipolar,
    Hz,
    Sec,
    Ms,
    Db,
    Semi,
    Oct,
    Mult,
    Choice(&'static [&'static str]),
}

#[derive(Clone, Copy, Debug)]
pub struct ParamInfo {
    pub key: &'static str,
    pub name: &'static str,
    pub group: Group,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub curve: Curve,
    pub unit: Unit,
    pub help: &'static str,
}

pub const FILTER_TYPES: &[&str] = &["LP12", "LP24", "BP", "HP"];

macro_rules! params {
    ($( $var:ident : $key:literal, $name:literal, $group:ident, $min:expr, $max:expr, $def:expr, $curve:ident, $unit:expr, $help:literal; )*) => {
        /// Parameter index.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(usize)]
        pub enum P { $($var),* }

        pub const PARAMS: &[ParamInfo] = &[
            $(ParamInfo {
                key: $key,
                name: $name,
                group: Group::$group,
                min: $min,
                max: $max,
                default: $def,
                curve: Curve::$curve,
                unit: $unit,
                help: $help,
            }),*
        ];

        impl P {
            pub const ALL: &[P] = &[$(P::$var),*];
        }
    };
}

params! {
    // ── Excite ────────────────────────────────────────────────────────────
    Strike:   "strike",  "Strike",    Excite, 0.0, 1.0, 0.7, Lin, Unit::Pct, "Mallet impact into the resonators";
    Hardness: "hard",    "Hardness",  Excite, 0.0, 1.0, 0.5, Lin, Unit::Pct, "Mallet material: felt, rubber, plastic pipe, steel";
    Bow:      "bow",     "Bow",       Excite, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Stick-slip friction exciter (a bow, a brake, a rusty hinge)";
    Pressure: "press",   "Pressure",  Excite, 0.0, 1.0, 0.4, Lin, Unit::Pct, "Bow pressure. Lean on it for squeal and screech";
    Grit:     "grit",    "Grit",      Excite, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Random crackle impacts: rust flakes, rain, sand";
    Breath:   "breath",  "Breath",    Excite, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Continuous coloured noise / air";

    // ── Body ──────────────────────────────────────────────────────────────
    Metal:    "metal",   "Metal",     Body, 0.0, 1.0, 0.8, Lin, Unit::Pct, "Modal metal body (16 resonant modes)";
    Stiffness:"stiff",   "Stiffness", Body, 0.0, 1.0, 0.35, Lin, Unit::Pct, "Inharmonicity: string, bar, plate, bell";
    Shatter:  "shatter", "Shatter",   Body, 0.0, 1.0, 0.1, Lin, Unit::Pct, "Scatters the mode ratios like a cracked casting";
    Ring:     "ring",    "Ring",      Body, 0.05, 20.0, 2.5, Exp, Unit::Sec, "Decay time of the body";
    Rust:     "rust",    "Rust",      Body, 0.0, 1.0, 0.4, Lin, Unit::Pct, "Damps high modes: polished bronze to corroded iron";
    String:   "string",  "String",    Body, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Waveguide string";
    Mute:     "mute",    "Mute",      Body, 0.0, 1.0, 0.2, Lin, Unit::Pct, "Palm-mutes the string";
    Pipe:     "pipe",    "Pipe",      Body, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Closed-tube resonance (hollow, odd harmonics)";

    // ── Throat ────────────────────────────────────────────────────────────
    Growl:    "growl",   "Growl",     Throat, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Subharmonic, jittery saw throat oscillator";
    Swarm:    "swarm",   "Swarm",     Throat, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Detunes a swarm of growl oscillators";
    Fold:     "fold",    "Fold",      Throat, 0.0, 1.0, 0.0, Lin, Unit::Pct, "West-coast wavefolder on the oscillators";
    Scream:   "scream",  "Scream",    Throat, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Self-feedback FM: whistle, scream, noise";
    Throat:   "throat",  "Throat",    Throat, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Formant vocal tract";
    Vowel:    "vowel",   "Vowel",     Throat, 0.0, 1.0, 0.0, Lin, Unit::Pct, "A, E, I, O, U";
    Size:     "size",    "Size",      Throat, 0.0, 1.0, 0.5, Lin, Unit::Pct, "Creature size (formant shift)";

    // ── Glass & Mind ──────────────────────────────────────────────────────
    Glass:    "glass",   "Glass",     Mind, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Singing wine-glass partials";
    Wobble:   "wobble",  "Wobble",    Mind, 0.0, 1.0, 0.3, Lin, Unit::Pct, "Beating between the glass partials";
    Thought:  "thought", "Thought",   Mind, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Logistic-map machine blips";
    Logic:    "logic",   "Logic",     Mind, 0.0, 1.0, 0.5, Lin, Unit::Pct, "Logistic r: order, bifurcation, chaos";
    Tick:     "tick",    "Tick",      Mind, 0.5, 40.0, 8.0, Exp, Unit::Hz, "Thought rate";

    // ── Chaos ─────────────────────────────────────────────────────────────
    Chaos:    "chaos",   "Chaos",     Chaos, 0.0, 1.0, 0.3, Lin, Unit::Pct, "Lorenz attractor speed: drift to audio-rate turbulence";
    Drift:    "drift",   "Drift",     Chaos, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Lorenz to pitch";
    Unrest:   "unrest",  "Unrest",    Chaos, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Lorenz to body: modes, vowel, bow";
    Couple:   "couple",  "Couple",    Chaos, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Oscillators shake the resonators";

    // ── Motion ────────────────────────────────────────────────────────────
    Zap:      "zap",     "Zap",       Motion, -1.0, 1.0, 0.0, Lin, Unit::Bipolar, "Pitch envelope: + dives like a laser, - rises";
    ZapTime:  "zaptime", "Zap Time",  Motion, 0.005, 3.0, 0.25, Exp, Unit::Sec, "Pitch envelope time";
    Elevator: "elev",    "Elevator",  Motion, -1.0, 1.0, 0.0, Lin, Unit::Bipolar, "Pitch keeps travelling while the key is held";
    Spin:     "spin",    "Spin",      Motion, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Rotating body: Doppler, tremolo and pan";
    SpinRate: "spinrate","Spin Rate", Motion, 0.05, 30.0, 2.0, Exp, Unit::Hz, "Rotation speed";

    // ── Dirt ──────────────────────────────────────────────────────────────
    Hum:      "hum",     "Hum",       Dirt, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Mains hum and ground-loop buzz";
    Alien:    "alien",   "Alien",     Dirt, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Ring modulation at an irrational ratio";
    Corrode:  "corrode", "Corrode",   Dirt, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Bit and sample-rate corrosion";
    Sympathy: "symp",    "Sympathy",  Dirt, 0.0, 1.0, 0.0, Lin, Unit::Pct, "Sympathetic resonance of a hidden metal frame";

    // ── Voice ─────────────────────────────────────────────────────────────
    Octave:   "oct",     "Octave",    Voice, -3.0, 3.0, 0.0, Step, Unit::Oct, "Transpose in octaves";
    Tune:     "tune",    "Tune",      Voice, -12.0, 12.0, 0.0, Lin, Unit::Semi, "Fine transpose";
    Glide:    "glide",   "Glide",     Voice, 0.0, 2.0, 0.0, Sq, Unit::Sec, "Portamento from the previous note";
    Attack:   "att",     "Attack",    Voice, 0.001, 10.0, 0.002, Exp, Unit::Sec, "Source envelope attack";
    Decay:    "dec",     "Decay",     Voice, 0.005, 10.0, 0.5, Exp, Unit::Sec, "Source envelope decay";
    Sustain:  "sus",     "Sustain",   Voice, 0.0, 1.0, 0.7, Lin, Unit::Pct, "Source envelope sustain";
    Release:  "rel",     "Release",   Voice, 0.005, 15.0, 0.8, Exp, Unit::Sec, "Release for sources and body damper";
    Width:    "width",   "Width",     Voice, 0.0, 1.0, 0.5, Lin, Unit::Pct, "Stereo spread of modes and voices";

    // ── Master filter ─────────────────────────────────────────────────────
    FilterType:"ftype",  "Type",      Filter, 0.0, 3.0, 1.0, Step, Unit::Choice(FILTER_TYPES), "Moog ladder: LP12, LP24, bandpass, highpass";
    Cutoff:   "cutoff",  "Cutoff",    Filter, 20.0, 20000.0, 12000.0, Exp, Unit::Hz, "Ladder cutoff";
    Reso:     "reso",    "Reso",      Filter, 0.0, 1.0, 0.2, Lin, Unit::Pct, "Ladder resonance (self-oscillates near max)";
    Drive:    "drive",   "Drive",     Filter, 0.0, 1.0, 0.1, Lin, Unit::Pct, "Ladder input drive";

    // ── EQ ────────────────────────────────────────────────────────────────
    LowCut:   "lowcut",  "Low Cut",   Eq, 10.0, 1000.0, 20.0, Exp, Unit::Hz, "High-pass";
    Low:      "low",     "Low",       Eq, -15.0, 15.0, 0.0, Lin, Unit::Db, "Low shelf at 150 Hz";
    Mid:      "mid",     "Mid",       Eq, -15.0, 15.0, 0.0, Lin, Unit::Db, "Mid peak";
    MidFreq:  "midf",    "Mid Freq",  Eq, 150.0, 8000.0, 1000.0, Exp, Unit::Hz, "Mid peak frequency";
    High:     "high",    "High",      Eq, -15.0, 15.0, 0.0, Lin, Unit::Db, "High shelf at 5 kHz";
    HighCut:  "hicut",   "High Cut",  Eq, 500.0, 20000.0, 20000.0, Exp, Unit::Hz, "Low-pass";

    // ── Reverb ────────────────────────────────────────────────────────────
    RvMix:    "rvmix",   "Mix",       Reverb, 0.0, 1.0, 0.25, Lin, Unit::Pct, "Dattorro plate wet/dry";
    RvDecay:  "rvdecay", "Decay",     Reverb, 0.0, 0.99, 0.7, Lin, Unit::Pct, "Tank feedback";
    RvSize:   "rvsize",  "Size",      Reverb, 0.4, 2.0, 1.0, Lin, Unit::Mult, "Tank size";
    RvDamp:   "rvdamp",  "Damp",      Reverb, 0.0, 1.0, 0.3, Lin, Unit::Pct, "High-frequency damping in the tank";
    RvPre:    "rvpre",   "Predelay",  Reverb, 0.0, 250.0, 10.0, Sq, Unit::Ms, "Predelay";
    RvMod:    "rvmod",   "Mod",       Reverb, 0.0, 1.0, 0.3, Lin, Unit::Pct, "Tank modulation (excursion)";

    // ── Out ───────────────────────────────────────────────────────────────
    Volume:   "vol",     "Volume",    Out, -40.0, 6.0, -6.0, Lin, Unit::Db, "Master volume";
}

pub const NUM_PARAMS: usize = PARAMS.len();

impl P {
    #[inline]
    pub fn idx(self) -> usize {
        self as usize
    }
    #[inline]
    pub fn info(self) -> &'static ParamInfo {
        &PARAMS[self as usize]
    }
    pub fn from_key(key: &str) -> Option<P> {
        P::ALL.iter().copied().find(|p| p.info().key == key)
    }
}

impl ParamInfo {
    pub fn is_discrete(&self) -> bool {
        self.curve == Curve::Step
    }

    pub fn clamp(&self, v: f32) -> f32 {
        let v = if v.is_finite() { v } else { self.default };
        let v = v.clamp(self.min, self.max);
        if self.is_discrete() { v.round() } else { v }
    }

    pub fn to_norm(&self, v: f32) -> f32 {
        let v = self.clamp(v);
        let n = match self.curve {
            Curve::Lin | Curve::Step => (v - self.min) / (self.max - self.min),
            Curve::Exp => (v / self.min).ln() / (self.max / self.min).ln(),
            Curve::Sq => ((v - self.min) / (self.max - self.min)).sqrt(),
        };
        n.clamp(0.0, 1.0)
    }

    pub fn from_norm(&self, n: f32) -> f32 {
        let n = n.clamp(0.0, 1.0);
        let v = match self.curve {
            Curve::Lin => self.min + n * (self.max - self.min),
            Curve::Step => (self.min + n * (self.max - self.min)).round(),
            Curve::Exp => self.min * (self.max / self.min).powf(n),
            Curve::Sq => self.min + n * n * (self.max - self.min),
        };
        self.clamp(v)
    }

    /// Human readable value, e.g. "1.2 kHz", "350 ms", "+3.0 dB".
    pub fn format(&self, v: f32) -> String {
        match self.unit {
            Unit::Pct => format!("{:.0}%", v * 100.0),
            Unit::Bipolar => format!("{:+.0}%", v * 100.0),
            Unit::Hz => {
                if v >= 1000.0 {
                    format!("{:.1} kHz", v / 1000.0)
                } else if v >= 100.0 {
                    format!("{:.0} Hz", v)
                } else {
                    format!("{:.1} Hz", v)
                }
            }
            Unit::Sec => {
                if v < 1.0 {
                    format!("{:.0} ms", v * 1000.0)
                } else {
                    format!("{:.2} s", v)
                }
            }
            Unit::Ms => format!("{:.0} ms", v),
            Unit::Db => format!("{:+.1} dB", v),
            Unit::Semi => format!("{:+.2} st", v),
            Unit::Oct => format!("{:+.0} oct", v),
            Unit::Mult => format!("{:.2}x", v),
            Unit::Choice(labels) => labels
                .get(v.round().max(0.0) as usize)
                .copied()
                .unwrap_or("?")
                .to_string(),
        }
    }
}

pub fn default_values() -> [f32; NUM_PARAMS] {
    let mut v = [0.0; NUM_PARAMS];
    for (i, p) in PARAMS.iter().enumerate() {
        v[i] = p.default;
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_matches_table() {
        assert_eq!(P::ALL.len(), NUM_PARAMS);
        for (i, p) in P::ALL.iter().enumerate() {
            assert_eq!(p.idx(), i);
        }
    }

    #[test]
    fn keys_unique() {
        for (i, a) in PARAMS.iter().enumerate() {
            for b in &PARAMS[i + 1..] {
                assert_ne!(a.key, b.key);
            }
        }
    }

    #[test]
    fn at_least_30_strange_params() {
        let n = PARAMS.iter().filter(|p| p.group.is_strange()).count();
        assert!(n >= 30, "only {n} strange params");
    }

    #[test]
    fn defaults_in_range_and_norm_roundtrip() {
        for p in PARAMS {
            assert!(p.default >= p.min && p.default <= p.max, "{}", p.key);
            if p.curve == Curve::Exp {
                assert!(p.min > 0.0, "{}", p.key);
            }
            for i in 0..=20 {
                let n = i as f32 / 20.0;
                let v = p.from_norm(n);
                let n2 = p.to_norm(v);
                let v2 = p.from_norm(n2);
                assert!((v - v2).abs() <= 1e-3 * (p.max - p.min).abs().max(1.0), "{} {v} {v2}", p.key);
            }
        }
    }
}
