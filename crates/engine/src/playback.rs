use cpal::{traits::*, StreamConfig};
use rsynth_core::{Song, InstrumentLibrary, SynthError};
use crate::sequencer::{Sequencer, SequencerCommand};
use crossbeam::channel::Sender;
use std::sync::{Arc, Mutex};
use tracing::{info, error};
use anyhow::Result;

pub struct AudioEngine {
    _stream: cpal::Stream,
    command_tx: Sender<SequencerCommand>,
}

impl AudioEngine {
    pub fn new(song: Song, instruments: InstrumentLibrary) -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| SynthError::AudioError("No output device found".to_string()))?;
        
        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0 as f32;
        
        info!("Audio device: {}", device.name()?);
        info!("Sample rate: {}", sample_rate);
        
        let (mut sequencer, command_tx) = Sequencer::new(song, instruments, sample_rate);
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => Self::build_stream::<f32>(&device, &config.into(), sequencer)?,
            cpal::SampleFormat::I16 => Self::build_stream::<i16>(&device, &config.into(), sequencer)?,
            cpal::SampleFormat::U16 => Self::build_stream::<u16>(&device, &config.into(), sequencer)?,
            sample_format => {
                return Err(SynthError::AudioError(format!("Unsupported sample format: {:?}", sample_format)).into());
            }
        };
        
        stream.play()?;
        
        Ok(Self {
            _stream: stream,
            command_tx,
        })
    }
    
    fn build_stream<T>(
        device: &cpal::Device,
        config: &StreamConfig,
        mut sequencer: Sequencer,
    ) -> Result<cpal::Stream>
    where
        T: cpal::SizedSample + cpal::FromSample<f32>,
    {
        let channels = config.channels as usize;
        let err_fn = |err| error!("Audio stream error: {}", err);
        
        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let mut buffer = vec![0.0; data.len()];
                sequencer.process(&mut buffer);
                
                for (i, sample) in buffer.iter().enumerate() {
                    data[i] = T::from_sample(*sample);
                }
            },
            err_fn,
            None,
        )?;
        
        Ok(stream)
    }
    
    pub fn play(&self) -> Result<()> {
        self.command_tx.send(SequencerCommand::Play)?;
        Ok(())
    }
    
    pub fn stop(&self) -> Result<()> {
        self.command_tx.send(SequencerCommand::Stop)?;
        Ok(())
    }
    
    pub fn update_song(&self, song: Song) -> Result<()> {
        self.command_tx.send(SequencerCommand::UpdateSong(song))?;
        Ok(())
    }
}