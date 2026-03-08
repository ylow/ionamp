use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: i64,
    pub file_path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub duration_secs: Option<f64>,
    pub sample_rate: Option<i32>,
    pub bitrate: Option<i32>,
    pub format: Option<String>,
    pub file_size: Option<i64>,
    pub energy_vector: Option<Vec<f32>>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewTrack {
    pub file_path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub duration_secs: Option<f64>,
    pub sample_rate: Option<i32>,
    pub bitrate: Option<i32>,
    pub format: Option<String>,
    pub file_size: Option<i64>,
    pub energy_vector: Option<Vec<f32>>,
}
