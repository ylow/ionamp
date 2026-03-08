use rusqlite::{params, Connection, OptionalExtension, Result};

use crate::models::{NewTrack, Track};

pub fn energy_to_bytes(energy: &[f32]) -> Vec<u8> {
    energy.iter().flat_map(|f| f.to_le_bytes()).collect()
}

pub fn energy_from_bytes(bytes: &[u8]) -> Option<Vec<f32>> {
    if bytes.len() % 4 != 0 {
        return None;
    }
    Some(
        bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect(),
    )
}

pub fn row_to_track(row: &rusqlite::Row) -> Result<Track> {
    let energy_blob: Option<Vec<u8>> = row.get("energy_vector")?;
    let energy_vector = energy_blob.and_then(|b| energy_from_bytes(&b));
    Ok(Track {
        id: row.get("id")?,
        file_path: row.get("file_path")?,
        title: row.get("title")?,
        artist: row.get("artist")?,
        album: row.get("album")?,
        album_artist: row.get("album_artist")?,
        genre: row.get("genre")?,
        year: row.get("year")?,
        track_number: row.get("track_number")?,
        disc_number: row.get("disc_number")?,
        duration_secs: row.get("duration_secs")?,
        sample_rate: row.get("sample_rate")?,
        bitrate: row.get("bitrate")?,
        format: row.get("format")?,
        file_size: row.get("file_size")?,
        energy_vector,
        created_at: row.get("created_at")?,
    })
}

/// Inserts a track. Returns Some(id) if inserted, None if path already exists.
pub fn insert_track(conn: &Connection, track: &NewTrack) -> Result<Option<i64>> {
    let energy_bytes = track.energy_vector.as_ref().map(|v| energy_to_bytes(v));
    let rows = conn.execute(
        "INSERT OR IGNORE INTO tracks (
            file_path, title, artist, album, album_artist, genre,
            year, track_number, disc_number, duration_secs,
            sample_rate, bitrate, format, file_size, energy_vector
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            track.file_path,
            track.title,
            track.artist,
            track.album,
            track.album_artist,
            track.genre,
            track.year,
            track.track_number,
            track.disc_number,
            track.duration_secs,
            track.sample_rate,
            track.bitrate,
            track.format,
            track.file_size,
            energy_bytes,
        ],
    )?;
    if rows == 0 {
        Ok(None)
    } else {
        Ok(Some(conn.last_insert_rowid()))
    }
}

pub fn get_track_by_id(conn: &Connection, id: i64) -> Result<Option<Track>> {
    conn.query_row("SELECT * FROM tracks WHERE id = ?1", params![id], row_to_track)
        .optional()
}

pub fn get_track_by_path(conn: &Connection, path: &str) -> Result<Option<Track>> {
    conn.query_row(
        "SELECT * FROM tracks WHERE file_path = ?1",
        params![path],
        row_to_track,
    )
    .optional()
}

pub fn delete_track(conn: &Connection, id: i64) -> Result<bool> {
    let rows = conn.execute("DELETE FROM tracks WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}

pub fn track_exists_by_path(conn: &Connection, path: &str) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM tracks WHERE file_path = ?1)",
        params![path],
        |row| row.get(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_in_memory;

    fn make_test_track(path: &str) -> NewTrack {
        NewTrack {
            file_path: path.to_string(),
            title: Some("Test Song".to_string()),
            artist: Some("Test Artist".to_string()),
            album: Some("Test Album".to_string()),
            album_artist: None,
            genre: Some("Rock".to_string()),
            year: Some(2024),
            track_number: Some(1),
            disc_number: None,
            duration_secs: Some(180.0),
            sample_rate: Some(44100),
            bitrate: Some(320),
            format: Some("mp3".to_string()),
            file_size: Some(5_000_000),
            energy_vector: Some(vec![0.1, 0.5, 0.9]),
        }
    }

    #[test]
    fn test_insert_and_get_track() {
        let conn = open_db_in_memory().unwrap();
        let new_track = make_test_track("/music/test.mp3");
        let id = insert_track(&conn, &new_track).unwrap().unwrap();
        assert!(id > 0);

        let track = get_track_by_id(&conn, id).unwrap().unwrap();
        assert_eq!(track.file_path, "/music/test.mp3");
        assert_eq!(track.title.as_deref(), Some("Test Song"));
        assert_eq!(track.artist.as_deref(), Some("Test Artist"));
        assert_eq!(track.year, Some(2024));
        assert_eq!(track.energy_vector, Some(vec![0.1, 0.5, 0.9]));
    }

    #[test]
    fn test_insert_duplicate_path() {
        let conn = open_db_in_memory().unwrap();
        let track = make_test_track("/music/dup.mp3");
        let id1 = insert_track(&conn, &track).unwrap();
        assert!(id1.is_some());

        // Second insert with same path should be ignored (INSERT OR IGNORE)
        let id2 = insert_track(&conn, &track).unwrap();
        assert!(id2.is_none());

        // Only one track should exist
        let exists = track_exists_by_path(&conn, "/music/dup.mp3").unwrap();
        assert!(exists);
    }

    #[test]
    fn test_delete_track() {
        let conn = open_db_in_memory().unwrap();
        let track = make_test_track("/music/del.mp3");
        let id = insert_track(&conn, &track).unwrap().unwrap();

        let deleted = delete_track(&conn, id).unwrap();
        assert!(deleted);

        let found = get_track_by_id(&conn, id).unwrap();
        assert!(found.is_none());

        // Deleting again returns false
        let deleted_again = delete_track(&conn, id).unwrap();
        assert!(!deleted_again);
    }

    #[test]
    fn test_track_exists_by_path() {
        let conn = open_db_in_memory().unwrap();
        assert!(!track_exists_by_path(&conn, "/nope.mp3").unwrap());

        let track = make_test_track("/music/exists.mp3");
        insert_track(&conn, &track).unwrap();
        assert!(track_exists_by_path(&conn, "/music/exists.mp3").unwrap());
    }
}
