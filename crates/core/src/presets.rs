//! Factory presets. Each preset lists only the values that differ from the
//! parameter defaults (see `params.rs`).

use crate::params::{default_values, NUM_PARAMS, P};

#[derive(Clone, Copy, Debug)]
pub struct Preset {
    pub name: &'static str,
    pub category: &'static str,
    pub values: &'static [(P, f32)],
}

impl Preset {
    pub fn full_values(&self) -> [f32; NUM_PARAMS] {
        let mut v = default_values();
        for &(p, x) in self.values {
            v[p.idx()] = p.info().clamp(x);
        }
        v
    }
}

macro_rules! preset {
    ($name:literal, $cat:literal $(, $p:ident = $v:expr)* $(,)?) => {
        Preset { name: $name, category: $cat, values: &[$((P::$p, $v as f32)),*] }
    };
}

pub const CATEGORIES: &[&str] = &["Init", "Industrial", "Sci-Fi", "Creatures", "Glass", "Machine", "Percussion", "Drones"];

#[rustfmt::skip]
pub const PRESETS: &[Preset] = &[
    preset!("Init", "Init", Volume=-1.4),
    preset!("Metal-on-Metal Screeching Industrial Elevator", "Industrial", Strike=0.3, Hardness=0.9, Bow=0.85, Pressure=0.85, Grit=0.35, Metal=1.0, Stiffness=0.75, Shatter=0.55, Ring=3.0, Rust=0.25, Unrest=0.5, Chaos=0.55, Drift=0.15, Elevator=0.5, Hum=0.6, Sympathy=0.4, Corrode=0.1, Attack=0.3, Sustain=1.0, Release=2.0, Cutoff=9000, Reso=0.25, Drive=0.4, LowCut=60, Mid=4, MidFreq=2500, RvMix=0.35, RvDecay=0.8, RvSize=1.6, Width=0.9, Volume=-8.0),
    preset!("Struck Bowed Metal Sheet", "Industrial", Strike=0.9, Hardness=0.7, Bow=0.55, Pressure=0.55, Metal=1.0, Stiffness=0.9, Shatter=0.35, Ring=9.0, Rust=0.15, Unrest=0.25, Chaos=0.25, Attack=0.8, Decay=3, Sustain=0.8, Release=6, Width=1.0, Cutoff=14000, Reso=0.1, RvMix=0.4, RvDecay=0.85, RvSize=1.8, RvDamp=0.2, Octave=-1, Volume=-12.2),
    preset!("Resonating Metal Clothes Rack", "Industrial", Strike=0.8, Hardness=0.85, Metal=0.9, Stiffness=0.55, Shatter=0.8, Ring=4.0, Rust=0.35, Grit=0.15, Sympathy=0.85, Width=1.0, Release=4, Sustain=0.0, RvMix=0.3, RvDecay=0.6, RvSize=0.8, Octave=1, LowCut=120, Volume=-8.4),
    preset!("Rusted Boiler Room", "Industrial", Strike=0.5, Hardness=0.4, Metal=0.9, Stiffness=0.45, Shatter=0.5, Ring=2.0, Rust=0.9, Pipe=0.6, Grit=0.5, Hum=0.7, Breath=0.25, Octave=-1, Release=2, Sustain=1, Drive=0.5, Cutoff=3000, Reso=0.3, RvMix=0.35, RvSize=1.3, Volume=-11.1),
    preset!("Freight Container Door", "Industrial", Strike=0.4, Hardness=0.6, Bow=0.9, Pressure=0.95, Metal=1.0, Stiffness=0.5, Shatter=0.7, Ring=1.5, Rust=0.6, Grit=0.45, Elevator=-0.3, Unrest=0.6, Chaos=0.6, Octave=-1, Attack=0.1, Sustain=1, Release=1.2, Drive=0.5, RvMix=0.4, RvDecay=0.8, RvSize=2.0, RvDamp=0.5, Volume=-6.8),
    preset!("Factory Floor at 3AM", "Industrial", Strike=0.6, Hardness=0.75, Metal=0.7, Stiffness=0.6, Shatter=0.3, Ring=1.2, Rust=0.5, Thought=0.4, Logic=0.3, Tick=4, Couple=0.7, Hum=0.9, Grit=0.2, Corrode=0.2, Sustain=1.0, Release=1.5, Octave=-1, Cutoff=5000, RvMix=0.45, RvDecay=0.88, RvSize=2.0, RvPre=80, Volume=-2.3),
    preset!("Sheet Metal Thunder", "Industrial", Strike=0.9, Hardness=0.35, Metal=1.0, Stiffness=1.0, Shatter=0.9, Ring=6, Rust=0.5, Grit=0.6, Unrest=0.7, Chaos=0.7, Sympathy=0.5, Octave=-2, Release=6, Width=1.0, Drive=0.4, Low=5, RvMix=0.45, RvDecay=0.85, RvSize=2.0, Volume=-20.4),
    preset!("Hydraulic Press Lament", "Industrial", Bow=0.7, Pressure=0.7, Strike=0.0, Metal=0.9, Stiffness=0.4, Ring=4, Breath=0.6, Hardness=0.2, Throat=0.6, Vowel=0.75, Size=0.8, Elevator=-0.2, Hum=0.4, Attack=0.8, Sustain=1, Release=3, Octave=-1, Pipe=0.3, Cutoff=4000, RvMix=0.35, Volume=-6.8),
    preset!("Submarine Hull Stress", "Industrial", Bow=0.6, Pressure=0.9, Strike=0.2, Hardness=0.3, Metal=1.0, Stiffness=0.3, Shatter=0.6, Ring=7, Rust=0.5, Unrest=0.8, Chaos=0.2, Elevator=-0.2, Pipe=0.35, Octave=-2, Attack=1.0, Sustain=1, Release=4, FilterType=0, Cutoff=1800, Reso=0.3, RvMix=0.5, RvDecay=0.9, RvSize=2.0, RvDamp=0.6, Volume=-6.7),
    preset!("UFO Engine Room", "Sci-Fi", Strike=0.0, Metal=0.6, Bow=0.3, Pressure=0.6, Stiffness=0.6, Shatter=0.4, Ring=5.0, Rust=0.3, Growl=0.4, Swarm=0.7, Scream=0.15, Pipe=0.5, Spin=0.85, SpinRate=0.9, Hum=0.8, Chaos=0.35, Drift=0.2, Unrest=0.4, Couple=0.5, Sympathy=0.6, Alien=0.35, Octave=-2, Attack=1.5, Sustain=1.0, Release=4, FilterType=0, Cutoff=2500, Reso=0.45, Drive=0.3, RvMix=0.45, RvDecay=0.85, RvSize=2.0, RvMod=0.7, Width=1.0, Volume=-9.7),
    preset!("Godzilla Laser Beam", "Sci-Fi", Strike=0, Metal=0.4, Stiffness=0.9, Ring=1.0, Scream=0.9, Growl=0.5, Swarm=0.5, Fold=0.5, Zap=0.75, ZapTime=1.2, Spin=0.5, SpinRate=18, Alien=0.4, Corrode=0.25, Grit=0.25, Breath=0.2, Couple=0.6, Chaos=0.85, Unrest=0.5, Drift=0.15, Attack=0.05, Sustain=1.0, Release=1.5, Cutoff=8000, Reso=0.35, Drive=0.6, Mid=5, MidFreq=1800, RvMix=0.35, RvDecay=0.8, RvSize=1.5, Volume=-5.8),
    preset!("Tractor Beam", "Sci-Fi", Scream=0.5, Glass=0.4, Spin=0.7, SpinRate=7, Elevator=0.25, Alien=0.3, Metal=0, Strike=0, Pipe=0.4, Chaos=0.4, Unrest=0.3, Attack=0.6, Sustain=1, Release=2, FilterType=2, Cutoff=1400, Reso=0.6, RvMix=0.4, Volume=3.9),
    preset!("Hyperspace Ignition", "Sci-Fi", Zap=-0.85, ZapTime=2.5, Growl=0.7, Swarm=0.9, Scream=0.4, Breath=0.6, Hardness=0.8, Strike=0, Metal=0.3, Stiffness=1.0, Ring=1, Spin=0.4, SpinRate=12, Attack=0.5, Sustain=1, Release=3, Cutoff=9000, Drive=0.5, RvMix=0.4, RvSize=2.0, Volume=-3.7),
    preset!("Alien Telegraph", "Sci-Fi", Thought=1.0, Logic=1.0, Tick=22, Alien=0.7, Strike=0, Metal=0, Scream=0.2, Corrode=0.3, Sustain=1, Release=0.5, Octave=1, FilterType=2, Cutoff=1800, Reso=0.5, RvMix=0.3, Volume=1.3),
    preset!("Mothership Choir", "Sci-Fi", Growl=0.7, Swarm=0.8, Throat=1.0, Vowel=0.55, Size=0.65, Spin=0.5, SpinRate=0.4, Glass=0.2, Strike=0, Metal=0, Chaos=0.2, Unrest=0.5, Drift=0.1, Attack=1.5, Sustain=1, Release=4, Alien=0.15, Cutoff=7000, RvMix=0.55, RvDecay=0.9, RvSize=2.0, Volume=4.8),
    preset!("Dead Satellite", "Sci-Fi", Scream=0.35, Glass=0.3, Alien=0.5, Corrode=0.45, Spin=0.3, SpinRate=0.15, Thought=0.3, Logic=0.95, Tick=2, Strike=0, Metal=0.3, Stiffness=0.9, Couple=0.5, Drift=0.2, Chaos=0.5, Attack=2, Sustain=1, Release=5, Octave=1, FilterType=2, Cutoff=3000, Reso=0.55, RvMix=0.6, RvDecay=0.92, RvSize=2.0, LowCut=200, Volume=4.2),
    preset!("Teleporter Malfunction", "Sci-Fi", Zap=0.5, ZapTime=0.08, Thought=0.7, Logic=0.95, Tick=30, Scream=0.5, Fold=0.6, Corrode=0.6, Alien=0.4, Strike=0, Metal=0.4, Stiffness=1, Couple=0.6, Chaos=0.9, Unrest=0.8, Drift=0.4, Sustain=1, Release=0.6, RvMix=0.3, Volume=-2.3),
    preset!("Quantum Foam", "Sci-Fi", Grit=0.9, Hardness=1.0, Strike=0, Metal=0.9, Stiffness=0.95, Shatter=1.0, Ring=0.3, Rust=0.0, Glass=0.15, Unrest=1.0, Chaos=1.0, Sustain=1, Release=2, Octave=2, Width=1.0, LowCut=300, RvMix=0.5, RvSize=1.2, Volume=-2.7),
    preset!("Cryo Chamber Hum", "Sci-Fi", Hum=0.9, Breath=0.4, Hardness=0.9, Glass=0.35, Wobble=0.1, Strike=0, Metal=0.4, Bow=0.15, Pressure=0.2, Stiffness=0.6, Ring=10, Rust=0.1, Pipe=0.3, Attack=2.5, Sustain=1, Release=5, Octave=-1, FilterType=0, Cutoff=1800, Reso=0.4, RvMix=0.5, RvDecay=0.9, RvDamp=0.1, Volume=-3.1),
    preset!("T-Rex Roar", "Creatures", Strike=0, Metal=0, Growl=1.0, Swarm=0.35, Fold=0.35, Scream=0.25, Throat=1.0, Vowel=0.05, Size=0.95, Breath=0.5, Hardness=0.3, Chaos=0.75, Drift=0.35, Unrest=0.6, Zap=-0.25, ZapTime=0.6, Elevator=-0.3, Octave=-2, Attack=0.25, Decay=1.5, Sustain=0.6, Release=1.2, Glide=0.3, Drive=0.5, Cutoff=3500, Reso=0.2, Low=4, LowCut=25, RvMix=0.3, RvDecay=0.75, RvSize=1.8, RvPre=30, Volume=-2.7),
    preset!("Kaiju Footstep", "Creatures", Strike=1.0, Hardness=0.1, Metal=0.9, Stiffness=0.2, Shatter=0.6, Ring=3, Rust=0.95, Pipe=0.3, Grit=0.3, Growl=0.3, Throat=0.6, Size=1.0, Decay=0.8, Sustain=0, Octave=-3, Drive=0.7, Low=6, Cutoff=1500, RvMix=0.4, RvDecay=0.8, RvSize=2.0, Release=3, Volume=-12.9),
    preset!("Leviathan Song", "Creatures", Growl=0.4, Glass=0.5, Wobble=0.15, Throat=0.9, Vowel=0.85, Size=1.0, Glide=0.9, Drift=0.3, Chaos=0.15, Elevator=0.08, Strike=0, Metal=0, Attack=1.2, Sustain=1, Release=4, Octave=-1, Cutoff=3000, RvMix=0.65, RvDecay=0.93, RvSize=2.0, RvPre=100, RvDamp=0.6, Volume=3.3),
    preset!("Swamp Thing Gurgle", "Creatures", Growl=0.6, Breath=0.5, Hardness=0.1, Throat=1.0, Vowel=0.8, Size=0.8, Unrest=1.0, Chaos=0.75, Drift=0.25, Pipe=0.5, Strike=0, Metal=0, Fold=0.2, Octave=-1, Attack=0.2, Sustain=0.9, Release=1, FilterType=0, Cutoff=1500, Reso=0.5, Drive=0.5, RvMix=0.25, Volume=-6.6),
    preset!("Insect Hive Mind", "Creatures", Growl=0.5, Swarm=1.0, Scream=0.3, Throat=0.7, Vowel=0.45, Size=0.0, Drift=0.15, Chaos=0.9, Thought=0.3, Logic=0.95, Tick=20, Strike=0, Metal=0, Spin=0.3, SpinRate=9, Octave=1, Sustain=1, Release=0.8, LowCut=300, RvMix=0.25, Volume=3.9),
    preset!("Banshee", "Creatures", Scream=0.6, Glass=0.3, Throat=0.8, Vowel=0.5, Size=0.15, Drift=0.35, Chaos=0.45, Elevator=0.15, Glide=0.5, Breath=0.4, Hardness=0.8, Strike=0, Metal=0, Attack=0.6, Sustain=1, Release=3, Octave=1, RvMix=0.6, RvDecay=0.9, RvSize=2.0, Volume=-1.0),
    preset!("Cave Troll Snore", "Creatures", Growl=0.9, Breath=0.7, Hardness=0.15, Throat=1.0, Vowel=0.9, Size=1.0, Unrest=0.6, Chaos=0.4, Zap=-0.2, ZapTime=1.5, Strike=0, Metal=0, Octave=-2, Attack=0.8, Decay=1.5, Sustain=0.4, Release=1, FilterType=0, Cutoff=900, Reso=0.3, Drive=0.6, RvMix=0.2, RvSize=0.6, Volume=-1.6),
    preset!("Dragon Breath", "Creatures", Breath=1.0, Hardness=0.6, Growl=0.5, Fold=0.5, Throat=0.7, Vowel=0.1, Size=0.9, Grit=0.4, Strike=0, Metal=0.5, Stiffness=0.9, Shatter=0.6, Ring=0.5, Couple=0.5, Chaos=0.8, Unrest=0.6, Zap=0.2, ZapTime=1.0, Attack=0.15, Sustain=0.8, Release=1.5, Octave=-1, Drive=0.6, RvMix=0.35, Volume=-6.6),
    preset!("Gargoyle Choir", "Creatures", Growl=0.6, Swarm=0.5, Throat=1.0, Vowel=0.7, Size=0.75, Strike=0.4, Hardness=0.3, Metal=0.6, Stiffness=0.5, Ring=4, Rust=0.7, Couple=0.4, Corrode=0.1, Attack=0.9, Sustain=1, Release=3, Octave=-1, RvMix=0.55, RvDecay=0.9, RvSize=2.0, Volume=3.7),
    preset!("Hungry Ghost", "Creatures", Glass=0.5, Wobble=0.8, Throat=0.6, Vowel=0.95, Size=0.3, Breath=0.5, Hardness=0.7, Drift=0.3, Chaos=0.25, Strike=0, Metal=0, Elevator=-0.1, Attack=2, Sustain=1, Release=5, FilterType=0, Cutoff=3500, Reso=0.4, RvMix=0.7, RvDecay=0.94, RvSize=2.0, RvPre=150, Volume=6.0),
    preset!("Haunting Singing Glass", "Glass", Strike=0.15, Hardness=0.95, Metal=0.6, Stiffness=0.5, Shatter=0.05, Ring=8, Rust=0.05, Bow=0.25, Pressure=0.25, Glass=0.9, Wobble=0.35, Unrest=0.15, Chaos=0.15, Drift=0.08, Attack=1.2, Decay=2, Sustain=0.85, Release=5, Octave=1, Width=0.8, LowCut=200, Cutoff=16000, Reso=0.0, RvMix=0.55, RvDecay=0.9, RvSize=2.0, RvDamp=0.15, RvMod=0.6, RvPre=40, Sympathy=0.3, Volume=-3.7),
    preset!("Crystal Lattice", "Glass", Strike=0.8, Hardness=1.0, Metal=1.0, Stiffness=0.8, Shatter=0.25, Ring=6, Rust=0.0, Glass=0.25, Sympathy=0.5, Octave=2, Width=1, Release=5, LowCut=250, RvMix=0.45, RvDecay=0.85, Volume=-10.2),
    preset!("Frozen Lake Cracks", "Glass", Strike=0.9, Hardness=0.95, Grit=0.6, Metal=1.0, Stiffness=1.0, Shatter=0.9, Ring=2.5, Rust=0.1, Zap=0.35, ZapTime=0.05, Elevator=-0.1, Width=1, Sustain=1, Release=3, LowCut=80, RvMix=0.55, RvDecay=0.9, RvSize=2.0, RvDamp=0.1, Volume=-12.9),
    preset!("Wine Glass Armonica", "Glass", Glass=1.0, Wobble=0.2, Bow=0.2, Pressure=0.2, Strike=0, Metal=0.5, Stiffness=0.45, Ring=6, Rust=0.0, Attack=0.4, Sustain=1, Release=2.5, Octave=1, LowCut=150, RvMix=0.4, RvDecay=0.82, Volume=-4.4),
    preset!("Ice Cave Wind", "Glass", Breath=0.9, Hardness=0.95, Metal=0.6, Stiffness=0.85, Ring=5, Rust=0.1, Glass=0.25, Wobble=0.7, Strike=0, Pipe=0.5, Unrest=0.5, Chaos=0.15, Attack=2, Sustain=1, Release=4, FilterType=2, Cutoff=3000, Reso=0.6, RvMix=0.6, RvDecay=0.92, RvSize=2.0, Volume=6.0),
    preset!("Broken Music Box", "Glass", Strike=0.8, Hardness=0.95, Metal=1.0, Stiffness=0.7, Shatter=0.35, Ring=1.8, Rust=0.2, Corrode=0.35, Drift=0.25, Chaos=0.2, Octave=2, Elevator=-0.05, Sustain=0, Decay=0.5, Release=2, LowCut=300, Cutoff=9000, RvMix=0.35, Volume=-6.0),
    preset!("Prism Bells", "Glass", Strike=0.9, Hardness=0.85, Metal=1.0, Stiffness=0.6, Shatter=0.0, Ring=7, Rust=0.05, Spin=0.4, SpinRate=4, Width=1, Release=6, Octave=1, Sympathy=0.35, RvMix=0.45, RvDecay=0.88, Volume=-10.1),
    preset!("Thoughts of the Machine", "Machine", Strike=0.3, Hardness=0.9, Metal=0.35, Stiffness=0.7, Ring=0.6, Thought=1.0, Logic=0.85, Tick=9, Glass=0.15, Wobble=0.6, Couple=0.5, Corrode=0.15, Hum=0.25, Chaos=0.4, Unrest=0.3, Spin=0.2, SpinRate=0.3, Sustain=1.0, Release=2, Octave=1, FilterType=2, Cutoff=2200, Reso=0.3, RvMix=0.45, RvDecay=0.85, RvSize=1.4, RvPre=60, RvMod=0.6, Volume=0.6),
    preset!("Lorenz Butterfly", "Machine", Strike=0, Metal=0.7, Bow=0.45, Pressure=0.5, Stiffness=0.5, Ring=3, Glass=0.3, Drift=0.5, Unrest=0.6, Chaos=0.25, Spin=0.4, SpinRate=0.3, Attack=0.8, Sustain=1, Release=3, RvMix=0.5, RvDecay=0.88, Volume=-5.4),
    preset!("Logistic Bifurcation", "Machine", Thought=1.0, Logic=0.55, Tick=12, Strike=0.5, Hardness=0.85, Metal=0.5, Stiffness=0.55, Ring=0.8, Couple=0.8, Sustain=1, Release=1, FilterType=2, Cutoff=3000, Reso=0.35, RvMix=0.35, Volume=6.0),
    preset!("Modem Prayer", "Machine", Thought=1.0, Logic=0.98, Tick=35, Scream=0.3, Throat=0.6, Vowel=0.4, Size=0.3, Corrode=0.4, Alien=0.2, Strike=0, Metal=0, Sustain=1, Release=1, Octave=1, FilterType=2, Cutoff=2000, Reso=0.4, RvMix=0.5, RvDecay=0.9, RvPre=50, Volume=5.3),
    preset!("Circuit Bent Toy", "Machine", Growl=0.4, Fold=0.8, Scream=0.4, Corrode=0.7, Thought=0.5, Logic=0.7, Tick=6, Glide=0.15, Strike=0, Metal=0, Drift=0.2, Chaos=0.6, Sustain=0.6, Decay=0.4, Release=0.3, Octave=1, Cutoff=6000, Reso=0.3, RvMix=0.15, Volume=1.8),
    preset!("Corrupted Memory", "Machine", Glass=0.4, Thought=0.5, Logic=0.92, Tick=5, Corrode=0.85, Alien=0.3, Elevator=-0.15, Strike=0.3, Metal=0.5, Stiffness=0.7, Ring=2, Drift=0.3, Chaos=0.35, Attack=0.5, Sustain=1, Release=3, RvMix=0.5, RvDecay=0.9, Volume=3.3),
    preset!("Neural Static", "Machine", Grit=1.0, Hardness=0.9, Breath=0.3, Strike=0, Metal=0.5, Stiffness=0.9, Shatter=1, Ring=0.2, Thought=0.4, Logic=1.0, Tick=40, Alien=0.5, Corrode=0.3, Unrest=1, Chaos=1, Sustain=1, Release=0.5, FilterType=2, Cutoff=4000, Reso=0.4, RvMix=0.25, Volume=6.0),
    preset!("Clockwork Heart", "Machine", Thought=0.6, Logic=0.1, Tick=2.2, Strike=0.5, Hardness=0.6, Metal=0.6, Stiffness=0.3, Ring=0.4, Rust=0.6, String=0.5, Mute=0.6, Couple=1.0, Pipe=0.3, Sustain=1, Release=0.6, Octave=-1, Cutoff=3000, RvMix=0.25, RvSize=0.6, Volume=4.3),
    preset!("Hitting Muted Strings with a Plastic Pipe", "Percussion", Strike=1.0, Hardness=0.55, Metal=0.25, Stiffness=0.15, Shatter=0.3, Ring=0.6, Rust=0.8, String=1.0, Mute=0.8, Pipe=0.45, Release=0.4, Width=0.6, Cutoff=6000, Reso=0.15, Drive=0.3, Low=3, RvMix=0.15, RvDecay=0.5, RvSize=0.6, Volume=3.2),
    preset!("Rebar Harp", "Percussion", Strike=0.85, Hardness=0.8, Metal=0.8, Stiffness=0.3, Shatter=0.15, Ring=5, Rust=0.25, String=0.6, Mute=0.05, Width=0.9, Release=4, RvMix=0.3, RvDecay=0.8, Volume=-7.4),
    preset!("Chain-Link Fence Rain", "Percussion", Strike=0.2, Hardness=0.9, Grit=0.85, Metal=0.8, Stiffness=0.65, Shatter=1.0, Ring=1.5, Rust=0.3, Sympathy=0.4, Sustain=1, Release=2, Attack=0.5, Octave=1, Width=1.0, LowCut=150, RvMix=0.4, RvSize=1.5, Volume=-10.4),
    preset!("Haunted Gamelan", "Percussion", Strike=0.9, Hardness=0.75, Metal=1.0, Stiffness=0.55, Shatter=0.2, Ring=5, Rust=0.2, Drift=0.15, Chaos=0.1, Glass=0.2, Wobble=0.5, Sympathy=0.45, Width=1, Release=5, Sustain=0, Decay=1.5, RvMix=0.45, RvDecay=0.88, RvPre=30, Volume=-14.9),
    preset!("Bent Spoon Kalimba", "Percussion", Strike=0.85, Hardness=0.65, Metal=0.9, Stiffness=0.25, Shatter=0.45, Ring=1.2, Rust=0.5, Pipe=0.35, Elevator=-0.08, Drift=0.1, Release=1.5, Octave=1, RvMix=0.25, RvSize=0.7, Volume=-3.1),
    preset!("Oil Drum Bass", "Percussion", Strike=1.0, Hardness=0.3, Metal=1.0, Stiffness=0.15, Shatter=0.35, Ring=1.5, Rust=0.6, Pipe=0.6, Zap=0.2, ZapTime=0.08, Octave=-2, Drive=0.5, Low=4, Cutoff=2500, Release=1.5, RvMix=0.15, Volume=-12.9),
    preset!("Gong from the Deep", "Percussion", Strike=1.0, Hardness=0.25, Metal=1.0, Stiffness=0.7, Shatter=0.3, Ring=14, Rust=0.25, Unrest=0.3, Chaos=0.12, Sympathy=0.4, Octave=-2, Release=10, Width=1, RvMix=0.45, RvDecay=0.9, RvSize=2.0, Volume=-15.7),
    preset!("Plastic Bucket Drum", "Percussion", Strike=1.0, Hardness=0.5, Metal=0.6, Stiffness=0.1, Shatter=0.6, Ring=0.35, Rust=0.9, Pipe=0.7, Zap=0.25, ZapTime=0.04, Octave=-1, Release=0.5, Cutoff=4000, Drive=0.4, RvMix=0.15, RvSize=0.5, Volume=-2.7),
    preset!("Tuned Radiator", "Percussion", Strike=0.9, Hardness=0.85, Metal=1.0, Stiffness=0.4, Shatter=0.5, Ring=2.5, Rust=0.45, Sympathy=0.6, Hum=0.3, Pipe=0.25, Release=2.5, Width=0.8, RvMix=0.3, Volume=-13.0),
    preset!("Bowed Saw Blade", "Percussion", Bow=0.6, Pressure=0.45, Strike=0, Metal=1.0, Stiffness=0.3, Shatter=0.1, Ring=6, Rust=0.1, Drift=0.25, Chaos=0.15, Glide=0.4, Attack=0.5, Sustain=1, Release=2, Spin=0.15, SpinRate=5.5, RvMix=0.45, RvDecay=0.85, Volume=-11.6),
    preset!("Abandoned Steel Mill", "Drones", Strike=0.3, Hardness=0.2, Bow=0.4, Pressure=0.3, Metal=0.8, Stiffness=0.55, Shatter=0.4, Ring=12, Rust=0.2, Breath=0.3, Hum=0.5, Sympathy=0.8, Unrest=0.3, Chaos=0.1, Attack=3, Sustain=1, Release=8, Octave=-2, Cutoff=2500, Reso=0.3, RvMix=0.6, RvDecay=0.93, RvSize=2.0, RvPre=120, Volume=-17.7),
    preset!("Rust Belt Drone", "Drones", Bow=0.5, Pressure=0.65, Strike=0, Metal=0.8, Stiffness=0.45, Ring=8, Rust=0.7, Growl=0.3, Swarm=0.6, Pipe=0.4, Hum=0.4, Couple=0.3, Unrest=0.3, Chaos=0.1, Octave=-2, Attack=3, Sustain=1, Release=6, Cutoff=2000, Reso=0.35, Drive=0.4, RvMix=0.5, RvDecay=0.9, Volume=-14.1),
    preset!("Deep Sea Cable", "Drones", Growl=0.4, Bow=0.3, Pressure=0.7, Strike=0, Metal=0.7, String=0.5, Mute=0.0, Ring=15, Stiffness=0.15, Throat=0.5, Vowel=0.9, Size=1.0, Drift=0.2, Chaos=0.08, Octave=-3, Attack=4, Sustain=1, Release=8, FilterType=0, Cutoff=900, Reso=0.4, RvMix=0.55, RvDecay=0.92, RvSize=2, RvDamp=0.7, Volume=-12.0),
    preset!("Cathedral of Pipes", "Drones", Breath=0.6, Hardness=0.5, Pipe=1.0, Growl=0.3, Glass=0.3, Strike=0, Metal=0, Swarm=0.4, Attack=1.5, Sustain=1, Release=4, Octave=-1, RvMix=0.65, RvDecay=0.94, RvSize=2.0, RvPre=60, Width=1, Volume=-7.7),
    preset!("Void Choir", "Drones", Growl=0.5, Swarm=1.0, Throat=1.0, Vowel=0.3, Size=0.6, Glass=0.25, Unrest=0.6, Chaos=0.1, Drift=0.15, Spin=0.3, SpinRate=0.2, Strike=0, Metal=0, Attack=3, Sustain=1, Release=6, RvMix=0.7, RvDecay=0.95, RvSize=2.0, RvDamp=0.4, Volume=6.0),
    preset!("Solar Wind", "Drones", Breath=0.8, Hardness=0.75, Glass=0.3, Wobble=0.9, Scream=0.15, Spin=0.6, SpinRate=0.12, Unrest=0.5, Chaos=0.2, Alien=0.2, Strike=0, Metal=0.5, Stiffness=0.8, Ring=6, Elevator=0.05, Attack=3, Sustain=1, Release=6, FilterType=1, Cutoff=6000, Reso=0.4, RvMix=0.6, RvDecay=0.92, RvSize=2.0, Volume=6.0),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::PARAMS;

    #[test]
    fn at_least_50_factory_presets() {
        let factory = PRESETS.iter().filter(|p| p.category != "Init").count();
        assert!(factory >= 50, "only {factory} presets");
    }

    #[test]
    fn names_unique_and_categories_known() {
        for (i, a) in PRESETS.iter().enumerate() {
            assert!(CATEGORIES.contains(&a.category), "{}", a.category);
            for b in &PRESETS[i + 1..] {
                assert_ne!(a.name, b.name);
            }
        }
    }

    #[test]
    fn values_in_range_and_not_duplicated() {
        for p in PRESETS {
            for (i, &(k, v)) in p.values.iter().enumerate() {
                let info = &PARAMS[k.idx()];
                assert!(v >= info.min && v <= info.max, "{}: {} = {}", p.name, info.key, v);
                assert!(!p.values[i + 1..].iter().any(|&(k2, _)| k2 == k), "{}: {} twice", p.name, info.key);
            }
        }
    }

    #[test]
    fn wishlist_presets_present() {
        for name in [
            "Metal-on-Metal Screeching Industrial Elevator",
            "Struck Bowed Metal Sheet",
            "UFO Engine Room",
            "T-Rex Roar",
            "Godzilla Laser Beam",
            "Resonating Metal Clothes Rack",
            "Hitting Muted Strings with a Plastic Pipe",
            "Haunting Singing Glass",
            "Thoughts of the Machine",
        ] {
            assert!(PRESETS.iter().any(|p| p.name == name), "missing {name}");
        }
    }
}
