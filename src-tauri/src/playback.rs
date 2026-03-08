use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Mutex;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use serde::Serialize;

pub struct PlaybackManager {
    handle: OutputStreamHandle,
    sink: Sink,
    current_path: Option<String>,
    current_title: Option<String>,
    current_artist: Option<String>,
    duration_secs: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaybackStatus {
    pub playing: bool,
    pub paused: bool,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub current_path: Option<String>,
    pub current_title: Option<String>,
    pub current_artist: Option<String>,
    /// True when the track has finished (sink is empty and we had a track loaded)
    pub ended: bool,
}

impl PlaybackManager {
    pub fn new() -> Result<Self, String> {
        let (stream, handle) =
            OutputStream::try_default().map_err(|e| format!("audio output error: {}", e))?;
        // Leak the stream so it stays alive for the entire app lifetime.
        // OutputStream is !Send so we can't store it in Mutex-managed state.
        std::mem::forget(stream);
        let sink = Sink::try_new(&handle).map_err(|e| format!("sink error: {}", e))?;
        sink.pause(); // Start paused
        Ok(Self {
            handle,
            sink,
            current_path: None,
            current_title: None,
            current_artist: None,
            duration_secs: 0.0,
        })
    }

    pub fn play_file(
        &mut self,
        path: &str,
        title: Option<String>,
        artist: Option<String>,
        duration_secs: f64,
    ) -> Result<(), String> {
        // Stop current playback
        self.sink.stop();
        // Create a new sink (old one is consumed by stop)
        self.sink =
            Sink::try_new(&self.handle).map_err(|e| format!("sink error: {}", e))?;

        let file =
            File::open(Path::new(path)).map_err(|e| format!("file open error: {}", e))?;
        let reader = BufReader::new(file);
        let source =
            Decoder::new(reader).map_err(|e| format!("decode error: {}", e))?;

        self.sink.append(source);
        self.current_path = Some(path.to_string());
        self.current_title = title;
        self.current_artist = artist;
        self.duration_secs = duration_secs;

        Ok(())
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn resume(&self) {
        self.sink.play();
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.current_path = None;
        self.current_title = None;
        self.current_artist = None;
        self.duration_secs = 0.0;
    }

    pub fn seek(&self, position_secs: f64) -> Result<(), String> {
        self.sink
            .try_seek(std::time::Duration::from_secs_f64(position_secs))
            .map_err(|e| format!("seek error: {}", e))
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn status(&self) -> PlaybackStatus {
        let has_track = self.current_path.is_some();
        let empty = self.sink.empty();
        let paused = self.sink.is_paused();

        PlaybackStatus {
            playing: has_track && !empty && !paused,
            paused: has_track && !empty && paused,
            position_secs: self.sink.get_pos().as_secs_f64(),
            duration_secs: self.duration_secs,
            current_path: self.current_path.clone(),
            current_title: self.current_title.clone(),
            current_artist: self.current_artist.clone(),
            ended: has_track && empty,
        }
    }
}

pub type SharedPlayback = Mutex<PlaybackManager>;
