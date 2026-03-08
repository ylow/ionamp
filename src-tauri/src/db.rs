use rusqlite::{Connection, Result};

pub fn open_db(path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    run_migrations(&conn)?;
    Ok(conn)
}

pub fn open_db_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    // journal_mode returns a result row, so use query_row
    let _: String = conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS tracks (
            id            INTEGER PRIMARY KEY,
            file_path     TEXT UNIQUE NOT NULL,
            title         TEXT,
            artist        TEXT,
            album         TEXT,
            album_artist  TEXT,
            genre         TEXT,
            year          INTEGER,
            track_number  INTEGER,
            disc_number   INTEGER,
            duration_secs REAL,
            sample_rate   INTEGER,
            bitrate       INTEGER,
            format        TEXT,
            file_size     INTEGER,
            energy_vector BLOB,
            created_at    TEXT DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist);
        CREATE INDEX IF NOT EXISTS idx_tracks_album ON tracks(album);
        CREATE INDEX IF NOT EXISTS idx_tracks_title ON tracks(title);
        CREATE INDEX IF NOT EXISTS idx_tracks_genre ON tracks(genre);
        CREATE INDEX IF NOT EXISTS idx_tracks_year ON tracks(year);

        CREATE TABLE IF NOT EXISTS tag_categories (
            id   INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tag_values (
            id          INTEGER PRIMARY KEY,
            category_id INTEGER NOT NULL REFERENCES tag_categories(id),
            value       TEXT NOT NULL,
            UNIQUE(category_id, value)
        );

        CREATE TABLE IF NOT EXISTS track_tags (
            track_id     INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
            tag_value_id INTEGER NOT NULL REFERENCES tag_values(id) ON DELETE CASCADE,
            PRIMARY KEY (track_id, tag_value_id)
        );

        CREATE TABLE IF NOT EXISTS playlists (
            id         INTEGER PRIMARY KEY,
            name       TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS playlist_tracks (
            id          INTEGER PRIMARY KEY,
            playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
            track_id    INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
            position    INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_playlist_tracks_playlist
            ON playlist_tracks(playlist_id, position);
        ",
    )?;

    // FTS5 virtual table — CREATE VIRTUAL TABLE doesn't support IF NOT EXISTS,
    // so check if it already exists first.
    let fts_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='tracks_fts'",
        [],
        |row| row.get(0),
    )?;

    if !fts_exists {
        conn.execute_batch(
            "
            CREATE VIRTUAL TABLE tracks_fts USING fts5(
                title, artist, album, genre,
                content=tracks, content_rowid=id
            );
            ",
        )?;
    }

    // FTS sync triggers — use IF NOT EXISTS via checking sqlite_master
    let trigger_exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='trigger' AND name='tracks_ai'",
        [],
        |row| row.get(0),
    )?;

    if !trigger_exists {
        conn.execute_batch(
            "
            CREATE TRIGGER tracks_ai AFTER INSERT ON tracks BEGIN
                INSERT INTO tracks_fts(rowid, title, artist, album, genre)
                VALUES (new.id, new.title, new.artist, new.album, new.genre);
            END;

            CREATE TRIGGER tracks_ad AFTER DELETE ON tracks BEGIN
                INSERT INTO tracks_fts(tracks_fts, rowid, title, artist, album, genre)
                VALUES ('delete', old.id, old.title, old.artist, old.album, old.genre);
            END;

            CREATE TRIGGER tracks_au AFTER UPDATE ON tracks BEGIN
                INSERT INTO tracks_fts(tracks_fts, rowid, title, artist, album, genre)
                VALUES ('delete', old.id, old.title, old.artist, old.album, old.genre);
                INSERT INTO tracks_fts(rowid, title, artist, album, genre)
                VALUES (new.id, new.title, new.artist, new.album, new.genre);
            END;
            ",
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_db_creates_tables() {
        let conn = open_db_in_memory().unwrap();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();

        assert!(tables.contains(&"tracks".to_string()));
        assert!(tables.contains(&"tag_categories".to_string()));
        assert!(tables.contains(&"tag_values".to_string()));
        assert!(tables.contains(&"track_tags".to_string()));
        assert!(tables.contains(&"playlists".to_string()));
        assert!(tables.contains(&"playlist_tracks".to_string()));
        assert!(tables.contains(&"tracks_fts".to_string()));
    }

    #[test]
    fn test_open_db_idempotent() {
        let conn = open_db_in_memory().unwrap();
        // Running migrations again on the same connection should not error
        run_migrations(&conn).unwrap();
    }

    #[test]
    fn test_fts_trigger_insert() {
        let conn = open_db_in_memory().unwrap();
        conn.execute(
            "INSERT INTO tracks (file_path, title, artist, album, genre) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["/test.mp3", "Test Song", "Test Artist", "Test Album", "Rock"],
        )
        .unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tracks_fts WHERE tracks_fts MATCH 'Test Song'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_fts_trigger_delete() {
        let conn = open_db_in_memory().unwrap();
        conn.execute(
            "INSERT INTO tracks (file_path, title, artist) VALUES (?1, ?2, ?3)",
            rusqlite::params!["/test.mp3", "Delete Me", "Artist"],
        )
        .unwrap();

        let id: i64 = conn.last_insert_rowid();
        conn.execute("DELETE FROM tracks WHERE id = ?1", rusqlite::params![id])
            .unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tracks_fts WHERE tracks_fts MATCH 'Delete'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_foreign_keys_enforced() {
        let conn = open_db_in_memory().unwrap();
        // Try inserting a track_tag with a non-existent track_id
        let result = conn.execute(
            "INSERT INTO track_tags (track_id, tag_value_id) VALUES (9999, 9999)",
            [],
        );
        assert!(result.is_err());
    }
}
