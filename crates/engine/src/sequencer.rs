use rsynth_core::{Song, InstrumentLibrary, create_instrument_synth, SynthNode};
use fundsp::prelude::*;
use std::sync::{Arc, Mutex};
use crossbeam::channel::{Sender, Receiver, bounded};
use tracing::{info, debug};

pub enum SequencerCommand {
    Play,
    Stop,
    UpdateSong(Song),
    SetPosition(f32),
}

pub struct Sequencer {
    song: Arc<Mutex<Song>>,
    instruments: Arc<InstrumentLibrary>,
    sample_rate: f32,
    current_time: f64,
    playing: bool,
    command_rx: Receiver<SequencerCommand>,
    // Simplified: we'll use a mixer node that we rebuild when needed
    mixer: Box<dyn AudioUnit>,
}

impl Sequencer {
    pub fn new(
        song: Song,
        instruments: InstrumentLibrary,
        sample_rate: f32,
    ) -> (Self, Sender<SequencerCommand>) {
        let (tx, rx) = bounded(100);
        
        // Start with silence
        let mixer = Box::new(zero()) as Box<dyn AudioUnit>;
        
        let sequencer = Self {
            song: Arc::new(Mutex::new(song)),
            instruments: Arc::new(instruments),
            sample_rate,
            current_time: 0.0,
            playing: false,
            command_rx: rx,
            mixer,
        };
        
        (sequencer, tx)
    }
    
    pub fn process(&mut self, output: &mut [f32]) {
        // Process commands
        while let Ok(cmd) = self.command_rx.try_recv() {
            match cmd {
                SequencerCommand::Play => {
                    self.playing = true;
                    info!("Sequencer started");
                }
                SequencerCommand::Stop => {
                    self.playing = false;
                    self.current_time = 0.0;
                    info!("Sequencer stopped");
                }
                SequencerCommand::UpdateSong(new_song) => {
                    *self.song.lock().unwrap() = new_song;
                    info!("Song updated");
                }
                SequencerCommand::SetPosition(time) => {
                    self.current_time = time as f64;
                }
            }
        }
        
        if !self.playing {
            output.fill(0.0);
            return;
        }
        
        // For now, just generate a simple test tone
        let samples_per_channel = output.len() / 2;
        
        for i in 0..samples_per_channel {
            let time = self.current_time + (i as f64 / self.sample_rate as f64);
            
            // Simple metronome click for testing
            let beat = (time * 2.0) as usize;
            let beat_phase = (time * 2.0).fract();
            
            let click = if beat_phase < 0.01 {
                (1.0 - beat_phase / 0.01) * 0.5
            } else {
                0.0
            };
            
            output[i * 2] = click as f32;
            output[i * 2 + 1] = click as f32;
        }
        
        self.current_time += samples_per_channel as f64 / self.sample_rate as f64;
    }
}