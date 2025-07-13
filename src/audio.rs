use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use tracing::info;
use symphonia::core::codecs::CODEC_TYPE_NULL;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::default::get_probe;

pub struct AudioManager {
    _stream: OutputStream,
    _stream_handle: rodio::OutputStreamHandle,
    sink: Option<Sink>,
    current_file: Option<String>,
    is_playing: bool,
    is_paused: bool,
    current_duration: Option<Duration>,
}

impl AudioManager {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().expect("Failed to create audio stream");
        
        Self {
            _stream,
            _stream_handle: stream_handle,
            sink: None,
            current_file: None,
            is_playing: false,
            is_paused: false,
            current_duration: None,
        }
    }

    pub fn play_file(&mut self, file_path: &str) -> Result<()> {
        info!("Playing file: {}", file_path);
        
        // Stop current playback if any
        self.stop();
        
        // Create a new sink
        let sink = Sink::try_new(&self._stream_handle)?;
        
        // Open and decode the audio file
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader)?;
        
        // Play the audio
        sink.append(source);
        sink.play();
        
        self.sink = Some(sink);
        self.current_file = Some(file_path.to_string());
        self.is_playing = true;
        self.is_paused = false;
        self.current_duration = Self::probe_duration(file_path);
        
        Ok(())
    }

    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            self.is_paused = true;
            self.is_playing = false;
            info!("Audio paused");
        }
    }

    pub fn resume(&mut self) {
        if let Some(sink) = &self.sink {
            sink.play();
            self.is_playing = true;
            self.is_paused = false;
            info!("Audio resumed");
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
        self.sink = None;
        self.current_file = None;
        self.is_playing = false;
        self.is_paused = false;
        self.current_duration = None;
        info!("Audio stopped");
    }

    pub fn set_volume(&mut self, volume: f32) {
        if let Some(sink) = &self.sink {
            sink.set_volume(volume);
        }
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn current_file(&self) -> Option<&String> {
        self.current_file.as_ref()
    }

    pub fn get_position(&self) -> Duration {
        if let Some(sink) = &self.sink {
            Duration::from_secs_f32(sink.len() as f32 / 44100.0) // Approximate position
        } else {
            Duration::ZERO
        }
    }

    pub fn get_current_position(&self) -> Duration {
        if let Some(sink) = &self.sink {
            // Get the current playback position
            let samples_played = sink.len() as f32;
            Duration::from_secs_f32(samples_played / 44100.0)
        } else {
            Duration::ZERO
        }
    }

    pub fn get_total_duration(&self) -> Option<Duration> {
        self.current_duration
    }

    fn probe_duration(file_path: &str) -> Option<Duration> {
        let file = File::open(file_path).ok()?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let probed = get_probe().format(
            &Default::default(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        ).ok()?;
        let format = probed.format;
        let track = format.tracks().iter().find(|t| t.codec_params.codec != CODEC_TYPE_NULL)?;
        let duration = track.codec_params.n_frames.and_then(|frames| {
            track.codec_params.sample_rate.map(|rate| Duration::from_secs_f64(frames as f64 / rate as f64))
        });
        duration
    }

    pub fn is_finished(&self) -> bool {
        if let Some(sink) = &self.sink {
            sink.len() == 0 && !sink.is_paused()
        } else {
            false
        }
    }
} 