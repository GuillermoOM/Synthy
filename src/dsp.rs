use fundsp::hacker::*;

/// A custom ADSR node that reads parameters from Shared handles in real-time.
/// This allows changing Attack, Decay, Sustain, and Release while a note is playing.
#[derive(Clone)]
struct SharedAdsr {
    attack: Shared,
    decay: Shared,
    sustain: Shared,
    release: Shared,
    // Locked parameters for the current note
    active_attack: f32,
    active_decay: f32,
    active_sustain: f32,
    active_release: f32,
    sample_rate: f32,
    value: f32,
    stage: u8, // 0: Idle, 1: Attack, 2: Decay, 3: Sustain, 4: Release
    last_gate: f32,
}

impl SharedAdsr {
    fn new(attack: Shared, decay: Shared, sustain: Shared, release: Shared) -> Self {
        Self {
            active_attack: attack.value().max(0.001),
            active_decay: decay.value().max(0.001),
            active_sustain: sustain.value().clamp(0.0, 1.0),
            active_release: release.value().max(0.001),
            attack,
            decay,
            sustain,
            release,
            sample_rate: 44100.0,
            value: 0.0,
            stage: 0,
            last_gate: 0.0,
        }
    }
}

impl AudioNode for SharedAdsr {
    const ID: u64 = 0x534841445352; // "SHADSR"
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.value = 0.0;
        self.stage = 0;
        self.last_gate = 0.0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate as f32;
    }

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let gate = input[0];
        
        let dt = 1.0 / self.sample_rate;

        // Gate transition logic
        if gate > 0.0 && self.last_gate <= 0.0 {
            self.stage = 1; // Attack
            // Lock in parameters for the new note
            self.active_attack = self.attack.value().max(0.001);
            self.active_decay = self.decay.value().max(0.001);
            self.active_sustain = self.sustain.value().clamp(0.0, 1.0);
            self.active_release = self.release.value().max(0.001);
        } else if gate <= 0.0 && self.last_gate > 0.0 {
            self.stage = 4; // Release
        }
        self.last_gate = gate;

        let a = self.active_attack;
        let d = self.active_decay;
        let s = self.active_sustain;
        let r = self.active_release;

        match self.stage {
            1 => { // Attack: linear ramp from 0 to 1
                self.value += dt / a;
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.stage = 2; // Decay
                }
            }
            2 => { // Decay: linear ramp from 1 to sustain level
                self.value -= dt / d * (1.0 - s);
                if self.value <= s {
                    self.value = s;
                    self.stage = 3; // Sustain
                }
            }
            3 => { // Sustain: hold level until gate is released
                self.value = s;
            }
            4 => { // Release: linear ramp from current level to 0
                self.value -= dt / r;
                if self.value <= 0.0 {
                    self.value = 0.0;
                    self.stage = 0; // Idle
                }
            }
            _ => {
                self.value = 0.0;
            }
        }

        [self.value].into()
    }
}

/// An efficient oscillator that can switch waveforms without branching the graph.
#[derive(Clone)]
struct SharedOscillator {
    phase: f32,
    sample_rate: f32,
    last_gate: f32,
    waveform: Shared, // 0: Sine, 1: Triangle, 2: Saw, 3: Square
}

impl SharedOscillator {
    fn new(waveform: Shared) -> Self {
        Self {
            phase: 0.0,
            sample_rate: 44100.0,
            last_gate: 0.0,
            waveform,
        }
    }
}

impl AudioNode for SharedOscillator {
    const ID: u64 = 0x53484f5343; // "SHOSC"
    type Inputs = U2; // [Frequency, Gate]
    type Outputs = U1;

    fn reset(&mut self) {
        self.phase = 0.0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate as f32;
    }

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let freq = input[0];
        let gate = input[1];
        
        // Reset phase on Note On to prevent clicks and ensure consistent transient
        if gate > 0.0 && self.last_gate <= 0.0 {
            self.phase = 0.0;
        }
        self.last_gate = gate;

        let dt = 1.0 / self.sample_rate;
        self.phase = (self.phase + freq * dt).fract();

        let v = match self.waveform.value() as i32 {
            0 => (self.phase * std::f32::consts::TAU).sin(), // Sine
            1 => { // Triangle
                let x = self.phase * 4.0;
                if x < 1.0 { x }
                else if x < 3.0 { 2.0 - x }
                else { x - 4.0 }
            }
            2 => self.phase * 2.0 - 1.0, // Saw
            3 => if self.phase < 0.5 { 1.0 } else { -1.0 }, // Square
            _ => (self.phase * std::f32::consts::TAU).sin(),
        };

        [v].into()
    }
}

/// Handles for a single voice.
pub struct VoiceHandles {
    pub freq: Shared,
    pub gate: Shared,
}

/// All shared handles for the synth.
pub struct SynthHandles {
    pub voices: [VoiceHandles; 5],
    pub fm_index: Shared,
    pub volume: Shared,
    pub attack: Shared,
    pub decay: Shared,
    pub sustain: Shared,
    pub release: Shared,
    pub waveform_select: Shared,
    pub lfo_rate: Shared,
}

/// Build the 5-voice polyphonic synthesis graph.
pub fn build_synth_graph() -> (Box<dyn AudioUnit>, SynthHandles) {
    let fm_index = shared(2.0);
    let volume = shared(0.5);
    let waveform_select = shared(0.0);
    let lfo_rate = shared(0.5); // 0.1Hz - 10Hz
    let attack = shared(0.03);
    let decay = shared(0.1);
    let sustain = shared(0.5);
    let release = shared(0.1);

    let mut voices_handles = Vec::with_capacity(5);
    
    // Initialize 5 sets of per-voice handles.
    let f0 = shared(440.0); let g0 = shared(0.0);
    let f1 = shared(440.0); let g1 = shared(0.0);
    let f2 = shared(440.0); let g2 = shared(0.0);
    let f3 = shared(440.0); let g3 = shared(0.0);
    let f4 = shared(440.0); let g4 = shared(0.0);

    // Global LFO: modulates the FM Index for a growling effect
    // We map the 0.0-1.0 knob to 0.1Hz - 20.0Hz
    let lfo = (var(&lfo_rate) >> map(|r| 0.1 + r[0] * 19.9)) >> sine();
    let lfo_mod = (lfo * 2.0) + var(&fm_index);

    // Build the graph using the voices.
    macro_rules! create_voice_graph_lfo {
        ($f:expr, $g:expr) => {{
            let s_freq = var(&$f) >> follow(0.01);
            let s_gate = var(&$g);
            // Each voice now takes the global lfo_mod into account
            let s_fm_index = lfo_mod.clone() >> follow(0.01);
            
            let env = s_gate.clone() >> An(SharedAdsr::new(
                attack.clone(),
                decay.clone(),
                sustain.clone(),
                release.clone(),
            ));

            // Modulator: always a sine for classic FM
            let modulator = (s_freq.clone() * s_fm_index) >> sine();
            
            // Carrier: Using our efficient multi-waveform oscillator
            // It now takes frequency and gate as inputs [freq, gate]
            let carrier = ((s_freq + modulator * 200.0) | s_gate) >> An(SharedOscillator::new(waveform_select.clone()));
            
            carrier * env
        }}
    }

    let graph = (
        create_voice_graph_lfo!(f0, g0) +
        create_voice_graph_lfo!(f1, g1) +
        create_voice_graph_lfo!(f2, g2) +
        create_voice_graph_lfo!(f3, g3) +
        create_voice_graph_lfo!(f4, g4)
    ) >> mul(0.2) >> (pass() * (var(&volume) >> follow(0.1)));

    voices_handles.push(VoiceHandles { freq: f0, gate: g0 });
    voices_handles.push(VoiceHandles { freq: f1, gate: g1 });
    voices_handles.push(VoiceHandles { freq: f2, gate: g2 });
    voices_handles.push(VoiceHandles { freq: f3, gate: g3 });
    voices_handles.push(VoiceHandles { freq: f4, gate: g4 });

    let handles = SynthHandles {
        voices: voices_handles.try_into().ok().expect("Failed to convert handles"),
        fm_index,
        volume,
        attack,
        decay,
        sustain,
        release,
        waveform_select,
        lfo_rate,
    };

    (Box::new(graph), handles)
}
