use fundsp::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum WaveformType {
    Sine,
    Square,
    Saw,
    Triangle,
    Noise,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Envelope {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.8,
            release: 0.2,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FilterType {
    LowPass { cutoff: f32, resonance: f32 },
    HighPass { cutoff: f32, resonance: f32 },
    BandPass { center: f32, bandwidth: f32 },
}

pub type SynthNode = Box<dyn AudioUnit>;

pub fn create_oscillator(waveform: WaveformType, frequency: f32) -> SynthNode {
    match waveform {
        WaveformType::Sine => Box::new(sine_hz::<f64>(frequency)) as SynthNode,
        WaveformType::Square => Box::new(square_hz(frequency)) as SynthNode,
        WaveformType::Saw => Box::new(saw_hz(frequency)) as SynthNode,
        WaveformType::Triangle => Box::new(triangle_hz(frequency)) as SynthNode,
        WaveformType::Noise => Box::new(white()) as SynthNode,
    }
}

pub fn create_envelope(env: &Envelope) -> SynthNode {
    Box::new(adsr_live(env.attack, env.decay, env.sustain, env.release)) as SynthNode
}

pub fn create_filter(filter: &FilterType) -> SynthNode {
    match filter {
        FilterType::LowPass { cutoff, resonance } => {
            Box::new(lowpass_hz(*cutoff, *resonance)) as SynthNode
        }
        FilterType::HighPass { cutoff, resonance } => {
            Box::new(highpass_hz(*cutoff, *resonance)) as SynthNode
        }
        FilterType::BandPass { center, bandwidth } => {
            Box::new(bandpass_hz(*center, *bandwidth)) as SynthNode
        }
    }
}

pub fn pitch_envelope(start: f32, end: f32, time: f32) -> SynthNode {
    Box::new(envelope(move |t| {
        if t < time {
            start * (end / start).powf(t / time)
        } else {
            end
        }
    })) as SynthNode
}