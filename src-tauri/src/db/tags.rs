use rusqlite::{params, Connection, Result};

use crate::models::{TagCategory, TagValue};

pub fn create_category(conn: &Connection, name: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO tag_categories (name) VALUES (?1)",
        params![name],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_categories(conn: &Connection) -> Result<Vec<TagCategory>> {
    let mut stmt = conn.prepare("SELECT id, name FROM tag_categories ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(TagCategory {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    rows.collect()
}

pub fn delete_category(conn: &Connection, id: i64) -> Result<()> {
    // Delete tag_values first (cascades to track_tags via FK)
    conn.execute(
        "DELETE FROM tag_values WHERE category_id = ?1",
        params![id],
    )?;
    conn.execute("DELETE FROM tag_categories WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn create_tag_value(conn: &Connection, category_id: i64, value: &str) -> Result<i64> {
    conn.execute(
        "INSERT INTO tag_values (category_id, value) VALUES (?1, ?2)",
        params![category_id, value],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_values_for_category(conn: &Connection, category_id: i64) -> Result<Vec<TagValue>> {
    let mut stmt = conn.prepare(
        "SELECT tv.id, tv.category_id, tc.name, tv.value
         FROM tag_values tv
         JOIN tag_categories tc ON tc.id = tv.category_id
         WHERE tv.category_id = ?1
         ORDER BY tv.value",
    )?;
    let rows = stmt.query_map(params![category_id], |row| {
        Ok(TagValue {
            id: row.get(0)?,
            category_id: row.get(1)?,
            category_name: row.get(2)?,
            value: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn tag_tracks(conn: &Connection, track_ids: &[i64], tag_value_id: i64) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO track_tags (track_id, tag_value_id) VALUES (?1, ?2)",
    )?;
    for &track_id in track_ids {
        stmt.execute(params![track_id, tag_value_id])?;
    }
    Ok(())
}

pub fn untag_tracks(conn: &Connection, track_ids: &[i64], tag_value_id: i64) -> Result<()> {
    let mut stmt = conn.prepare(
        "DELETE FROM track_tags WHERE track_id = ?1 AND tag_value_id = ?2",
    )?;
    for &track_id in track_ids {
        stmt.execute(params![track_id, tag_value_id])?;
    }
    Ok(())
}

pub fn get_tags_for_track(conn: &Connection, track_id: i64) -> Result<Vec<TagValue>> {
    let mut stmt = conn.prepare(
        "SELECT tv.id, tv.category_id, tc.name, tv.value
         FROM track_tags tt
         JOIN tag_values tv ON tv.id = tt.tag_value_id
         JOIN tag_categories tc ON tc.id = tv.category_id
         WHERE tt.track_id = ?1
         ORDER BY tc.name, tv.value",
    )?;
    let rows = stmt.query_map(params![track_id], |row| {
        Ok(TagValue {
            id: row.get(0)?,
            category_id: row.get(1)?,
            category_name: row.get(2)?,
            value: row.get(3)?,
        })
    })?;
    rows.collect()
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
            artist: None,
            album: None,
            album_artist: None,
            genre: None,
            year: None,
            track_number: None,
            disc_number: None,
            duration_secs: None,
            sample_rate: None,
            bitrate: None,
            format: None,
            file_size: None,
            energy_vector: None,
        }
    }

    #[test]
    fn test_create_category_and_value() {
        let conn = open_db_in_memory().unwrap();
        let cat_id = create_category(&conn, "Mood").unwrap();
        assert!(cat_id > 0);

        let val_id = create_tag_value(&conn, cat_id, "Happy").unwrap();
        assert!(val_id > 0);

        let values = get_values_for_category(&conn, cat_id).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].value, "Happy");
        assert_eq!(values[0].category_name, "Mood");
    }

    #[test]
    fn test_tag_and_untag_track() {
        let conn = open_db_in_memory().unwrap();
        let tid = insert_track(&conn, &make_test_track("/a.mp3")).unwrap().unwrap();
        let cat_id = create_category(&conn, "Mood").unwrap();
        let val_id = create_tag_value(&conn, cat_id, "Chill").unwrap();

        tag_tracks(&conn, &[tid], val_id).unwrap();
        let tags = get_tags_for_track(&conn, tid).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].value, "Chill");

        untag_tracks(&conn, &[tid], val_id).unwrap();
        let tags = get_tags_for_track(&conn, tid).unwrap();
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_get_tags_for_track() {
        let conn = open_db_in_memory().unwrap();
        let tid = insert_track(&conn, &make_test_track("/a.mp3")).unwrap().unwrap();

        let mood_id = create_category(&conn, "Mood").unwrap();
        let energy_id = create_category(&conn, "Energy").unwrap();
        let happy = create_tag_value(&conn, mood_id, "Happy").unwrap();
        let high = create_tag_value(&conn, energy_id, "High").unwrap();

        tag_tracks(&conn, &[tid], happy).unwrap();
        tag_tracks(&conn, &[tid], high).unwrap();

        let tags = get_tags_for_track(&conn, tid).unwrap();
        assert_eq!(tags.len(), 2);
        // Sorted by category name then value
        assert_eq!(tags[0].category_name, "Energy");
        assert_eq!(tags[1].category_name, "Mood");
    }

    #[test]
    fn test_delete_category_cascades() {
        let conn = open_db_in_memory().unwrap();
        let tid = insert_track(&conn, &make_test_track("/a.mp3")).unwrap().unwrap();
        let cat_id = create_category(&conn, "Mood").unwrap();
        let val_id = create_tag_value(&conn, cat_id, "Chill").unwrap();
        tag_tracks(&conn, &[tid], val_id).unwrap();

        delete_category(&conn, cat_id).unwrap();

        let tags = get_tags_for_track(&conn, tid).unwrap();
        assert_eq!(tags.len(), 0);

        let categories = list_categories(&conn).unwrap();
        assert_eq!(categories.len(), 0);
    }

    #[test]
    fn test_duplicate_tag_ignored() {
        let conn = open_db_in_memory().unwrap();
        let tid = insert_track(&conn, &make_test_track("/a.mp3")).unwrap().unwrap();
        let cat_id = create_category(&conn, "Mood").unwrap();
        let val_id = create_tag_value(&conn, cat_id, "Chill").unwrap();

        tag_tracks(&conn, &[tid], val_id).unwrap();
        tag_tracks(&conn, &[tid], val_id).unwrap(); // duplicate — should not error

        let tags = get_tags_for_track(&conn, tid).unwrap();
        assert_eq!(tags.len(), 1);
    }
}
