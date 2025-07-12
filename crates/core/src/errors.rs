use thiserror::Error;

#[derive(Error, Debug)]
pub enum SynthError {
    #[error("Invalid frequency: {0}")]
    InvalidFrequency(f32),
    
    #[error("Invalid BPM: {0}")]
    InvalidBpm(f32),
    
    #[error("Pattern error: {0}")]
    PatternError(String),
    
    #[error("Instrument not found: {0}")]
    InstrumentNotFound(String),
    
    #[error("Audio error: {0}")]
    AudioError(String),
}

pub type Result<T> = std::result::Result<T, SynthError>;