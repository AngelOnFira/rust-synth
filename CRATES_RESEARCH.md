# Rust Audio Crates Research

## Audio I/O Libraries

### CPAL (Cross-Platform Audio Library)
- **Purpose**: Low-level cross-platform audio I/O
- **Pros**:
  - Direct hardware access
  - Supports both input and output streams
  - Pure Rust implementation
  - Fine control over buffer sizes and latency
  - ASIO support on Windows for professional audio
- **Cons**:
  - Low-level API requires manual buffer management
  - More complex to implement basic playback
- **Best for**: Our project if we need precise control over latency and buffer management

### Rodio
- **Purpose**: High-level audio playback built on CPAL
- **Pros**:
  - Easy to use for simple playback
  - Built-in format decoding (MP3, FLAC, WAV, Vorbis)
  - Audio mixing capabilities
  - Good starting point for beginners
- **Cons**:
  - Less control over low-level details
  - Primarily focused on playback (not recording)
  - Some reported stability issues in edge cases
- **Best for**: Quick prototyping, but may be limiting for real-time synthesis

## DSP and Synthesis Libraries

### FunDSP
- **Version**: 0.19.1 (actively maintained)
- **Purpose**: Functional audio processing and synthesis
- **Pros**:
  - Composable graph notation
  - No macros needed - integrates as first-class Rust
  - Generic channel support (mono→stereo easily)
  - Works in no_std environments
  - Analyzes latencies and frequency responses analytically
  - Mathematical approach to DSP
- **Cons**:
  - Learning curve for functional approach
  - May be overly abstract for beginners
- **Best for**: Building our synthesis engine with clean, composable abstractions

### HexoDSP
- **Purpose**: Modular synthesis library (powers HexoSynth)
- **Pros**:
  - High-level API for DSP graphs
  - Growing collection of modules (oscillators, filters, envelopes, sequencers)
  - Runtime-changeable graphs
  - Optional JIT compiler for custom DSP
  - Built for modular synthesis specifically
- **Cons**:
  - May be too opinionated for our custom language
  - Larger dependency
- **Best for**: If we want a batteries-included approach

### DASP (formerly Sample)
- **Purpose**: Low-level PCM DSP fundamentals
- **Pros**:
  - Very low-level control
  - High performance
  - Works without std library
  - Good for building custom DSP from scratch
- **Cons**:
  - Very low-level - need to implement everything
  - Less convenient abstractions
- **Best for**: If we want maximum control and performance

## Real-Time Considerations

### Latency Requirements
- For live synthesis, need <10ms latency (ideally 2-5ms)
- Buffer sizes typically 64-512 samples at 44.1/48kHz
- Must avoid allocations in audio thread
- Lock-free communication between threads

### Performance Patterns
- Use lock-free data structures (e.g., ringbuffer crate)
- Pre-allocate all buffers
- Avoid system calls in audio callback
- Consider SIMD optimizations

## Recommendation for Our Project

### Core Audio Stack
1. **Audio I/O**: Start with CPAL for maximum control
   - Direct buffer access for low latency
   - Can add Rodio layer later if needed

2. **DSP Engine**: FunDSP as primary synthesis library
   - Clean functional abstractions align with our "atoms→instruments" philosophy
   - Composable design matches our language goals
   - Can inspect signal flow programmatically

3. **Additional Tools**:
   - `hound` for WAV file writing
   - `rubato` for resampling if needed
   - `realfft` for FFT operations (spectrum analysis)

### Architecture Benefits
- FunDSP's functional approach maps well to our declarative language
- CPAL gives us control for live coding hot-reload
- Both work in real-time contexts
- Can build educational visualizations on top

### Implementation Path
1. Start with CPAL + FunDSP prototype
2. Build basic oscillators and filters
3. Add pattern sequencing on top
4. Implement file watching and hot reload
5. Optimize for real-time performance

## Additional Libraries to Consider

### Parser/Language
- `pest` - Parser generator for our music language
- `nom` - Parser combinators (alternative approach)

### File Formats
- `serde` + `toml` - If we want TOML-based format
- Custom parser for our terse notation

### Utilities
- `notify` - File watching for hot reload
- `crossterm` - Terminal UI
- `ratatui` - Advanced terminal UI (for visualizations)