pub mod atoms;
pub mod instruments;
pub mod patterns;
pub mod errors;

pub use atoms::{WaveformType, Envelope, FilterType, SynthNode, create_oscillator, create_envelope, create_filter, pitch_envelope};
pub use instruments::{InstrumentDefinition, InstrumentLibrary, create_instrument_synth};
pub use patterns::{Pattern, MelodicPattern, Note, NotePitch, Track, Song};
pub use errors::{SynthError, Result};