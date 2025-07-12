use pest::Parser;
use pest_derive::Parser;
use anyhow::Result;
use crate::ast::*;
use rsynth_core::{WaveformType, Envelope, FilterType, InstrumentDefinition};
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct SongParser;

pub fn parse_song_file(input: &str) -> Result<SongFile> {
    let pairs = SongParser::parse(Rule::file, input)?;
    
    let mut instruments = HashMap::new();
    let mut patterns = HashMap::new();
    let mut sections = Vec::new();
    let mut tempo = 120.0;
    
    // Add default instruments
    let kick = InstrumentDefinition::kick();
    instruments.insert(kick.name.clone(), kick);
    let snare = InstrumentDefinition::snare();
    instruments.insert(snare.name.clone(), snare);
    let hihat = InstrumentDefinition::hihat();
    instruments.insert(hihat.name.clone(), hihat);
    let bass = InstrumentDefinition::bass();
    instruments.insert(bass.name.clone(), bass);
    
    for pair in pairs {
        match pair.as_rule() {
            Rule::file => {
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::statement => {
                            // Unwrap the statement wrapper
                            for stmt in inner.into_inner() {
                                match stmt.as_rule() {
                                    Rule::pattern_def => {
                                        let (name, pattern) = parse_pattern(stmt)?;
                                        patterns.insert(name, pattern);
                                    }
                                    Rule::tempo_def => {
                                        tempo = parse_tempo(stmt)?;
                                    }
                                    Rule::song_section => {
                                        sections.push(parse_section(stmt)?);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Rule::EOI => {}
                        rule => {
                            eprintln!("Unhandled rule in parser: {:?}", rule);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    Ok(SongFile {
        instruments,
        patterns,
        sections,
        tempo,
    })
}

fn parse_pattern(pair: pest::iterators::Pair<Rule>) -> Result<(String, PatternDef)> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    
    let mut tracks = HashMap::new();
    let mut tempo = None;
    
    for item in inner {
        match item.as_rule() {
            Rule::pattern_tracks => {
                for track_pair in item.into_inner() {
                    if track_pair.as_rule() == Rule::pattern_track {
                        let mut track_inner = track_pair.into_inner();
                        let track_name = track_inner.next().unwrap().as_str().to_string();
                        let pattern_str = track_inner.next().unwrap().as_str().to_string();
                        tracks.insert(track_name, pattern_str);
                    }
                }
            }
            Rule::tempo_spec => {
                tempo = Some(parse_tempo_spec(item)?);
            }
            _ => {}
        }
    }
    
    Ok((name, PatternDef { tracks, tempo }))
}

fn parse_tempo(pair: pest::iterators::Pair<Rule>) -> Result<f32> {
    let mut inner = pair.into_inner();
    // Skip through to find the number
    for item in inner {
        if item.as_rule() == Rule::number {
            return Ok(item.as_str().parse::<f32>()?);
        }
    }
    Err(anyhow::anyhow!("No number found in tempo definition"))
}

fn parse_tempo_spec(pair: pest::iterators::Pair<Rule>) -> Result<f32> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::number => Ok(inner.as_str().parse::<f32>()?),
        _ => Ok(120.0), // default
    }
}

fn parse_section(pair: pest::iterators::Pair<Rule>) -> Result<Section> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let bars = inner.next().unwrap().as_str().parse::<u32>()?;
    
    let mut items = Vec::new();
    
    for item in inner {
        if item.as_rule() == Rule::section_content {
            for section_item in item.into_inner() {
                if section_item.as_rule() == Rule::section_item {
                    let mut item_inner = section_item.into_inner();
                    let pattern = item_inner.next().unwrap().as_str().to_string();
                    let modifier = item_inner.next().map(|m| {
                        match m.as_str() {
                            "only" => Modifier::Only,
                            s if s.starts_with('*') => {
                                let num = s[1..].trim().parse::<u32>().unwrap_or(1);
                                Modifier::Multiply(num)
                            }
                            _ => Modifier::Only,
                        }
                    });
                    items.push(SectionItem { pattern, modifier });
                }
            }
        }
    }
    
    Ok(Section { name, bars, items })
}