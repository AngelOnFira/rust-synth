// Prototype 2: True Modular Approach
// This approach mimics hardware modular synths with patch cables

use std::collections::HashMap;
use std::f64::consts::PI;

// Core module trait
trait Module: Send {
    fn process(&mut self, inputs: &[f64], outputs: &mut [f64], sample_rate: f64);
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
    fn name(&self) -> &str;
}

// Connection between modules
#[derive(Clone, Debug)]
struct Connection {
    from_module: String,
    from_output: usize,
    to_module: String,
    to_input: usize,
    attenuation: f64, // Cable "resistance" or mixing level
}

// The modular rack
struct ModularRack {
    modules: HashMap<String, Box<dyn Module>>,
    connections: Vec<Connection>,
    buffers: HashMap<String, Vec<f64>>,
}

impl ModularRack {
    fn new() -> Self {
        Self {
            modules: HashMap::new(),
            connections: Vec::new(),
            buffers: HashMap::new(),
        }
    }
    
    fn add_module(&mut self, name: String, module: Box<dyn Module>) {
        let num_outputs = module.num_outputs();
        self.modules.insert(name.clone(), module);
        self.buffers.insert(name, vec![0.0; num_outputs]);
    }
    
    fn patch(&mut self, from: &str, from_out: usize, to: &str, to_in: usize, level: f64) {
        self.connections.push(Connection {
            from_module: from.to_string(),
            from_output: from_out,
            to_module: to.to_string(),
            to_input: to_in,
            attenuation: level,
        });
    }
    
    fn process(&mut self, sample_rate: f64) -> f64 {
        // Clear buffers
        for buffer in self.buffers.values_mut() {
            buffer.fill(0.0);
        }
        
        // Process each module
        for (name, module) in &mut self.modules {
            let num_inputs = module.num_inputs();
            let mut inputs = vec![0.0; num_inputs];
            
            // Gather inputs from connections
            for conn in &self.connections {
                if conn.to_module == *name {
                    if let Some(from_buffer) = self.buffers.get(&conn.from_module) {
                        if conn.from_output < from_buffer.len() && conn.to_input < inputs.len() {
                            inputs[conn.to_input] += from_buffer[conn.from_output] * conn.attenuation;
                        }
                    }
                }
            }
            
            // Process module
            if let Some(outputs) = self.buffers.get_mut(name) {
                module.process(&inputs, outputs, sample_rate);
            }
        }
        
        // Return main output
        self.buffers.get("output")
            .and_then(|buf| buf.get(0))
            .copied()
            .unwrap_or(0.0)
    }
}

// --- Module Implementations ---

// VCO (Voltage Controlled Oscillator)
struct VCO {
    phase: f64,
    base_freq: f64,
    waveform: String,
}

impl VCO {
    fn new(freq: f64, waveform: &str) -> Self {
        Self {
            phase: 0.0,
            base_freq: freq,
            waveform: waveform.to_string(),
        }
    }
}

impl Module for VCO {
    fn process(&mut self, inputs: &[f64], outputs: &mut [f64], sample_rate: f64) {
        // Input 0: V/Oct pitch CV
        // Input 1: FM input
        // Input 2: PWM for square wave
        
        let pitch_cv = inputs.get(0).copied().unwrap_or(0.0);
        let fm_cv = inputs.get(1).copied().unwrap_or(0.0);
        let pwm = inputs.get(2).copied().unwrap_or(0.5);
        
        // Calculate frequency with CV modulation (1V/octave standard)
        let freq = self.base_freq * 2.0_f64.powf(pitch_cv + fm_cv * 0.1);
        
        // Generate waveform
        outputs[0] = match self.waveform.as_str() {
            "sine" => (self.phase * 2.0 * PI).sin(),
            "square" => if self.phase < pwm { 1.0 } else { -1.0 },
            "saw" => self.phase * 2.0 - 1.0,
            "triangle" => {
                if self.phase < 0.5 {
                    self.phase * 4.0 - 1.0
                } else {
                    3.0 - self.phase * 4.0
                }
            }
            _ => 0.0,
        };
        
        // Update phase
        self.phase += freq / sample_rate;
        while self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }
    
    fn num_inputs(&self) -> usize { 3 }
    fn num_outputs(&self) -> usize { 1 }
    fn name(&self) -> &str { "VCO" }
}

// ADSR Envelope Generator
struct ADSR {
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    state: String,
    level: f64,
    time_in_state: f64,
    gate_prev: bool,
}

impl ADSR {
    fn new(a: f64, d: f64, s: f64, r: f64) -> Self {
        Self {
            attack: a,
            decay: d,
            sustain: s,
            release: r,
            state: "idle".to_string(),
            level: 0.0,
            time_in_state: 0.0,
            gate_prev: false,
        }
    }
}

impl Module for ADSR {
    fn process(&mut self, inputs: &[f64], outputs: &mut [f64], sample_rate: f64) {
        let gate = inputs.get(0).copied().unwrap_or(0.0) > 0.5;
        let dt = 1.0 / sample_rate;
        
        // Detect gate changes
        if gate && !self.gate_prev {
            self.state = "attack".to_string();
            self.time_in_state = 0.0;
        } else if !gate && self.gate_prev && self.state != "idle" {
            self.state = "release".to_string();
            self.time_in_state = 0.0;
        }
        
        self.gate_prev = gate;
        
        // Process envelope
        match self.state.as_str() {
            "attack" => {
                self.level = self.time_in_state / self.attack;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.state = "decay".to_string();
                    self.time_in_state = 0.0;
                }
            }
            "decay" => {
                self.level = 1.0 - (1.0 - self.sustain) * (self.time_in_state / self.decay);
                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.state = "sustain".to_string();
                }
            }
            "sustain" => {
                self.level = self.sustain;
            }
            "release" => {
                self.level = self.sustain * (1.0 - self.time_in_state / self.release);
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.state = "idle".to_string();
                }
            }
            _ => {
                self.level = 0.0;
            }
        }
        
        self.time_in_state += dt;
        outputs[0] = self.level;
    }
    
    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
    fn name(&self) -> &str { "ADSR" }
}

// VCA (Voltage Controlled Amplifier)
struct VCA;

impl Module for VCA {
    fn process(&mut self, inputs: &[f64], outputs: &mut [f64], _sample_rate: f64) {
        let signal = inputs.get(0).copied().unwrap_or(0.0);
        let cv = inputs.get(1).copied().unwrap_or(1.0);
        outputs[0] = signal * cv;
    }
    
    fn num_inputs(&self) -> usize { 2 }
    fn num_outputs(&self) -> usize { 1 }
    fn name(&self) -> &str { "VCA" }
}

// VCF (Voltage Controlled Filter)
struct VCF {
    cutoff: f64,
    resonance: f64,
    filter_type: String,
    // State variables for filter
    buf0: f64,
    buf1: f64,
}

impl VCF {
    fn new(cutoff: f64, resonance: f64, filter_type: &str) -> Self {
        Self {
            cutoff,
            resonance,
            filter_type: filter_type.to_string(),
            buf0: 0.0,
            buf1: 0.0,
        }
    }
}

impl Module for VCF {
    fn process(&mut self, inputs: &[f64], outputs: &mut [f64], sample_rate: f64) {
        let input = inputs.get(0).copied().unwrap_or(0.0);
        let cv = inputs.get(1).copied().unwrap_or(0.0);
        
        // Calculate cutoff with CV modulation
        let cutoff = (self.cutoff * 2.0_f64.powf(cv * 2.0)).min(sample_rate * 0.49);
        
        // Simple state variable filter
        let f = 2.0 * (PI * cutoff / sample_rate).sin();
        let q = 1.0 / self.resonance.max(0.5);
        
        let low = self.buf0 + f * self.buf1;
        let high = input - low - q * self.buf1;
        let band = f * high + self.buf1;
        let notch = high + low;
        
        self.buf0 = low;
        self.buf1 = band;
        
        outputs[0] = match self.filter_type.as_str() {
            "lowpass" => low,
            "highpass" => high,
            "bandpass" => band,
            "notch" => notch,
            _ => input,
        };
    }
    
    fn num_inputs(&self) -> usize { 2 }
    fn num_outputs(&self) -> usize { 1 }
    fn name(&self) -> &str { "VCF" }
}

// Noise Generator
struct Noise;

impl Module for Noise {
    fn process(&mut self, _inputs: &[f64], outputs: &mut [f64], _sample_rate: f64) {
        outputs[0] = rand::random::<f64>() * 2.0 - 1.0;
    }
    
    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 1 }
    fn name(&self) -> &str { "Noise" }
}

// Mixer
struct Mixer {
    channels: usize,
}

impl Module for Mixer {
    fn process(&mut self, inputs: &[f64], outputs: &mut [f64], _sample_rate: f64) {
        let mut sum = 0.0;
        for i in 0..self.channels.min(inputs.len()) {
            sum += inputs[i];
        }
        outputs[0] = sum / self.channels as f64;
    }
    
    fn num_inputs(&self) -> usize { self.channels }
    fn num_outputs(&self) -> usize { 1 }
    fn name(&self) -> &str { "Mixer" }
}

// Clock/Trigger Sequencer
struct Clock {
    bpm: f64,
    phase: f64,
    pattern: Vec<bool>,
    step: usize,
}

impl Clock {
    fn new(bpm: f64, pattern: &str) -> Self {
        Self {
            bpm,
            phase: 0.0,
            pattern: pattern.chars().map(|c| c == 'x' || c == 'X').collect(),
            step: 0,
        }
    }
}

impl Module for Clock {
    fn process(&mut self, _inputs: &[f64], outputs: &mut [f64], sample_rate: f64) {
        let step_duration = 60.0 / self.bpm / 4.0;
        
        // Check if we've crossed a step boundary
        let prev_step = self.step;
        self.step = (self.phase / step_duration) as usize % self.pattern.len();
        
        // Output gate if pattern has a hit
        outputs[0] = if self.pattern[self.step] { 1.0 } else { 0.0 };
        
        // Output trigger on step change
        outputs[1] = if self.step != prev_step && self.pattern[self.step] { 1.0 } else { 0.0 };
        
        self.phase += 1.0 / sample_rate;
        if self.phase >= step_duration * self.pattern.len() as f64 {
            self.phase -= step_duration * self.pattern.len() as f64;
        }
    }
    
    fn num_inputs(&self) -> usize { 0 }
    fn num_outputs(&self) -> usize { 2 } // Gate and Trigger
    fn name(&self) -> &str { "Clock" }
}

// Output module
struct Output;

impl Module for Output {
    fn process(&mut self, inputs: &[f64], outputs: &mut [f64], _sample_rate: f64) {
        outputs[0] = inputs.get(0).copied().unwrap_or(0.0);
    }
    
    fn num_inputs(&self) -> usize { 1 }
    fn num_outputs(&self) -> usize { 1 }
    fn name(&self) -> &str { "Output" }
}

// Patch creation functions
fn create_kick_patch(rack: &mut ModularRack) {
    // Add modules
    rack.add_module("kick_clock".to_string(), Box::new(Clock::new(120.0, "x...x...x...x...")));
    rack.add_module("kick_env".to_string(), Box::new(ADSR::new(0.001, 0.3, 0.0, 0.0)));
    rack.add_module("kick_pitch_env".to_string(), Box::new(ADSR::new(0.001, 0.05, 0.0, 0.0)));
    rack.add_module("kick_vco".to_string(), Box::new(VCO::new(50.0, "sine")));
    rack.add_module("kick_vca".to_string(), Box::new(VCA));
    
    // Patch cables
    rack.patch("kick_clock", 0, "kick_env", 0, 1.0); // Clock gate to envelope
    rack.patch("kick_clock", 0, "kick_pitch_env", 0, 1.0); // Clock gate to pitch envelope
    rack.patch("kick_pitch_env", 0, "kick_vco", 0, 2.0); // Pitch envelope to VCO pitch
    rack.patch("kick_vco", 0, "kick_vca", 0, 1.0); // VCO to VCA signal
    rack.patch("kick_env", 0, "kick_vca", 1, 1.0); // Envelope to VCA CV
}

fn create_snare_patch(rack: &mut ModularRack) {
    // Add modules
    rack.add_module("snare_clock".to_string(), Box::new(Clock::new(120.0, "....x.......x...")));
    rack.add_module("snare_env".to_string(), Box::new(ADSR::new(0.001, 0.08, 0.0, 0.0)));
    rack.add_module("snare_noise".to_string(), Box::new(Noise));
    rack.add_module("snare_osc1".to_string(), Box::new(VCO::new(200.0, "triangle")));
    rack.add_module("snare_osc2".to_string(), Box::new(VCO::new(340.0, "triangle")));
    rack.add_module("snare_mixer".to_string(), Box::new(Mixer { channels: 3 }));
    rack.add_module("snare_filter".to_string(), Box::new(VCF::new(1500.0, 2.0, "bandpass")));
    rack.add_module("snare_vca".to_string(), Box::new(VCA));
    
    // Patch cables
    rack.patch("snare_clock", 0, "snare_env", 0, 1.0);
    rack.patch("snare_noise", 0, "snare_mixer", 0, 0.6);
    rack.patch("snare_osc1", 0, "snare_mixer", 1, 0.2);
    rack.patch("snare_osc2", 0, "snare_mixer", 2, 0.2);
    rack.patch("snare_mixer", 0, "snare_filter", 0, 1.0);
    rack.patch("snare_filter", 0, "snare_vca", 0, 1.0);
    rack.patch("snare_env", 0, "snare_vca", 1, 1.0);
}

fn main() {
    println!("=== True Modular Synthesis (Patch Cable Style) ===\n");
    
    // Create the rack
    let mut rack = ModularRack::new();
    
    // Create patches
    create_kick_patch(&mut rack);
    create_snare_patch(&mut rack);
    
    // Add main mixer and output
    rack.add_module("main_mixer".to_string(), Box::new(Mixer { channels: 2 }));
    rack.add_module("output".to_string(), Box::new(Output));
    
    // Patch to main mixer
    rack.patch("kick_vca", 0, "main_mixer", 0, 0.8);
    rack.patch("snare_vca", 0, "main_mixer", 1, 0.6);
    rack.patch("main_mixer", 0, "output", 0, 1.0);
    
    // Print patch info
    println!("Modules in rack:");
    for (name, module) in &rack.modules {
        println!("  - {} ({} inputs, {} outputs)", name, module.num_inputs(), module.num_outputs());
    }
    
    println!("\nPatch cables:");
    for conn in &rack.connections {
        println!("  - {}[{}] -> {}[{}] (level: {})", 
            conn.from_module, conn.from_output, 
            conn.to_module, conn.to_input, conn.attenuation);
    }
    
    // Render audio
    let sample_rate = 44100.0;
    let duration = 4.0;
    let num_samples = (sample_rate * duration) as usize;
    let mut output = Vec::with_capacity(num_samples);
    
    println!("\nRendering {} seconds of audio...", duration);
    for _ in 0..num_samples {
        output.push(rack.process(sample_rate));
    }
    
    // Write to file
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create("prototype2_modular.wav", spec).unwrap();
    for &sample in &output {
        writer.write_sample((sample.clamp(-1.0, 1.0) * 32767.0) as i16).unwrap();
    }
    writer.finalize().unwrap();
    
    println!("\nOutput written to: prototype2_modular.wav");
    println!("\nThis approach mimics hardware modular synths with virtual patch cables.");
    println!("Pros: Very flexible, educational, true to hardware modular");
    println!("Cons: More complex to set up, requires understanding of modular routing");
}

// Add to Cargo.toml dependencies:
// rand = "0.8"