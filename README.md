# Rust Audio Synthesizer

A programmatic music creation tool built in Rust using FunDSP. Generate electronic music using a terse, pattern-based language with real-time playback and hot-reloading.

## Features

- **Pure Synthesis**: All sounds generated from fundamental oscillators (sine, square, saw, triangle, noise)
- **Pattern-Based Composition**: Visual drum patterns and melodic sequences
- **Live Coding**: Hot-reload on file save for instant feedback
- **Multi-track Support**: Layer drums and melodies
- **Export to WAV**: Render your compositions to audio files
- **Modular Architecture**: Separated into core, language, engine, and CLI crates

## Quick Start

### Build the Project

```bash
cargo build --release
```

### Play a Demo Song

```bash
# Real-time playback
cargo run --release -- play examples/demo.rsynth

# With hot-reload (edit the file and hear changes instantly)
cargo run --release -- play examples/demo.rsynth --watch

# Export to WAV
cargo run --release -- render examples/demo.rsynth --output demo.wav --duration 30
```

## Music Language

The synthesizer uses a custom pattern-based language:

```rsynth
// Set tempo
tempo: 120bpm

// Define drum patterns (visual notation)
drums = pattern {
    kick:  "x...x...x...x...",
    snare: "....x.......x...",
    hihat: "xxxxxxxxxxxxxxxx"
} @ 120bpm

// Define melodic patterns
bassline = pattern {
    bass: "C2 . . E2 . . G2 . F2 . . E2 . . D2 ."
} @ 120bpm

// Song structure
[intro: 4bars]
    drums

[main: 8bars]
    drums
    bassline
```

### Pattern Notation

- **Drum Patterns**: Use `x` for hits and `.` for rests
- **Melodic Patterns**: Use note names (C2, E3, etc.) and `.` for rests
- **Timing**: Each character represents one 16th note

## Architecture

The project is organized into multiple crates:

- **`rsynth-core`**: Core data structures and synthesis primitives
- **`rsynth-lang`**: Parser for the music description language
- **`rsynth-engine`**: Audio playback and export engine
- **`rsynth-cli`**: Command-line interface

## Built-in Instruments

The synthesizer comes with pre-defined instruments:

- **Kick**: Sine wave with pitch envelope
- **Snare**: Filtered white noise
- **Hi-hat**: High-passed white noise
- **Bass**: Sawtooth wave with low-pass filter

## Development Status

This is a proof-of-concept demonstrating:

- FunDSP integration for synthesis
- Pattern-based music composition
- Real-time audio with CPAL
- Hot-reloading with file watching

### Current Limitations

- Basic voice management (simple metronome for real-time playback)
- Limited synthesis options (basic oscillators and filters)
- Simple pattern language (16-step patterns only)
- No MIDI support yet

### Future Enhancements

- Full polyphonic voice management
- More synthesis options (FM, granular, wavetable)
- Advanced pattern features (velocity, automation)
- MIDI input/output
- GUI with visual pattern editor
- VST plugin export

## Testing

Run the test suite:

```bash
cargo test
```

## Examples

Check the `examples/` directory for demo songs:

- `simple_beat.rsynth` - Basic 4/4 drum pattern
- `demo.rsynth` - Techno pattern with bass line

## Design Philosophy

Everything is built from fundamental sound atoms. Even complex instruments are combinations of:

- **Oscillators**: Basic waveforms
- **Envelopes**: ADSR amplitude shaping
- **Filters**: Frequency shaping
- **Modulators**: LFOs and envelopes

This transparency allows users to understand and create their own instruments from scratch.

## Contributing

This is an experimental project exploring programmatic music creation in Rust. Contributions and ideas are welcome!

## License

MIT