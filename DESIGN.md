# Rust Audio Synthesis Project Design Document

## Project Vision

A beginner-friendly, code-based music creation tool that generates all sounds from fundamental synthesis building blocks. The project emphasizes learning through exploration, allowing users to see how complex sounds are built from simple components.

### Core Principles
- **Everything from scratch**: All sounds generated through synthesis, no samples
- **Transparency**: Users can inspect how any sound is constructed
- **Pattern-based**: Terse, visual notation inspired by esolangs and trackers
- **Live coding**: Hot reload for immediate feedback
- **Educational**: Gradually introduces music theory and synthesis concepts

## Target Users
1. **Programmers** interested in music creation without deep music theory knowledge
2. **Musicians** wanting to explore synthesis and algorithmic composition
3. **Learners** curious about DSP and audio programming

## Technical Architecture

### Layer Structure (Bottom to Top)

#### 1. Atoms (Fundamental Building Blocks)
- **Oscillators**: sine, square, triangle, sawtooth, noise
- **Basic Operations**: addition, multiplication, frequency/amplitude modulation
- **Time**: sample rate, buffer management

#### 2. Operators (Sound Modifiers)
- **Envelopes**: ADSR (Attack, Decay, Sustain, Release)
- **Filters**: Low-pass, high-pass, band-pass, notch
- **Effects**: Delay, reverb, distortion, compression
- **Modulators**: LFOs (Low Frequency Oscillators), envelope followers

#### 3. Instruments (Synthesizer Patches)
- Combinations of atoms and operators
- Parameterized for real-time control
- Definable in user files
- Introspectable (view construction)

#### 4. Patterns (Musical Sequences)
- Note sequences with timing
- Parameter automation
- Visual pattern notation
- Support for polyrhythms

#### 5. Tracks (Parallel Voices)
- Multiple patterns playing simultaneously
- Mixing and volume control
- Effects chains per track

#### 6. Songs (Complete Compositions)
- Arrangement of patterns
- Song structure (intro, verse, chorus, etc.)
- Global tempo and key
- Master effects

## Music Description Language

### Design Goals
- Terse but readable
- Visual pattern representation
- Progressive disclosure of complexity
- Live-codeable

### Example Syntax (Preliminary)

```
// === INSTRUMENT DEFINITIONS ===
// Simple bass synthesizer
bass = {
  osc: saw(freq)              // Sawtooth oscillator
  env: adsr(0.01, 0.1, 0.8, 0.2)  // Envelope
  filt: lpf(200hz, res: 0.3)  // Low-pass filter
  out: osc * env | filt       // Signal flow
}

// Kick drum from scratch
kick = {
  pitch: 60hz * exp_decay(0.1)  // Pitch envelope
  osc: sin(pitch)
  env: exp_decay(0.5)
  out: osc * env * 0.8
}

// === PATTERNS ===
// Visual drum pattern (16 steps)
drums = pattern {
  kick: "x...x...x...x..."
  snar: "....x.......x..."
  hhat: "x.x.x.x.x.x.x.x."
} @ 120bpm

// Melodic pattern with note names
bassline = pattern {
  notes: "C2 . . E2 . . G2 . F2 . . E2 . . D2 ."
  vel:   "ff . . mf . . ff . mp . . mf . . pp ."
} @ bass  // Use 'bass' instrument

// Parameter automation
filter_sweep = pattern {
  bass.filt.freq: "200...1000...200"  // Sweep from 200Hz to 1kHz
} over 4bars

// === SONG STRUCTURE ===
song = {
  tempo: 120bpm
  
  [intro: 4bars]
    drums.kick only  // Solo kick
    
  [verse: 8bars]
    drums
    bassline
    
  [buildup: 4bars]
    drums
    bassline
    filter_sweep  // Apply automation
    
  [drop: 16bars]
    drums * 2  // Double-time drums
    bassline
    lead_melody
}

// === LIVE CODING FEATURES ===
#solo bassline      // Only play bassline
#mute drums.hhat    // Mute hi-hats
#loop verse         // Loop the verse section
```

## User Interface

### CLI Commands
```bash
# Play a song file with hot reload
rust-synth play song.rsynth --watch

# Render to audio file
rust-synth render song.rsynth -o output.wav

# Interactive mode (REPL)
rust-synth repl

# Inspect an instrument
rust-synth inspect song.rsynth --instrument bass

# Generate example songs
rust-synth examples --genre techno
```

### Terminal UI Features
- Waveform visualization
- Spectrum analyzer
- Pattern grid display
- Parameter values
- CPU/memory usage

## Technical Requirements

### Audio Engine
- Real-time synthesis with low latency
- Multi-threaded processing
- Lock-free audio thread
- Efficient buffer management

### File Watching
- Hot reload on save
- Incremental compilation
- Error recovery

### Export Formats
- WAV (uncompressed)
- FLAC (lossless compression)
- Real-time audio output

## Learning Path

### Tutorials Structure
1. **Hello Sine Wave** - Generate your first sound
2. **Building a Kick Drum** - Combine oscillator and envelope
3. **Pattern Basics** - Create rhythms
4. **Melodic Patterns** - Notes and scales
5. **Synthesis Deep Dive** - Understanding oscillators and filters
6. **Genre Templates** - How to make techno, ambient, etc.

### Music Theory Integration
- Start with frequencies, introduce note names gradually
- Visual representations of intervals and scales
- Interactive experiments (what happens if...?)

## Performance Considerations

### Optimization Opportunities
- SIMD for DSP operations
- Compile-time synthesis graph optimization
- Lazy evaluation of unused tracks
- Parallel processing per track

### Benchmarking
- Measure CPU usage per voice
- Track memory allocations
- Profile real-time safety

## Future Extensions

### Potential Features
- MIDI input/output
- OSC support for external control
- GUI with node-based patching
- Community instrument library
- AI-assisted composition
- VST plugin export

### Modularity
- Plugin system for custom operators
- User-defined pattern notations
- External language bindings

## Development Phases

### Phase 1: Core Engine (MVP)
- Basic oscillators and operators
- Simple pattern playback
- WAV export
- CLI interface

### Phase 2: Language & Live Coding
- Full language parser
- Hot reload
- REPL mode
- Basic tutorials

### Phase 3: Polish & Performance
- Optimization
- Advanced effects
- Terminal UI
- Genre examples

### Phase 4: Community & Extensions
- Plugin system
- GUI exploration
- Documentation
- Workshop materials

## Technical Decisions to Make

1. **Audio Backend**: cpal, rodio, or custom?
   - **Recommendation**: CPAL for direct control over latency and buffers
2. **DSP Library**: dasp, fundsp, or from scratch?
   - **Recommendation**: FunDSP for its functional, composable approach
3. **Parser**: nom, pest, or hand-written?
   - **Decision pending**: Depends on language complexity
4. **File Format**: Custom or existing (YAML, TOML)?
   - **Recommendation**: Custom format for terse pattern notation
5. **Threading Model**: Single audio thread or parallel?
   - **Recommendation**: Lock-free audio thread with worker threads

## Success Metrics

- Can create a complete electronic track
- Learning curve under 1 hour for basic sounds
- Real-time performance on modest hardware
- Community adoption and contributions

## Inspiration & References

- **Trackers**: Pattern-based sequencing
- **SuperCollider**: Live coding concepts
- **Sonic Pi**: Educational approach
- **TidalCycles**: Pattern language
- **Esolangs**: Terse, expressive notation

## Open Questions

1. How to handle polyphony elegantly?
2. Best visual representations for synthesis concepts?
3. How much music theory to expose initially?
4. Balance between terseness and readability?
5. Standard library of instruments/effects?

---

*This document is a living design that will evolve as we explore implementation details and gather feedback.*