use rsynth_core::{InstrumentDefinition, Pattern, Song, Track, WaveformType, Envelope, FilterType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SongFile {
    pub instruments: HashMap<String, InstrumentDefinition>,
    pub patterns: HashMap<String, PatternDef>,
    pub sections: Vec<Section>,
    pub tempo: f32,
}

#[derive(Debug, Clone)]
pub struct PatternDef {
    pub tracks: HashMap<String, String>,
    pub tempo: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub bars: u32,
    pub items: Vec<SectionItem>,
}

#[derive(Debug, Clone)]
pub struct SectionItem {
    pub pattern: String,
    pub modifier: Option<Modifier>,
}

#[derive(Debug, Clone)]
pub enum Modifier {
    Only,
    Multiply(u32),
}

impl SongFile {
    pub fn to_song(&self) -> Song {
        let mut song = Song::new("demo".to_string(), self.tempo);
        
        for (pattern_name, pattern_def) in &self.patterns {
            for (instrument_name, pattern_str) in &pattern_def.tracks {
                let track_name = format!("{}_{}", pattern_name, instrument_name);
                let mut track = Track::new(track_name.clone());
                
                let pattern_str = pattern_str.trim_matches('"');
                
                if pattern_str.contains(|c: char| c.is_alphabetic() && c != 'x' && c != 'X') {
                    // Melodic pattern
                    let notes: Vec<_> = pattern_str.split_whitespace()
                        .map(|s| rsynth_core::Note::parse(s))
                        .collect();
                    
                    track.melodic_patterns.push(rsynth_core::MelodicPattern {
                        name: pattern_name.clone(),
                        notes,
                        instrument: instrument_name.clone(),
                    });
                } else {
                    // Drum pattern
                    track.patterns.push(Pattern::new(
                        pattern_name.clone(),
                        pattern_str,
                        instrument_name.clone(),
                    ));
                }
                
                song.tracks.insert(track_name, track);
            }
        }
        
        song
    }
}