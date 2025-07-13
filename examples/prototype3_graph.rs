// Prototype 3: Graph-Based Node System
// This approach uses a node graph where each node has inputs/outputs that can be connected

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

// Node input/output types
#[derive(Clone, Debug)]
enum Signal {
    Audio(f64),
    Control(f64),
    Trigger(bool),
}

// Node definition
trait Node: Send + Sync {
    fn process(&mut self, inputs: &HashMap<String, Signal>) -> HashMap<String, Signal>;
    fn get_inputs(&self) -> Vec<String>;
    fn get_outputs(&self) -> Vec<String>;
}

// Graph structure
struct NodeGraph {
    nodes: HashMap<String, Arc<Mutex<Box<dyn Node>>>>,
    connections: Vec<(String, String, String, String)>, // (from_node, from_output, to_node, to_input)
    signals: HashMap<String, Signal>,
}

impl NodeGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            signals: HashMap::new(),
        }
    }
    
    fn add_node(&mut self, name: String, node: Box<dyn Node>) {
        self.nodes.insert(name, Arc::new(Mutex::new(node)));
    }
    
    fn connect(&mut self, from_node: &str, from_output: &str, to_node: &str, to_input: &str) {
        self.connections.push((
            from_node.to_string(),
            from_output.to_string(),
            to_node.to_string(),
            to_input.to_string(),
        ));
    }
    
    fn process(&mut self) -> Option<f64> {
        // Clear signals
        self.signals.clear();
        
        // Process nodes in order (simplified - real implementation would topologically sort)
        let node_names: Vec<String> = self.nodes.keys().cloned().collect();
        
        for node_name in &node_names {
            if let Some(node_arc) = self.nodes.get(node_name) {
                // Collect inputs for this node
                let mut inputs = HashMap::new();
                
                for (from_node, from_output, to_node, to_input) in &self.connections {
                    if to_node == node_name {
                        let signal_key = format!("{}:{}", from_node, from_output);
                        if let Some(signal) = self.signals.get(&signal_key) {
                            inputs.insert(to_input.clone(), signal.clone());
                        }
                    }
                }
                
                // Process node
                let outputs = node_arc.lock().unwrap().process(&inputs);
                
                // Store outputs
                for (output_name, signal) in outputs {
                    let signal_key = format!("{}:{}", node_name, output_name);
                    self.signals.insert(signal_key, signal);
                }
            }
        }
        
        // Get final output
        self.signals.get("output:out")
            .and_then(|s| match s {
                Signal::Audio(v) => Some(*v),
                _ => None,
            })
    }
}

// --- Node Implementations ---

// Oscillator Node
struct OscillatorNode {
    frequency: f64,
    phase: f64,
    waveform: String,
}

impl OscillatorNode {
    fn new(freq: f64, waveform: &str) -> Self {
        Self {
            frequency: freq,
            phase: 0.0,
            waveform: waveform.to_string(),
        }
    }
}

impl Node for OscillatorNode {
    fn process(&mut self, inputs: &HashMap<String, Signal>) -> HashMap<String, Signal> {
        let mut outputs = HashMap::new();
        
        // Get frequency modulation
        let fm = inputs.get("fm")
            .and_then(|s| match s {
                Signal::Control(v) => Some(*v),
                _ => None,
            })
            .unwrap_or(0.0);
        
        let freq = self.frequency * (1.0 + fm);
        
        // Generate waveform
        let sample = match self.waveform.as_str() {
            "sine" => (self.phase * 2.0 * std::f64::consts::PI).sin(),
            "square" => if self.phase < 0.5 { 1.0 } else { -1.0 },
            "saw" => self.phase * 2.0 - 1.0,
            _ => 0.0,
        };
        
        // Update phase
        self.phase += freq / 44100.0;
        while self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        
        outputs.insert("out".to_string(), Signal::Audio(sample));
        outputs
    }
    
    fn get_inputs(&self) -> Vec<String> {
        vec!["fm".to_string()]
    }
    
    fn get_outputs(&self) -> Vec<String> {
        vec!["out".to_string()]
    }
}

// Envelope Node
struct EnvelopeNode {
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
    level: f64,
    stage: String,
    time: f64,
    triggered: bool,
}

impl EnvelopeNode {
    fn new(a: f64, d: f64, s: f64, r: f64) -> Self {
        Self {
            attack: a,
            decay: d,
            sustain: s,
            release: r,
            level: 0.0,
            stage: "idle".to_string(),
            time: 0.0,
            triggered: false,
        }
    }
}

impl Node for EnvelopeNode {
    fn process(&mut self, inputs: &HashMap<String, Signal>) -> HashMap<String, Signal> {
        let mut outputs = HashMap::new();
        
        // Check for trigger
        let trigger = inputs.get("trigger")
            .and_then(|s| match s {
                Signal::Trigger(v) => Some(*v),
                _ => None,
            })
            .unwrap_or(false);
        
        if trigger && !self.triggered {
            self.stage = "attack".to_string();
            self.time = 0.0;
        }
        self.triggered = trigger;
        
        // Process envelope stages
        let dt = 1.0 / 44100.0;
        
        match self.stage.as_str() {
            "attack" => {
                self.level = self.time / self.attack;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = "decay".to_string();
                    self.time = 0.0;
                }
            }
            "decay" => {
                self.level = 1.0 - (1.0 - self.sustain) * (self.time / self.decay);
                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.stage = "sustain".to_string();
                }
            }
            "sustain" => {
                if !trigger {
                    self.stage = "release".to_string();
                    self.time = 0.0;
                }
            }
            "release" => {
                self.level = self.sustain * (1.0 - self.time / self.release);
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = "idle".to_string();
                }
            }
            _ => {}
        }
        
        self.time += dt;
        
        outputs.insert("out".to_string(), Signal::Control(self.level));
        outputs
    }
    
    fn get_inputs(&self) -> Vec<String> {
        vec!["trigger".to_string()]
    }
    
    fn get_outputs(&self) -> Vec<String> {
        vec!["out".to_string()]
    }
}

// Filter Node
struct FilterNode {
    cutoff: f64,
    resonance: f64,
    state1: f64,
    state2: f64,
}

impl FilterNode {
    fn new(cutoff: f64, resonance: f64) -> Self {
        Self {
            cutoff,
            resonance,
            state1: 0.0,
            state2: 0.0,
        }
    }
}

impl Node for FilterNode {
    fn process(&mut self, inputs: &HashMap<String, Signal>) -> HashMap<String, Signal> {
        let mut outputs = HashMap::new();
        
        let input = inputs.get("in")
            .and_then(|s| match s {
                Signal::Audio(v) => Some(*v),
                _ => None,
            })
            .unwrap_or(0.0);
        
        let cutoff_cv = inputs.get("cutoff")
            .and_then(|s| match s {
                Signal::Control(v) => Some(*v),
                _ => None,
            })
            .unwrap_or(0.0);
        
        // Calculate filter coefficients
        let cutoff = self.cutoff * (1.0 + cutoff_cv * 2.0);
        let f = 2.0 * (std::f64::consts::PI * cutoff / 44100.0).sin();
        let q = 1.0 / self.resonance.max(0.5);
        
        // State variable filter
        let low = self.state2 + f * self.state1;
        let high = input - low - q * self.state1;
        let band = f * high + self.state1;
        
        self.state1 = band;
        self.state2 = low;
        
        outputs.insert("out".to_string(), Signal::Audio(low));
        outputs
    }
    
    fn get_inputs(&self) -> Vec<String> {
        vec!["in".to_string(), "cutoff".to_string()]
    }
    
    fn get_outputs(&self) -> Vec<String> {
        vec!["out".to_string()]
    }
}

// VCA Node
struct VCANode;

impl Node for VCANode {
    fn process(&mut self, inputs: &HashMap<String, Signal>) -> HashMap<String, Signal> {
        let mut outputs = HashMap::new();
        
        let signal = inputs.get("in")
            .and_then(|s| match s {
                Signal::Audio(v) => Some(*v),
                _ => None,
            })
            .unwrap_or(0.0);
        
        let cv = inputs.get("cv")
            .and_then(|s| match s {
                Signal::Control(v) => Some(*v),
                _ => None,
            })
            .unwrap_or(1.0);
        
        outputs.insert("out".to_string(), Signal::Audio(signal * cv));
        outputs
    }
    
    fn get_inputs(&self) -> Vec<String> {
        vec!["in".to_string(), "cv".to_string()]
    }
    
    fn get_outputs(&self) -> Vec<String> {
        vec!["out".to_string()]
    }
}

// Mixer Node
struct MixerNode {
    channels: usize,
}

impl Node for MixerNode {
    fn process(&mut self, inputs: &HashMap<String, Signal>) -> HashMap<String, Signal> {
        let mut outputs = HashMap::new();
        let mut sum = 0.0;
        
        for i in 0..self.channels {
            let input_name = format!("in{}", i + 1);
            if let Some(Signal::Audio(v)) = inputs.get(&input_name) {
                sum += v;
            }
        }
        
        outputs.insert("out".to_string(), Signal::Audio(sum / self.channels as f64));
        outputs
    }
    
    fn get_inputs(&self) -> Vec<String> {
        (1..=self.channels).map(|i| format!("in{}", i)).collect()
    }
    
    fn get_outputs(&self) -> Vec<String> {
        vec!["out".to_string()]
    }
}

// Sequencer Node
struct SequencerNode {
    pattern: Vec<bool>,
    step: usize,
    time: f64,
    bpm: f64,
}

impl SequencerNode {
    fn new(pattern: &str, bpm: f64) -> Self {
        Self {
            pattern: pattern.chars().map(|c| c == 'x' || c == 'X').collect(),
            step: 0,
            time: 0.0,
            bpm,
        }
    }
}

impl Node for SequencerNode {
    fn process(&mut self, _inputs: &HashMap<String, Signal>) -> HashMap<String, Signal> {
        let mut outputs = HashMap::new();
        
        let step_duration = 60.0 / self.bpm / 4.0;
        let prev_step = self.step;
        
        self.step = (self.time / step_duration) as usize % self.pattern.len();
        
        let trigger = self.step != prev_step && self.pattern[self.step];
        
        self.time += 1.0 / 44100.0;
        
        outputs.insert("trigger".to_string(), Signal::Trigger(trigger));
        outputs
    }
    
    fn get_inputs(&self) -> Vec<String> {
        vec![]
    }
    
    fn get_outputs(&self) -> Vec<String> {
        vec!["trigger".to_string()]
    }
}

// Output Node
struct OutputNode;

impl Node for OutputNode {
    fn process(&mut self, inputs: &HashMap<String, Signal>) -> HashMap<String, Signal> {
        let mut outputs = HashMap::new();
        
        if let Some(signal) = inputs.get("in") {
            outputs.insert("out".to_string(), signal.clone());
        }
        
        outputs
    }
    
    fn get_inputs(&self) -> Vec<String> {
        vec!["in".to_string()]
    }
    
    fn get_outputs(&self) -> Vec<String> {
        vec!["out".to_string()]
    }
}

// Create drum patches
fn create_kick_graph(graph: &mut NodeGraph) {
    // Add nodes
    graph.add_node("kick_seq".to_string(), Box::new(SequencerNode::new("x...x...x...x...", 120.0)));
    graph.add_node("kick_osc".to_string(), Box::new(OscillatorNode::new(50.0, "sine")));
    graph.add_node("kick_env".to_string(), Box::new(EnvelopeNode::new(0.001, 0.4, 0.0, 0.0)));
    graph.add_node("kick_pitch_env".to_string(), Box::new(EnvelopeNode::new(0.001, 0.05, 0.0, 0.0)));
    graph.add_node("kick_vca".to_string(), Box::new(VCANode));
    
    // Connect nodes
    graph.connect("kick_seq", "trigger", "kick_env", "trigger");
    graph.connect("kick_seq", "trigger", "kick_pitch_env", "trigger");
    graph.connect("kick_pitch_env", "out", "kick_osc", "fm");
    graph.connect("kick_osc", "out", "kick_vca", "in");
    graph.connect("kick_env", "out", "kick_vca", "cv");
}

fn create_snare_graph(graph: &mut NodeGraph) {
    // Add nodes
    graph.add_node("snare_seq".to_string(), Box::new(SequencerNode::new("....x.......x...", 120.0)));
    graph.add_node("snare_osc1".to_string(), Box::new(OscillatorNode::new(200.0, "sine")));
    graph.add_node("snare_osc2".to_string(), Box::new(OscillatorNode::new(340.0, "sine")));
    graph.add_node("snare_env".to_string(), Box::new(EnvelopeNode::new(0.001, 0.08, 0.0, 0.0)));
    graph.add_node("snare_mixer".to_string(), Box::new(MixerNode { channels: 2 }));
    graph.add_node("snare_filter".to_string(), Box::new(FilterNode::new(1500.0, 2.0)));
    graph.add_node("snare_vca".to_string(), Box::new(VCANode));
    
    // Connect nodes
    graph.connect("snare_seq", "trigger", "snare_env", "trigger");
    graph.connect("snare_osc1", "out", "snare_mixer", "in1");
    graph.connect("snare_osc2", "out", "snare_mixer", "in2");
    graph.connect("snare_mixer", "out", "snare_filter", "in");
    graph.connect("snare_filter", "out", "snare_vca", "in");
    graph.connect("snare_env", "out", "snare_vca", "cv");
}

fn main() {
    println!("=== Graph-Based Node System ===\n");
    
    // Create graph
    let mut graph = NodeGraph::new();
    
    // Build drum patches
    create_kick_graph(&mut graph);
    create_snare_graph(&mut graph);
    
    // Add main mixer and output
    graph.add_node("main_mixer".to_string(), Box::new(MixerNode { channels: 2 }));
    graph.add_node("output".to_string(), Box::new(OutputNode));
    
    // Connect to main mixer
    graph.connect("kick_vca", "out", "main_mixer", "in1");
    graph.connect("snare_vca", "out", "main_mixer", "in2");
    graph.connect("main_mixer", "out", "output", "in");
    
    // Print graph info
    println!("Nodes in graph:");
    for (name, node) in &graph.nodes {
        let n = node.lock().unwrap();
        println!("  - {} (inputs: {:?}, outputs: {:?})", 
            name, n.get_inputs(), n.get_outputs());
    }
    
    println!("\nConnections:");
    for (from_node, from_output, to_node, to_input) in &graph.connections {
        println!("  - {}:{} -> {}:{}", from_node, from_output, to_node, to_input);
    }
    
    // Render audio
    let sample_rate = 44100;
    let duration = 4.0;
    let num_samples = (sample_rate as f64 * duration) as usize;
    let mut output = Vec::with_capacity(num_samples);
    
    println!("\nRendering {} seconds of audio...", duration);
    for _ in 0..num_samples {
        output.push(graph.process().unwrap_or(0.0));
    }
    
    // Write to file
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create("prototype3_graph.wav", spec).unwrap();
    for &sample in &output {
        writer.write_sample((sample.clamp(-1.0, 1.0) * 32767.0) as i16).unwrap();
    }
    writer.finalize().unwrap();
    
    println!("\nOutput written to: prototype3_graph.wav");
    println!("\nThis approach uses a node graph with typed inputs/outputs.");
    println!("Pros: Type-safe connections, visual programming friendly");
    println!("Cons: More boilerplate, need to handle signal types");
}