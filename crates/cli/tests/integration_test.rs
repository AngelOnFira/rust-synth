use rsynth_core::{InstrumentLibrary, Song, Pattern, Track};
use rsynth_engine::export_song_to_wav;
use rsynth_lang::parse_song_file;
use std::path::Path;
use hound::WavReader;

#[test]
fn test_basic_pattern_parsing() {
    let input = r#"
tempo: 120bpm

drums = pattern {
    kick: "x...x...x...x..."
} @ 120bpm

[main: 4bars]
    drums
"#;

    let result = parse_song_file(input);
    assert!(result.is_ok());
    
    let song_file = result.unwrap();
    assert_eq!(song_file.tempo, 120.0);
    assert!(song_file.patterns.contains_key("drums"));
}

#[test]
fn test_wav_export_creates_file() {
    let mut song = Song::new("test".to_string(), 120.0);
    let mut track = Track::new("kick_track".to_string());
    track.patterns.push(Pattern::new(
        "kick_pattern".to_string(),
        "x...x...x...x...",
        "kick".to_string(),
    ));
    song.tracks.insert("kick_track".to_string(), track);
    
    let instruments = InstrumentLibrary::default();
    let output_path = "test_output.wav";
    
    let result = export_song_to_wav(&song, &instruments, output_path, 1.0);
    assert!(result.is_ok());
    assert!(Path::new(output_path).exists());
    
    // Verify the WAV file is valid and contains audio
    let reader = WavReader::open(output_path).unwrap();
    let spec = reader.spec();
    assert_eq!(spec.channels, 2);
    assert_eq!(spec.sample_rate, 44100);
    
    // Count samples to ensure audio was written
    let sample_count = reader.len();
    assert!(sample_count > 0);
    // hound's len() returns total samples across all channels
    // 44100 samples/sec * 2 channels * 1 second = 88200
    assert_eq!(sample_count, 88200); // Total samples in file
    
    // Clean up
    std::fs::remove_file(output_path).ok();
}

#[test]
fn test_melodic_pattern_parsing() {
    let input = r#"
tempo: 120bpm

bassline = pattern {
    bass: "C2 . E2 . G2 . F2 ."
} @ 120bpm

[main: 4bars]
    bassline
"#;

    let result = parse_song_file(input);
    assert!(result.is_ok());
    
    let song_file = result.unwrap();
    let song = song_file.to_song();
    
    // Verify the melodic pattern was created
    let track = song.tracks.values().next().unwrap();
    assert_eq!(track.melodic_patterns.len(), 1);
    
    let pattern = &track.melodic_patterns[0];
    assert_eq!(pattern.notes.len(), 8); // "C2 . E2 . G2 . F2 ." = 8 steps
    
    // Check specific notes
    assert!(pattern.notes[0].is_some());
    assert!(pattern.notes[1].is_none()); // "." means rest
    assert!(pattern.notes[2].is_some());
}

#[test]
fn test_note_frequency_calculation() {
    use rsynth_core::{Note, NotePitch};
    
    let a4 = Note {
        pitch: NotePitch::A,
        octave: 4,
        velocity: 1.0,
    };
    assert!((a4.frequency() - 440.0).abs() < 0.01);
    
    let c4 = Note {
        pitch: NotePitch::C,
        octave: 4,
        velocity: 1.0,
    };
    assert!((c4.frequency() - 261.63).abs() < 0.01);
}

#[test] 
fn test_full_song_export() {
    let input = r#"
tempo: 128bpm

drums = pattern {
    kick:  "x...x...x...x...",
    snare: "....x.......x...",
    hihat: "..x...x...x...x."
} @ 128bpm

bassline = pattern {
    bass: "C2 . . . E2 . . . G2 . . . F2 . . ."
} @ 128bpm

[intro: 2bars]
    drums

[main: 4bars]
    drums
    bassline
"#;

    let song_file = parse_song_file(input).unwrap();
    let song = song_file.to_song();
    let instruments = InstrumentLibrary::default();
    
    let output_path = "test_full_song.wav";
    let result = export_song_to_wav(&song, &instruments, output_path, 2.0);
    assert!(result.is_ok());
    
    // Verify file exists and has expected duration
    let reader = WavReader::open(output_path).unwrap();
    // 44100 samples/sec * 2 channels * 2 seconds = 176400
    assert_eq!(reader.len(), 176400); // 2 seconds at 44.1kHz (total samples)
    
    // Clean up
    std::fs::remove_file(output_path).ok();
}