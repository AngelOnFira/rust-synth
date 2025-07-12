use rsynth_core::{Song, InstrumentLibrary, create_instrument_synth};
use hound::{WavWriter, WavSpec};
use fundsp::prelude::*;
use std::path::Path;
use anyhow::Result;
use tracing::info;

pub fn export_song_to_wav<P: AsRef<Path>>(
    song: &Song,
    instruments: &InstrumentLibrary,
    output_path: P,
    duration_seconds: f32,
) -> Result<()> {
    let sample_rate = 44100;
    let channels = 2;
    let bits_per_sample = 16;
    
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = WavWriter::create(&output_path, spec)?;
    
    let total_samples = (sample_rate as f32 * duration_seconds) as usize;
    let step_duration = song.step_duration();
    
    info!("Exporting {} seconds of audio ({} samples)", duration_seconds, total_samples);
    info!("BPM: {}, Step duration: {}s", song.bpm, step_duration);
    
    // For the demo, let's create a simple pattern playback
    for sample_idx in 0..total_samples {
        let time = sample_idx as f32 / sample_rate as f32;
        let step_in_cycle = ((time / step_duration) as usize) % 16;
        
        let mut mix_left = 0.0;
        let mut mix_right = 0.0;
        
        // Simple drum pattern generation
        for track in song.tracks.values() {
            if track.muted {
                continue;
            }
            
            for pattern in &track.patterns {
                if pattern.is_active(step_in_cycle) {
                    // Simple click sound for drums
                    let beat_phase = (time / step_duration).fract();
                    if beat_phase < 0.01 {
                        let click = (1.0 - beat_phase / 0.01) * 0.3;
                        mix_left += click;
                        mix_right += click;
                    }
                }
            }
            
            // Basic melodic pattern support
            for melodic_pattern in &track.melodic_patterns {
                if let Some(note) = melodic_pattern.notes.get(step_in_cycle % melodic_pattern.notes.len()) {
                    if let Some(note) = note {
                        // Simple sine wave for melody
                        let phase = (time * note.frequency() * 2.0 * std::f32::consts::PI).sin();
                        let envelope = ((time / step_duration).fract()).powf(2.0);
                        let sample = phase * envelope * 0.1 * note.velocity;
                        mix_left += sample;
                        mix_right += sample;
                    }
                }
            }
        }
        
        // Clamp and write samples
        let sample_left = (mix_left.clamp(-1.0, 1.0) * 32767.0) as i16;
        let sample_right = (mix_right.clamp(-1.0, 1.0) * 32767.0) as i16;
        
        writer.write_sample(sample_left)?;
        writer.write_sample(sample_right)?;
    }
    
    writer.finalize()?;
    info!("Export complete: {:?}", output_path.as_ref());
    
    Ok(())
}