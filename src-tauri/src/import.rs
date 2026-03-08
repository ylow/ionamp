use std::path::PathBuf;

use rusqlite::Connection;
use serde::Serialize;
use walkdir::WalkDir;

use crate::audio::decode::decode_to_mono_pcm;
use crate::audio::energy::compute_energy_components;
use crate::audio::metadata::{extract_metadata, is_supported_audio};
use crate::db::tracks::{insert_track, track_exists_by_path};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ImportEvent {
    ScanComplete { total_files: usize },
    Progress { current: usize, total: usize, file_name: String },
    Skipped { file_name: String, reason: String },
    Complete { imported: usize, skipped: usize, errors: usize },
}

pub fn scan_audio_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut result = Vec::new();
    for path in paths {
        if path.is_file() {
            if is_supported_audio(path) {
                result.push(path.clone());
            }
        } else if path.is_dir() {
            for entry in WalkDir::new(path).follow_links(true).into_iter().flatten() {
                let p = entry.path();
                if p.is_file() && is_supported_audio(p) {
                    result.push(p.to_path_buf());
                }
            }
        }
    }
    result
}

pub fn run_import(
    conn: &Connection,
    files: &[PathBuf],
    mut on_event: impl FnMut(ImportEvent),
) -> Result<(), String> {
    on_event(ImportEvent::ScanComplete {
        total_files: files.len(),
    });

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;

    for (i, path) in files.iter().enumerate() {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        on_event(ImportEvent::Progress {
            current: i + 1,
            total: files.len(),
            file_name: file_name.clone(),
        });

        let path_str = path.to_string_lossy();

        // Check if already exists
        match track_exists_by_path(conn, &path_str) {
            Ok(true) => {
                on_event(ImportEvent::Skipped {
                    file_name,
                    reason: "already imported".to_string(),
                });
                skipped += 1;
                continue;
            }
            Ok(false) => {}
            Err(e) => {
                on_event(ImportEvent::Skipped {
                    file_name,
                    reason: format!("db error: {}", e),
                });
                errors += 1;
                continue;
            }
        }

        // Extract metadata
        let mut new_track = match extract_metadata(path) {
            Ok(t) => t,
            Err(e) => {
                on_event(ImportEvent::Skipped {
                    file_name,
                    reason: format!("metadata error: {}", e),
                });
                errors += 1;
                continue;
            }
        };

        // Decode and compute energy
        match decode_to_mono_pcm(path) {
            Ok(audio) => {
                let ec = compute_energy_components(&audio.samples);
                new_track.energy_rms = Some(ec.rms);
                new_track.energy_centroid = Some(ec.centroid);
                new_track.energy_onset = Some(ec.onset);
            }
            Err(e) => {
                // Still import the track, just without energy data
                on_event(ImportEvent::Skipped {
                    file_name: file_name.clone(),
                    reason: format!("decode warning (importing without energy): {}", e),
                });
            }
        }

        // Insert into DB
        match insert_track(conn, &new_track) {
            Ok(Some(_)) => imported += 1,
            Ok(None) => {
                skipped += 1;
            }
            Err(e) => {
                on_event(ImportEvent::Skipped {
                    file_name,
                    reason: format!("insert error: {}", e),
                });
                errors += 1;
            }
        }
    }

    on_event(ImportEvent::Complete {
        imported,
        skipped,
        errors,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_in_memory;
    use std::fs;

    #[test]
    fn test_scan_audio_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("song.mp3"), b"fake").unwrap();
        fs::write(dir.path().join("song.flac"), b"fake").unwrap();
        fs::write(dir.path().join("readme.txt"), b"text").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/deep.wav"), b"fake").unwrap();

        let files = scan_audio_files(&[dir.path().to_path_buf()]);
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_import_inserts_tracks() {
        let conn = open_db_in_memory().unwrap();
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let files = scan_audio_files(&[fixtures]);

        let mut events = Vec::new();
        run_import(&conn, &files, |e| events.push(e)).unwrap();

        // Should have imported the test fixtures
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
            .unwrap();
        assert!(count >= 2, "Expected at least 2 tracks, got {}", count);

        // Check that Complete event was emitted
        let complete = events.iter().find(|e| matches!(e, ImportEvent::Complete { .. }));
        assert!(complete.is_some());
    }

    #[test]
    fn test_import_skips_duplicates() {
        let conn = open_db_in_memory().unwrap();
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let files = scan_audio_files(&[fixtures]);

        // First import
        run_import(&conn, &files, |_| {}).unwrap();
        let count1: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
            .unwrap();

        // Second import — all should be skipped
        let mut events = Vec::new();
        run_import(&conn, &files, |e| events.push(e)).unwrap();
        let count2: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count1, count2);

        // Complete event should show 0 imported
        if let Some(ImportEvent::Complete { imported, .. }) = events.iter().find(|e| matches!(e, ImportEvent::Complete { .. })) {
            assert_eq!(*imported, 0);
        }
    }
}
