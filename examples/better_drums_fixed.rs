// Better Drum Synthesis Examples
// This shows how to create more realistic drum sounds using synthesis techniques

use fundsp::prelude::*;
use hound::WavWriter;

// Professional Kick Drum
// Components: pitch-swept sine, click transient, sub-bass layer
fn pro_kick() -> Box<dyn AudioUnit> {
    // Main body: sine with pitch envelope
    // Starts at 200Hz and sweeps down to 50Hz
    let pitch_env = envelope(|t: f64| {
        let sweep_time = 0.08;
        if t < sweep_time {
            200.0 * (50.0_f64 / 200.0).powf(t / sweep_time)
        } else {
            50.0
        }
    });
    let body = sine::<f64>() * pitch_env;
    
    // Click: Short burst of filtered noise for punch
    let click = white() 
        >> highpass_hz(5000.0, 1.0) 
        * envelope(|t: f64| if t < 0.002 { 1.0 - t / 0.002 } else { 0.0 })
        * 0.15;
    
    // Sub bass: Extra low sine for weight
    let sub = sine_hz(25.0) * envelope(|t: f64| (-t / 0.8).exp()) * 0.2;
    
    // Amplitude envelope
    let amp_env = envelope(|t: f64| {
        let attack = 0.002;
        let decay = 0.5;
        if t < attack {
            t / attack
        } else {
            ((attack - t) / decay).exp()
        }
    });
    
    Box::new((body + click + sub) * amp_env * 0.9)
}

// Professional Snare Drum
// Components: tonal body (200Hz + 340Hz), filtered noise, resonant modes
fn pro_snare() -> Box<dyn AudioUnit> {
    // Tonal component: fundamental frequencies of snare
    let tone1 = triangle_hz(200.0);
    let tone2 = triangle_hz(340.0);
    let tone3 = sine_hz::<f64>(520.0) * 0.3; // Higher harmonic
    
    // Tone envelope - slightly longer than noise
    let tone_env = envelope(|t: f64| {
        if t < 0.002 {
            t / 0.002
        } else {
            (-t / 0.15).exp()
        }
    });
    
    let tones = (tone1 + tone2 + tone3) * tone_env * 0.3;
    
    // Noise component: bandpassed white noise
    let noise_env = envelope(|t: f64| {
        if t < 0.001 {
            t / 0.001
        } else {
            (-t / 0.08).exp()
        }
    });
    
    let noise = white() 
        >> bandpass_hz(400.0, 0.5)  // Body resonance
        >> highpass_hz(200.0, 1.0)  // Remove low rumble
        * noise_env * 0.7;
    
    // Snare rattle: higher frequency noise
    let rattle = white()
        >> bandpass_hz(1500.0, 2.0)
        * envelope(|t: f64| (-t / 0.05).exp())
        * 0.2;
    
    Box::new((tones + noise + rattle) * 0.8)
}

// Professional Hi-Hat (Closed)
// Components: metallic noise, FM synthesis, high-pass filtering
fn pro_hihat_closed() -> Box<dyn AudioUnit> {
    // FM synthesis for metallic sound
    let modulator = sine_hz::<f64>(1823.0);
    let carrier = sine_hz::<f64>(5431.0) * (constant(1.0) + modulator * 0.5);
    
    // Mix with filtered noise
    let noise = white() >> highpass_hz(8000.0, 2.0);
    
    // Very short envelope
    let env = envelope(|t: f64| {
        if t < 0.002 {
            t / 0.002
        } else {
            (-t / 0.03).exp()
        }
    });
    
    Box::new((carrier * 0.3 + noise * 0.7) * env * 0.5)
}

// Professional Hi-Hat (Open)
// Similar to closed but with longer decay and different filtering
fn pro_hihat_open() -> Box<dyn AudioUnit> {
    // Multiple FM operators for complex metallic sound
    let mod1 = sine_hz::<f64>(1823.0);
    let mod2 = sine_hz::<f64>(3671.0);
    let carrier1 = sine_hz::<f64>(5431.0) * (constant(1.0) + mod1 * 0.3);
    let carrier2 = sine_hz::<f64>(7919.0) * (constant(1.0) + mod2 * 0.2);
    
    // Filtered noise with resonance
    let noise = white() 
        >> highpass_hz(6000.0, 1.5)
        >> bandpass_hz(10000.0, 3.0);
    
    // Longer envelope with slight attack
    let env = envelope(|t: f64| {
        let attack = 0.005;
        let decay = 0.3;
        if t < attack {
            t / attack
        } else {
            ((attack - t) / decay).exp()
        }
    });
    
    Box::new((carrier1 * 0.2 + carrier2 * 0.2 + noise * 0.6) * env * 0.4)
}

// 808-style Bass Drum
// Long, sustained sine wave with pitch envelope
fn bass_808() -> Box<dyn AudioUnit> {
    let pitch_env = envelope(|t: f64| {
        let sweep_time = 0.1;
        if t < sweep_time {
            120.0 * (35.0_f64 / 120.0).powf(t / sweep_time)
        } else {
            35.0
        }
    });
    
    let amp_env = envelope(|t: f64| {
        let attack = 0.01;
        let decay = 1.5;
        if t < attack {
            t / attack
        } else {
            ((attack - t) / decay).exp() * 0.8
        }
    });
    
    Box::new(sine::<f64>() * pitch_env * amp_env)
}

// Clap sound (multiple short bursts)
fn pro_clap() -> Box<dyn AudioUnit> {
    // Multiple envelope peaks to simulate multiple hands
    let clap_env = envelope(|t: f64| {
        if t < 0.01 { 0.7 }
        else if t < 0.02 { 0.0 }
        else if t < 0.03 { 1.0 }
        else if t < 0.04 { 0.0 }
        else if t < 0.05 { 0.8 }
        else { (-(t - 0.05) / 0.03).exp() }
    });
    
    let noise = white() 
        >> bandpass_hz(1100.0, 0.6)
        >> highpass_hz(600.0, 1.0);
    
    Box::new(noise * clap_env * 0.7)
}

// Demo function to play patterns
fn render_pattern(pattern: &str, instrument: Box<dyn AudioUnit>, bpm: f32, duration: f32) -> Vec<f32> {
    let sample_rate = 44100.0;
    let step_duration = 60.0 / bpm / 4.0;
    let total_samples = (duration * sample_rate) as usize;
    let mut output = vec![0.0f32; total_samples];
    
    for (i, ch) in pattern.chars().cycle().enumerate() {
        let time = i as f32 * step_duration;
        if time > duration { break; }
        
        if ch == 'x' || ch == 'X' {
            let start_sample = (time * sample_rate) as usize;
            let mut inst = instrument.clone();
            
            for j in 0..std::cmp::min((step_duration * sample_rate * 4.0) as usize, total_samples - start_sample) {
                let mut out = [0.0];
                inst.tick(&[], &mut out);
                output[start_sample + j] += out[0];
            }
        }
    }
    
    output
}

fn main() {
    println!("=== Better Drum Synthesis Examples ===\n");
    
    let bpm = 120.0;
    let duration = 8.0;
    let sample_rate = 44100;
    
    // Create individual drum tracks
    println!("Rendering drums with professional synthesis techniques...\n");
    
    let patterns = vec![
        ("Kick    ", "x...x...x...x...", pro_kick()),
        ("Snare   ", "....x.......x...", pro_snare()),
        ("Hi-hat C", "x.x.x.x.x.x.x.x.", pro_hihat_closed()),
        ("Hi-hat O", "..............x.", pro_hihat_open()),
        ("808 Bass", "x...............", bass_808()),
        ("Clap    ", "....x.......x...", pro_clap()),
    ];
    
    // Render and mix all tracks
    let mut mix = vec![0.0f32; (duration * sample_rate as f32) as usize];
    
    for (name, pattern, instrument) in patterns {
        println!("{}: {}", name, pattern);
        let track = render_pattern(pattern, instrument, bpm, duration);
        
        // Mix with slight panning (stereo width)
        for (i, &sample) in track.iter().enumerate() {
            if i < mix.len() {
                mix[i] += sample;
            }
        }
    }
    
    // Normalize mix
    let max_val = mix.iter().fold(0.0f32, |max, &x| x.abs().max(max));
    if max_val > 0.0 {
        for sample in &mut mix {
            *sample *= 0.8 / max_val;
        }
    }
    
    // Write individual drum demos
    println!("\nWriting individual drum samples...");
    
    let drums = vec![
        ("kick_pro.wav", pro_kick()),
        ("snare_pro.wav", pro_snare()),
        ("hihat_closed_pro.wav", pro_hihat_closed()),
        ("hihat_open_pro.wav", pro_hihat_open()),
        ("808_bass.wav", bass_808()),
        ("clap_pro.wav", pro_clap()),
    ];
    
    for (filename, mut drum) in drums {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: sample_rate as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        
        let mut writer = WavWriter::create(filename, spec).unwrap();
        
        // Render 1 second of the drum hit
        for _ in 0..(sample_rate as usize) {
            let mut out = [0.0];
            drum.tick(&[], &mut out);
            writer.write_sample((out[0] * 32767.0) as i16).unwrap();
        }
        
        writer.finalize().unwrap();
        println!("  - {}", filename);
    }
    
    // Write full mix
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = WavWriter::create("better_drums_mix.wav", spec).unwrap();
    for &sample in &mix {
        writer.write_sample((sample * 32767.0) as i16).unwrap();
    }
    writer.finalize().unwrap();
    
    println!("\nOutput files:");
    println!("  - better_drums_mix.wav (full pattern)");
    println!("  - Individual drum hits for testing\n");
    
    println!("Key improvements over basic synthesis:");
    println!("  - Kick: Pitch envelope, click transient, sub-bass layer");
    println!("  - Snare: Multiple tonal components, separate envelopes");
    println!("  - Hi-hat: FM synthesis for metallic character");
    println!("  - 808: Long sustained bass with pitch sweep");
    println!("  - Clap: Multiple envelope peaks for realism");
}