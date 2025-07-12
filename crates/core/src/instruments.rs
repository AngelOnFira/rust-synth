use crate::atoms::{WaveformType, FilterType, SynthNode, Envelope as SynthEnvelope, create_oscillator};
use fundsp::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentDefinition {
    pub name: String,
    pub oscillator: WaveformType,
    pub envelope: SynthEnvelope,
    pub filter: Option<FilterType>,
    pub amplitude: f32,
}

impl InstrumentDefinition {
    pub fn kick() -> Self {
        Self {
            name: "kick".to_string(),
            oscillator: WaveformType::Sine,
            envelope: SynthEnvelope {
                attack: 0.001,
                decay: 0.5,
                sustain: 0.0,
                release: 0.0,
            },
            filter: None,
            amplitude: 0.8,
        }
    }

    pub fn snare() -> Self {
        Self {
            name: "snare".to_string(),
            oscillator: WaveformType::Noise,
            envelope: SynthEnvelope {
                attack: 0.001,
                decay: 0.15,
                sustain: 0.0,
                release: 0.0,
            },
            filter: Some(FilterType::HighPass {
                cutoff: 2000.0,
                resonance: 1.0,
            }),
            amplitude: 0.6,
        }
    }

    pub fn hihat() -> Self {
        Self {
            name: "hihat".to_string(),
            oscillator: WaveformType::Noise,
            envelope: SynthEnvelope {
                attack: 0.001,
                decay: 0.05,
                sustain: 0.0,
                release: 0.0,
            },
            filter: Some(FilterType::HighPass {
                cutoff: 8000.0,
                resonance: 1.0,
            }),
            amplitude: 0.4,
        }
    }

    pub fn bass() -> Self {
        Self {
            name: "bass".to_string(),
            oscillator: WaveformType::Saw,
            envelope: SynthEnvelope {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.8,
                release: 0.2,
            },
            filter: Some(FilterType::LowPass {
                cutoff: 200.0,
                resonance: 0.3,
            }),
            amplitude: 0.7,
        }
    }
}

pub fn create_instrument_synth(def: &InstrumentDefinition, note_freq: f32) -> SynthNode {
    if def.name == "kick" {
        // Special kick drum synthesis with pitch envelope
        let kick = sine_hz::<f64>(60.0) * envelope(|t| {
            if t < 0.1 {
                (30.0_f64 / 60.0).powf(t / 0.1)
            } else {
                0.5
            }
        }) * adsr_live(def.envelope.attack, def.envelope.decay, def.envelope.sustain, def.envelope.release);
        
        return Box::new(kick * constant(def.amplitude)) as SynthNode;
    }
    
    // General synthesis path
    match (def.oscillator, &def.filter) {
        (WaveformType::Sine, None) => {
            Box::new(sine_hz::<f64>(note_freq) * adsr_live(def.envelope.attack, def.envelope.decay, def.envelope.sustain, def.envelope.release) * constant(def.amplitude)) as SynthNode
        }
        (WaveformType::Saw, Some(FilterType::LowPass { cutoff, resonance })) => {
            Box::new(saw_hz(note_freq) * adsr_live(def.envelope.attack, def.envelope.decay, def.envelope.sustain, def.envelope.release) >> lowpass_hz(*cutoff, *resonance) * constant(def.amplitude)) as SynthNode
        }
        (WaveformType::Noise, Some(FilterType::HighPass { cutoff, resonance })) => {
            Box::new(white() * adsr_live(def.envelope.attack, def.envelope.decay, def.envelope.sustain, def.envelope.release) >> highpass_hz(*cutoff, *resonance) * constant(def.amplitude)) as SynthNode
        }
        _ => {
            // Fallback for other combinations
            create_oscillator(def.oscillator, note_freq)
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstrumentLibrary {
    instruments: HashMap<String, InstrumentDefinition>,
}

impl Default for InstrumentLibrary {
    fn default() -> Self {
        let mut instruments = HashMap::new();
        
        let kick = InstrumentDefinition::kick();
        instruments.insert(kick.name.clone(), kick);
        
        let snare = InstrumentDefinition::snare();
        instruments.insert(snare.name.clone(), snare);
        
        let hihat = InstrumentDefinition::hihat();
        instruments.insert(hihat.name.clone(), hihat);
        
        let bass = InstrumentDefinition::bass();
        instruments.insert(bass.name.clone(), bass);
        
        Self { instruments }
    }
}

impl InstrumentLibrary {
    pub fn get(&self, name: &str) -> Option<&InstrumentDefinition> {
        self.instruments.get(name)
    }
    
    pub fn add(&mut self, def: InstrumentDefinition) {
        self.instruments.insert(def.name.clone(), def);
    }
}