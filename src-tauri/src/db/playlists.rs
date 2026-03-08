use rusqlite::{params, Connection, Result};

use super::tracks::row_to_track;
use crate::models::{Playlist, PlaylistEntry};

pub fn create_playlist(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute("INSERT INTO playlists (name) VALUES (?1)", params![name])?;
    Ok(conn.last_insert_rowid())
}

pub fn list_playlists(conn: &Connection) -> Result<Vec<Playlist>> {
    let mut stmt = conn.prepare("SELECT id, name, created_at FROM playlists ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(Playlist {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn rename_playlist(conn: &Connection, id: i64, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE playlists SET name = ?1 WHERE id = ?2",
        params![new_name, id],
    )?;
    Ok(())
}

pub fn delete_playlist(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn add_tracks_to_playlist(conn: &Connection, playlist_id: i64, track_ids: &[i64]) -> Result<()> {
    // Get the current max position
    let max_pos: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), 0) FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist_id],
            |row| row.get(0),
        )?;

    let mut stmt = conn.prepare(
        "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?1, ?2, ?3)",
    )?;

    for (i, &track_id) in track_ids.iter().enumerate() {
        stmt.execute(params![playlist_id, track_id, max_pos + 1 + i as i32])?;
    }
    Ok(())
}

pub fn remove_from_playlist(conn: &Connection, entry_ids: &[i64]) -> Result<()> {
    if entry_ids.is_empty() {
        return Ok(());
    }
    // Delete entries
    let placeholders: Vec<String> = entry_ids.iter().map(|_| "?".to_string()).collect();
    let sql = format!(
        "DELETE FROM playlist_tracks WHERE id IN ({})",
        placeholders.join(", ")
    );
    let params: Vec<Box<dyn rusqlite::types::ToSql>> = entry_ids
        .iter()
        .map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>)
        .collect();
    conn.execute(&sql, rusqlite::params_from_iter(params.iter().map(|b| b.as_ref())))?;

    // Compact positions per playlist: re-number all remaining entries
    // We need to find affected playlists first — but since we already deleted,
    // just re-number all playlists that have gaps. For simplicity, do it globally.
    compact_all_playlist_positions(conn)?;

    Ok(())
}

fn compact_all_playlist_positions(conn: &Connection) -> Result<()> {
    let playlist_ids: Vec<i64> = {
        let mut stmt = conn.prepare("SELECT DISTINCT playlist_id FROM playlist_tracks")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect::<Result<Vec<_>>>()?
    };

    for pid in playlist_ids {
        compact_playlist_positions(conn, pid)?;
    }
    Ok(())
}

fn compact_playlist_positions(conn: &Connection, playlist_id: i64) -> Result<()> {
    let entries: Vec<i64> = {
        let mut stmt = conn.prepare(
            "SELECT id FROM playlist_tracks WHERE playlist_id = ?1 ORDER BY position, id",
        )?;
        let rows = stmt.query_map(params![playlist_id], |row| row.get(0))?;
        rows.collect::<Result<Vec<_>>>()?
    };

    let mut update_stmt =
        conn.prepare("UPDATE playlist_tracks SET position = ?1 WHERE id = ?2")?;
    for (i, entry_id) in entries.iter().enumerate() {
        update_stmt.execute(params![i as i32 + 1, entry_id])?;
    }
    Ok(())
}

pub fn reorder_playlist(conn: &Connection, playlist_id: i64, entry_ids: &[i64]) -> Result<()> {
    let mut stmt = conn.prepare(
        "UPDATE playlist_tracks SET position = ?1 WHERE id = ?2 AND playlist_id = ?3",
    )?;
    for (i, &entry_id) in entry_ids.iter().enumerate() {
        stmt.execute(params![i as i32 + 1, entry_id, playlist_id])?;
    }
    Ok(())
}

pub fn get_playlist_tracks(conn: &Connection, playlist_id: i64) -> Result<Vec<PlaylistEntry>> {
    let mut stmt = conn.prepare(
        "SELECT pt.id as entry_id, pt.track_id, pt.position, t.*
         FROM playlist_tracks pt
         JOIN tracks t ON t.id = pt.track_id
         WHERE pt.playlist_id = ?1
         ORDER BY pt.position",
    )?;

    let rows = stmt.query_map(params![playlist_id], |row| {
        let track = row_to_track_with_offset(row, 3)?;
        Ok(PlaylistEntry {
            id: row.get("entry_id")?,
            track_id: row.get("track_id")?,
            position: row.get("position")?,
            track,
        })
    })?;
    rows.collect()
}

/// Parse a Track from a row where track columns start at the given column offset.
fn row_to_track_with_offset(row: &rusqlite::Row, offset: usize) -> Result<crate::models::Track> {
    use super::tracks::energy_from_bytes;
    let energy_blob: Option<Vec<u8>> = row.get(offset + 15)?;
    let energy_vector = energy_blob.and_then(|b| energy_from_bytes(&b));
    Ok(crate::models::Track {
        id: row.get(offset)?,
        file_path: row.get(offset + 1)?,
        title: row.get(offset + 2)?,
        artist: row.get(offset + 3)?,
        album: row.get(offset + 4)?,
        album_artist: row.get(offset + 5)?,
        genre: row.get(offset + 6)?,
        year: row.get(offset + 7)?,
        track_number: row.get(offset + 8)?,
        disc_number: row.get(offset + 9)?,
        duration_secs: row.get(offset + 10)?,
        sample_rate: row.get(offset + 11)?,
        bitrate: row.get(offset + 12)?,
        format: row.get(offset + 13)?,
        file_size: row.get(offset + 14)?,
        energy_vector,
        created_at: row.get(offset + 16)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_in_memory;
    use crate::db::tracks::insert_track;
    use crate::models::NewTrack;

    fn make_test_track(path: &str) -> NewTrack {
        NewTrack {
            file_path: path.to_string(),
            title: Some("Song".to_string()),
            artist: Some("Artist".to_string()),
            album: None,
            album_artist: None,
            genre: None,
            year: None,
            track_number: None,
            disc_number: None,
            duration_secs: Some(200.0),
            sample_rate: None,
            bitrate: None,
            format: None,
            file_size: None,
            energy_vector: None,
        }
    }

    #[test]
    fn test_create_and_list_playlists() {
        let conn = open_db_in_memory().unwrap();
        create_playlist(&conn, "Playlist A").unwrap();
        create_playlist(&conn, "Playlist B").unwrap();

        let playlists = list_playlists(&conn).unwrap();
        assert_eq!(playlists.len(), 2);
        assert_eq!(playlists[0].name, "Playlist A");
        assert_eq!(playlists[1].name, "Playlist B");
    }

    #[test]
    fn test_delete_playlist_cascades() {
        let conn = open_db_in_memory().unwrap();
        let pid = create_playlist(&conn, "To Delete").unwrap();
        let tid = insert_track(&conn, &make_test_track("/a.mp3")).unwrap().unwrap();
        add_tracks_to_playlist(&conn, pid, &[tid]).unwrap();

        delete_playlist(&conn, pid).unwrap();

        // Playlist tracks should be gone too
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = ?1",
                params![pid],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_add_tracks_and_get_playlist_tracks() {
        let conn = open_db_in_memory().unwrap();
        let pid = create_playlist(&conn, "My Playlist").unwrap();
        let t1 = insert_track(&conn, &make_test_track("/a.mp3")).unwrap().unwrap();
        let t2 = insert_track(&conn, &make_test_track("/b.mp3")).unwrap().unwrap();

        add_tracks_to_playlist(&conn, pid, &[t1, t2]).unwrap();

        let entries = get_playlist_tracks(&conn, pid).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].position, 1);
        assert_eq!(entries[1].position, 2);
        assert_eq!(entries[0].track.file_path, "/a.mp3");
    }

    #[test]
    fn test_add_duplicate_track_allowed() {
        let conn = open_db_in_memory().unwrap();
        let pid = create_playlist(&conn, "Dups OK").unwrap();
        let tid = insert_track(&conn, &make_test_track("/a.mp3")).unwrap().unwrap();

        add_tracks_to_playlist(&conn, pid, &[tid, tid]).unwrap();

        let entries = get_playlist_tracks(&conn, pid).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_remove_from_playlist_compacts_positions() {
        let conn = open_db_in_memory().unwrap();
        let pid = create_playlist(&conn, "Remove Test").unwrap();
        let t1 = insert_track(&conn, &make_test_track("/a.mp3")).unwrap().unwrap();
        let t2 = insert_track(&conn, &make_test_track("/b.mp3")).unwrap().unwrap();
        let t3 = insert_track(&conn, &make_test_track("/c.mp3")).unwrap().unwrap();

        add_tracks_to_playlist(&conn, pid, &[t1, t2, t3]).unwrap();

        let entries = get_playlist_tracks(&conn, pid).unwrap();
        // Remove the middle entry
        remove_from_playlist(&conn, &[entries[1].id]).unwrap();

        let remaining = get_playlist_tracks(&conn, pid).unwrap();
        assert_eq!(remaining.len(), 2);
        assert_eq!(remaining[0].position, 1);
        assert_eq!(remaining[1].position, 2);
    }

    #[test]
    fn test_reorder_playlist() {
        let conn = open_db_in_memory().unwrap();
        let pid = create_playlist(&conn, "Reorder").unwrap();
        let t1 = insert_track(&conn, &make_test_track("/a.mp3")).unwrap().unwrap();
        let t2 = insert_track(&conn, &make_test_track("/b.mp3")).unwrap().unwrap();
        let t3 = insert_track(&conn, &make_test_track("/c.mp3")).unwrap().unwrap();

        add_tracks_to_playlist(&conn, pid, &[t1, t2, t3]).unwrap();

        let entries = get_playlist_tracks(&conn, pid).unwrap();
        // Reverse order
        let new_order = vec![entries[2].id, entries[1].id, entries[0].id];
        reorder_playlist(&conn, pid, &new_order).unwrap();

        let reordered = get_playlist_tracks(&conn, pid).unwrap();
        assert_eq!(reordered[0].track.file_path, "/c.mp3");
        assert_eq!(reordered[1].track.file_path, "/b.mp3");
        assert_eq!(reordered[2].track.file_path, "/a.mp3");
    }
}
