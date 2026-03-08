use std::path::Path;

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::Accessor;

use crate::models::NewTrack;

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "ogg", "wav", "aiff", "aac", "m4a", "alac", "opus",
];

pub fn is_supported_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("lofty error: {0}")]
    Lofty(#[from] lofty::error::LoftyError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn extract_metadata(path: &Path) -> Result<NewTrack, MetadataError> {
    if !is_supported_audio(path) {
        return Err(MetadataError::UnsupportedFormat(
            path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown")
                .to_string(),
        ));
    }

    let tagged_file = lofty::read_from_path(path)?;
    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());
    let properties = tagged_file.properties();

    let file_meta = std::fs::metadata(path)?;

    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    Ok(NewTrack {
        file_path: path.to_string_lossy().to_string(),
        title: tag.and_then(|t| t.title().map(|s| s.to_string())),
        artist: tag.and_then(|t| t.artist().map(|s| s.to_string())),
        album: tag.and_then(|t| t.album().map(|s| s.to_string())),
        album_artist: None, // lofty doesn't have a standard accessor for this
        genre: tag.and_then(|t| t.genre().map(|s| s.to_string())),
        year: tag.and_then(|t| t.year()).map(|y| y as i32),
        track_number: tag.and_then(|t| t.track()).map(|n| n as i32),
        disc_number: tag.and_then(|t| t.disk()).map(|n| n as i32),
        duration_secs: Some(properties.duration().as_secs_f64()),
        sample_rate: properties.sample_rate().map(|sr| sr as i32),
        bitrate: properties.audio_bitrate().map(|br| br as i32),
        format,
        file_size: Some(file_meta.len() as i64),
        energy_rms: None,
        energy_centroid: None,
        energy_onset: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_audio() {
        assert!(is_supported_audio(Path::new("song.mp3")));
        assert!(is_supported_audio(Path::new("song.FLAC")));
        assert!(is_supported_audio(Path::new("song.ogg")));
        assert!(is_supported_audio(Path::new("song.wav")));
        assert!(is_supported_audio(Path::new("song.m4a")));
        assert!(is_supported_audio(Path::new("song.opus")));
        assert!(!is_supported_audio(Path::new("file.txt")));
        assert!(!is_supported_audio(Path::new("file.png")));
        assert!(!is_supported_audio(Path::new("noext")));
    }

    #[test]
    fn test_extract_metadata_mp3() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test.mp3");
        let track = extract_metadata(&path).unwrap();
        assert_eq!(track.title.as_deref(), Some("Test Song"));
        assert_eq!(track.artist.as_deref(), Some("Test Artist"));
        assert_eq!(track.album.as_deref(), Some("Test Album"));
        assert_eq!(track.genre.as_deref(), Some("Rock"));
        assert_eq!(track.format.as_deref(), Some("mp3"));
        assert!(track.duration_secs.unwrap() > 0.0);
        assert!(track.file_size.unwrap() > 0);
        assert!(track.sample_rate.is_some());
    }

    #[test]
    fn test_extract_metadata_flac() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test.flac");
        let track = extract_metadata(&path).unwrap();
        assert_eq!(track.title.as_deref(), Some("Test Song"));
        assert_eq!(track.format.as_deref(), Some("flac"));
    }

    #[test]
    fn test_unsupported_format() {
        let path = Path::new("not_audio.txt");
        let result = extract_metadata(path);
        assert!(result.is_err());
        assert!(matches!(result, Err(MetadataError::UnsupportedFormat(_))));
    }
}
