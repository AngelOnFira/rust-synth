use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub steps: Vec<char>,
    pub instrument: String,
}

impl Pattern {
    pub fn new(name: String, pattern_str: &str, instrument: String) -> Self {
        Self {
            name,
            steps: pattern_str.chars().collect(),
            instrument,
        }
    }
    
    pub fn is_active(&self, step: usize) -> bool {
        self.steps.get(step % self.steps.len())
            .map(|&c| c == 'x' || c == 'X')
            .unwrap_or(false)
    }
    
    pub fn length(&self) -> usize {
        self.steps.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MelodicPattern {
    pub name: String,
    pub notes: Vec<Option<Note>>,
    pub instrument: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Note {
    pub pitch: NotePitch,
    pub octave: u8,
    pub velocity: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NotePitch {
    C, Cs, D, Ds, E, F, Fs, G, Gs, A, As, B,
}

impl Note {
    pub fn frequency(&self) -> f32 {
        let semitone = match self.pitch {
            NotePitch::C => 0,
            NotePitch::Cs => 1,
            NotePitch::D => 2,
            NotePitch::Ds => 3,
            NotePitch::E => 4,
            NotePitch::F => 5,
            NotePitch::Fs => 6,
            NotePitch::G => 7,
            NotePitch::Gs => 8,
            NotePitch::A => 9,
            NotePitch::As => 10,
            NotePitch::B => 11,
        };
        
        440.0 * 2.0_f32.powf((semitone as f32 + (self.octave as f32 - 4.0) * 12.0 - 9.0) / 12.0)
    }
    
    pub fn parse(s: &str) -> Option<Self> {
        if s == "." || s.is_empty() {
            return None;
        }
        
        let chars: Vec<char> = s.chars().collect();
        if chars.len() < 2 {
            return None;
        }
        
        let pitch = match chars[0] {
            'C' => NotePitch::C,
            'D' => NotePitch::D,
            'E' => NotePitch::E,
            'F' => NotePitch::F,
            'G' => NotePitch::G,
            'A' => NotePitch::A,
            'B' => NotePitch::B,
            _ => return None,
        };
        
        let mut idx = 1;
        let pitch = if idx < chars.len() && chars[idx] == '#' {
            idx += 1;
            match pitch {
                NotePitch::C => NotePitch::Cs,
                NotePitch::D => NotePitch::Ds,
                NotePitch::F => NotePitch::Fs,
                NotePitch::G => NotePitch::Gs,
                NotePitch::A => NotePitch::As,
                _ => pitch,
            }
        } else {
            pitch
        };
        
        let octave = if idx < chars.len() {
            chars[idx].to_digit(10)? as u8
        } else {
            4
        };
        
        Some(Note {
            pitch,
            octave,
            velocity: 1.0,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub patterns: Vec<Pattern>,
    pub melodic_patterns: Vec<MelodicPattern>,
    pub volume: f32,
    pub muted: bool,
    pub solo: bool,
}

impl Track {
    pub fn new(name: String) -> Self {
        Self {
            name,
            patterns: Vec::new(),
            melodic_patterns: Vec::new(),
            volume: 1.0,
            muted: false,
            solo: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub name: String,
    pub bpm: f32,
    pub tracks: HashMap<String, Track>,
}

impl Song {
    pub fn new(name: String, bpm: f32) -> Self {
        Self {
            name,
            bpm,
            tracks: HashMap::new(),
        }
    }
    
    pub fn step_duration(&self) -> f32 {
        60.0 / self.bpm / 4.0
    }
}