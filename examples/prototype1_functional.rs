// Prototype 1: Functional/Compositional Approach
// This approach uses function composition to build complex sounds from simple atoms

use fundsp::prelude::*;
use hound::WavWriter;

// Complex instruments built from atoms
fn kick_drum() -> impl AudioUnit + Clone {
    // Body: sine with pitch envelope
    let pitch_env = envelope(|t: f64| {
        let sweep_time = 0.05;
        if t < sweep_time {
            200.0 * (50.0_f64 / 200.0).powf(t / sweep_time)
        } else {
            50.0
        }
    });
    let body = sine::<f64>() * pitch_env * envelope(|t: f64| (-t / 0.5).exp());
    
    // Click: filtered noise burst
    let click = white() >> highpass_hz(3000.0, 1.0) * envelope(|t: f64| if t < 0.003 { 1.0 } else { 0.0 });
    
    // Sub: extra low frequency content
    let sub = sine_hz::<f64>(30.0) * envelope(|t: f64| (-t / 0.6).exp()) * 0.3;
    
    (body + click + sub) * 0.8
}

fn snare_drum() -> impl AudioUnit + Clone {
    // Tone: two triangle oscillators
    let tone1 = triangle_hz(200.0) * envelope(|t: f64| (-t / 0.15).exp());
    let tone2 = triangle_hz(340.0) * envelope(|t: f64| (-t / 0.15).exp());
    let tone = (tone1 + tone2) * 0.5;
    
    // Noise: filtered white noise
    let noise = white() >> bandpass_hz(500.0, 2.0) * envelope(|t: f64| (-t / 0.08).exp());
    
    (tone * 0.4 + noise * 0.6) * 0.7
}

fn hihat_closed() -> impl AudioUnit + Clone {
    // Metallic noise: mix of high square waves and noise
    let metallic = (square_hz(8000.0) + square_hz(12000.0)) * 0.3;
    let noise = white() >> highpass_hz(8000.0, 2.0);
    
    (metallic + noise) * envelope(|t: f64| (-t / 0.03).exp()) * 0.5
}

fn hihat_open() -> impl AudioUnit + Clone {
    // Similar to closed but longer decay
    let metallic = (square_hz(8000.0) + square_hz(12000.0)) * 0.3;
    let noise = white() >> highpass_hz(6000.0, 1.5);
    
    (metallic + noise) * envelope(|t: f64| (-t / 0.2).exp()) * 0.4
}

// Bass synth with filter modulation
fn bass_synth(note_freq: f32) -> impl AudioUnit + Clone {
    // Two detuned saws for fatness
    let osc1 = saw_hz(note_freq);
    let osc2 = saw_hz(note_freq * 1.01); // Slight detune
    
    // Simple amplitude envelope
    (osc1 + osc2) * 0.5 * adsr_live(0.01, 0.1, 0.7, 0.2) * 0.7
}

// Pattern sequencer
fn play_pattern<T: AudioUnit + Clone + 'static>(pattern: &str, instrument: T, bpm: f64) -> Vec<f32> {
    let step_duration = 60.0 / bpm / 4.0;
    let total_samples = (pattern.len() as f64 * step_duration * 44100.0) as usize;
    let mut output = vec![0.0f32; total_samples];
    
    for (i, ch) in pattern.chars().enumerate() {
        if ch == 'x' || ch == 'X' {
            let start_sample = (i as f64 * step_duration * 44100.0) as usize;
            let mut inst_instance = instrument.clone();
            
            // Render this hit
            for j in 0..std::cmp::min((step_duration * 44100.0 * 2.0) as usize, total_samples - start_sample) {
                let mut out = [0.0];
                inst_instance.tick(&[], &mut out);
                output[start_sample + j] += out[0];
            }
        }
    }
    
    output
}

fn main() {
    println!("=== Functional/Compositional Drum Synthesis ===\n");
    
    let bpm = 120.0;
    let duration = 4.0; // seconds
    let sample_rate = 44100;
    
    // Define patterns
    let kick_pattern = "x...x...x...x...";
    let snare_pattern = "....x.......x...";
    let hihat_pattern = "..x...x...x...x.";
    
    println!("Patterns:");
    println!("Kick:  {}", kick_pattern);
    println!("Snare: {}", snare_pattern);
    println!("Hihat: {}", hihat_pattern);
    
    // Generate individual tracks
    println!("\nGenerating drums...");
    let kick_track = play_pattern(kick_pattern, kick_drum(), bpm);
    let snare_track = play_pattern(snare_pattern, snare_drum(), bpm);
    let hihat_track = play_pattern(hihat_pattern, hihat_closed(), bpm);
    
    // Mix tracks
    let total_samples = (duration * sample_rate as f64) as usize;
    let mut mix = vec![0.0f32; total_samples];
    
    for i in 0..std::cmp::min(kick_track.len(), total_samples) {
        mix[i] += kick_track[i] + snare_track[i] + hihat_track[i];
    }
    
    // Generate bass line
    println!("Generating bass...");
    let bass_notes = [65.41f32, 0.0, 82.41, 0.0, 98.0, 0.0, 87.31, 0.0]; // C2, E2, G2, F2
    let note_duration = 60.0 / bpm / 2.0; // 8th notes
    
    for (i, &freq) in bass_notes.iter().cycle().enumerate() {
        let start = i as f64 * note_duration;
        if start > duration { break; }
        
        if freq > 0.0 {
            let mut bass = bass_synth(freq);
            let start_sample = (start * sample_rate as f64) as usize;
            
            for j in 0..std::cmp::min((note_duration * sample_rate as f64 * 2.0) as usize, total_samples - start_sample) {
                let mut out = [0.0];
                bass.tick(&[], &mut out);
                mix[start_sample + j] += out[0] * 0.5;
            }
        }
    }
    
    // Normalize
    let max_val = mix.iter().fold(0.0f32, |max, &x| x.abs().max(max));
    if max_val > 0.0 {
        for sample in &mut mix {
            *sample *= 0.8 / max_val;
        }
    }
    
    // Write to file
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = WavWriter::create("prototype1_functional.wav", spec).unwrap();
    for &sample in &mix {
        writer.write_sample((sample * 32767.0) as i16).unwrap();
    }
    writer.finalize().unwrap();
    
    println!("\nOutput written to: prototype1_functional.wav");
    println!("\nThis approach uses function composition to build sounds from basic atoms.");
    println!("Pros: Simple, functional, easy to understand");
    println!("Cons: Less flexible than true modular routing");
}