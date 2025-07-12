use clap::{Parser, Subcommand};
use anyhow::Result;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use rsynth_core::InstrumentLibrary;
use rsynth_engine::{AudioEngine, export_song_to_wav};
use rsynth_lang::parse_song_file;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "rsynth")]
#[command(about = "Rust Audio Synthesizer", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Play a song file with optional hot reload
    Play {
        /// Path to the song file
        file: PathBuf,
        
        /// Enable hot reload on file changes
        #[arg(short, long)]
        watch: bool,
    },
    
    /// Render a song to WAV file
    Render {
        /// Path to the song file
        file: PathBuf,
        
        /// Output WAV file path
        #[arg(short, long, default_value = "output.wav")]
        output: PathBuf,
        
        /// Duration in seconds
        #[arg(short, long, default_value = "30.0")]
        duration: f32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Play { file, watch } => {
            play_command(file, watch).await?;
        }
        Commands::Render { file, output, duration } => {
            render_command(file, output, duration).await?;
        }
    }
    
    Ok(())
}

async fn play_command(file: PathBuf, watch: bool) -> Result<()> {
    info!("Loading song file: {:?}", file);
    
    let content = tokio::fs::read_to_string(&file).await?;
    let song_file = parse_song_file(&content)?;
    let song = song_file.to_song();
    
    let instruments = InstrumentLibrary::default();
    let engine = AudioEngine::new(song.clone(), instruments)?;
    
    info!("Starting playback (BPM: {})", song.bpm);
    engine.play()?;
    
    if watch {
        info!("Watching for file changes...");
        
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        watcher.watch(file.as_ref(), RecursiveMode::NonRecursive)?;
        
        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    if let notify::EventKind::Modify(_) = event.kind {
                        info!("File changed, reloading...");
                        
                        match tokio::fs::read_to_string(&file).await {
                            Ok(content) => {
                                match parse_song_file(&content) {
                                    Ok(song_file) => {
                                        let new_song = song_file.to_song();
                                        engine.update_song(new_song)?;
                                        info!("Song reloaded successfully");
                                    }
                                    Err(e) => error!("Failed to parse song: {}", e),
                                }
                            }
                            Err(e) => error!("Failed to read file: {}", e),
                        }
                    }
                }
                Ok(Err(e)) => error!("Notify error: {:?}", e),
                Err(e) => error!("Watch channel error: {:?}", e),
            }
        }
    } else {
        info!("Press Ctrl+C to stop");
        tokio::signal::ctrl_c().await?;
        engine.stop()?;
    }
    
    Ok(())
}

async fn render_command(file: PathBuf, output: PathBuf, duration: f32) -> Result<()> {
    info!("Loading song file: {:?}", file);
    
    let content = tokio::fs::read_to_string(&file).await?;
    let song_file = parse_song_file(&content)?;
    let song = song_file.to_song();
    
    let instruments = InstrumentLibrary::default();
    
    info!("Rendering {} seconds to {:?}", duration, output);
    export_song_to_wav(&song, &instruments, output, duration)?;
    
    info!("Render complete!");
    
    Ok(())
}