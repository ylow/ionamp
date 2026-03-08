use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};

use super::tracks::row_to_track;
use crate::models::Track;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub filters: Vec<FilterPredicate>,
    pub group_by: Option<GroupByField>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterPredicate {
    Artist(String),
    Album(String),
    Genre(String),
    Year(i32),
    Tag { category_id: i64, value_id: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupByField {
    Artist,
    Album,
    Genre,
    Year,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub groups: Vec<TrackGroup>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackGroup {
    pub key: String,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOptions {
    pub artists: Vec<String>,
    pub albums: Vec<String>,
    pub genres: Vec<String>,
    pub years: Vec<i32>,
}

pub fn search_tracks(conn: &Connection, query: &SearchQuery) -> Result<SearchResult> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut needs_tag_join = false;

    // FTS text search
    if let Some(ref text) = query.text {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            conditions.push(
                "t.id IN (SELECT rowid FROM tracks_fts WHERE tracks_fts MATCH ?)".to_string(),
            );
            // Add wildcard for prefix matching
            let fts_query = format!("{}*", trimmed.replace('"', "\"\""));
            params.push(Box::new(fts_query));
        }
    }

    // Filters
    for filter in &query.filters {
        match filter {
            FilterPredicate::Artist(v) => {
                conditions.push("t.artist = ?".to_string());
                params.push(Box::new(v.clone()));
            }
            FilterPredicate::Album(v) => {
                conditions.push("t.album = ?".to_string());
                params.push(Box::new(v.clone()));
            }
            FilterPredicate::Genre(v) => {
                conditions.push("t.genre = ?".to_string());
                params.push(Box::new(v.clone()));
            }
            FilterPredicate::Year(y) => {
                conditions.push("t.year = ?".to_string());
                params.push(Box::new(*y));
            }
            FilterPredicate::Tag {
                value_id,
                ..
            } => {
                needs_tag_join = true;
                conditions.push("tt.tag_value_id = ?".to_string());
                params.push(Box::new(*value_id));
            }
        }
    }

    let join_clause = if needs_tag_join {
        "JOIN track_tags tt ON tt.track_id = t.id"
    } else {
        ""
    };

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Count total
    let count_sql = format!(
        "SELECT COUNT(DISTINCT t.id) FROM tracks t {} {}",
        join_clause, where_clause
    );
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let total_count: i64 =
        conn.query_row(&count_sql, rusqlite::params_from_iter(param_refs.iter().copied()), |row| row.get(0))?;

    // Build the query based on grouping
    let group_by_col = query.group_by.as_ref().map(|g| match g {
        GroupByField::Artist => "t.artist",
        GroupByField::Album => "t.album",
        GroupByField::Genre => "t.genre",
        GroupByField::Year => "CAST(t.year AS TEXT)",
    });

    let order_clause = if let Some(col) = group_by_col {
        format!("ORDER BY {} ASC, t.title ASC", col)
    } else {
        "ORDER BY t.title ASC".to_string()
    };

    let limit_clause = match (query.limit, query.offset) {
        (Some(limit), Some(offset)) => format!("LIMIT {} OFFSET {}", limit, offset),
        (Some(limit), None) => format!("LIMIT {}", limit),
        _ => String::new(),
    };

    let select_sql = format!(
        "SELECT DISTINCT t.* FROM tracks t {} {} {} {}",
        join_clause, where_clause, order_clause, limit_clause
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&select_sql)?;
    let tracks: Vec<Track> = stmt
        .query_map(rusqlite::params_from_iter(param_refs.iter().copied()), row_to_track)?
        .collect::<Result<Vec<_>>>()?;

    // Group results
    let groups = if let Some(ref group_by) = query.group_by {
        let key_fn = |t: &Track| -> String {
            match group_by {
                GroupByField::Artist => t.artist.clone().unwrap_or_else(|| "Unknown".to_string()),
                GroupByField::Album => t.album.clone().unwrap_or_else(|| "Unknown".to_string()),
                GroupByField::Genre => t.genre.clone().unwrap_or_else(|| "Unknown".to_string()),
                GroupByField::Year => t
                    .year
                    .map(|y| y.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
            }
        };

        let mut group_map: Vec<(String, Vec<Track>)> = Vec::new();
        for track in tracks {
            let key = key_fn(&track);
            if let Some(entry) = group_map.iter_mut().find(|(k, _)| k == &key) {
                entry.1.push(track);
            } else {
                group_map.push((key, vec![track]));
            }
        }

        group_map
            .into_iter()
            .map(|(key, tracks)| TrackGroup { key, tracks })
            .collect()
    } else {
        vec![TrackGroup {
            key: String::new(),
            tracks,
        }]
    };

    Ok(SearchResult {
        groups,
        total_count,
    })
}

pub fn get_filter_options(conn: &Connection) -> Result<FilterOptions> {
    let artists = get_distinct_values(conn, "artist")?;
    let albums = get_distinct_values(conn, "album")?;
    let genres = get_distinct_values(conn, "genre")?;

    let mut stmt = conn.prepare("SELECT DISTINCT year FROM tracks WHERE year IS NOT NULL ORDER BY year")?;
    let years: Vec<i32> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>>>()?;

    Ok(FilterOptions {
        artists,
        albums,
        genres,
        years,
    })
}

fn get_distinct_values(conn: &Connection, column: &str) -> Result<Vec<String>> {
    let sql = format!(
        "SELECT DISTINCT {} FROM tracks WHERE {} IS NOT NULL ORDER BY {}",
        column, column, column
    );
    let mut stmt = conn.prepare(&sql)?;
    let values: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>>>()?;
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_in_memory;
    use crate::db::tags;
    use crate::db::tracks::insert_track;
    use crate::models::NewTrack;

    fn seed_tracks(conn: &Connection) {
        let tracks = vec![
            NewTrack {
                file_path: "/a.mp3".into(),
                title: Some("Alpha".into()),
                artist: Some("Band A".into()),
                album: Some("Album 1".into()),
                genre: Some("Rock".into()),
                year: Some(2020),
                ..default_track()
            },
            NewTrack {
                file_path: "/b.mp3".into(),
                title: Some("Beta".into()),
                artist: Some("Band B".into()),
                album: Some("Album 2".into()),
                genre: Some("Jazz".into()),
                year: Some(2021),
                ..default_track()
            },
            NewTrack {
                file_path: "/c.mp3".into(),
                title: Some("Charlie".into()),
                artist: Some("Band A".into()),
                album: Some("Album 1".into()),
                genre: Some("Rock".into()),
                year: Some(2020),
                ..default_track()
            },
        ];
        for t in &tracks {
            insert_track(conn, t).unwrap();
        }
    }

    fn default_track() -> NewTrack {
        NewTrack {
            file_path: String::new(),
            title: None,
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
            energy_rms: None,
            energy_centroid: None,
            energy_onset: None,
        }
    }

    #[test]
    fn test_search_empty_returns_all() {
        let conn = open_db_in_memory().unwrap();
        seed_tracks(&conn);

        let result = search_tracks(
            &conn,
            &SearchQuery {
                text: None,
                filters: vec![],
                group_by: None,
                limit: None,
                offset: None,
            },
        )
        .unwrap();

        assert_eq!(result.total_count, 3);
        assert_eq!(result.groups[0].tracks.len(), 3);
    }

    #[test]
    fn test_search_fts_match() {
        let conn = open_db_in_memory().unwrap();
        seed_tracks(&conn);

        let result = search_tracks(
            &conn,
            &SearchQuery {
                text: Some("Alpha".into()),
                filters: vec![],
                group_by: None,
                limit: None,
                offset: None,
            },
        )
        .unwrap();

        assert_eq!(result.total_count, 1);
        assert_eq!(result.groups[0].tracks[0].title.as_deref(), Some("Alpha"));
    }

    #[test]
    fn test_filter_by_artist() {
        let conn = open_db_in_memory().unwrap();
        seed_tracks(&conn);

        let result = search_tracks(
            &conn,
            &SearchQuery {
                text: None,
                filters: vec![FilterPredicate::Artist("Band A".into())],
                group_by: None,
                limit: None,
                offset: None,
            },
        )
        .unwrap();

        assert_eq!(result.total_count, 2);
    }

    #[test]
    fn test_filter_by_tag() {
        let conn = open_db_in_memory().unwrap();
        seed_tracks(&conn);

        let cat_id = tags::create_category(&conn, "Mood").unwrap();
        let val_id = tags::create_tag_value(&conn, cat_id, "Happy").unwrap();

        // Tag only the first track
        tags::tag_tracks(&conn, &[1], val_id).unwrap();

        let result = search_tracks(
            &conn,
            &SearchQuery {
                text: None,
                filters: vec![FilterPredicate::Tag {
                    category_id: cat_id,
                    value_id: val_id,
                }],
                group_by: None,
                limit: None,
                offset: None,
            },
        )
        .unwrap();

        assert_eq!(result.total_count, 1);
    }

    #[test]
    fn test_combined_search_and_filter() {
        let conn = open_db_in_memory().unwrap();
        seed_tracks(&conn);

        let result = search_tracks(
            &conn,
            &SearchQuery {
                text: Some("Rock".into()),
                filters: vec![FilterPredicate::Artist("Band A".into())],
                group_by: None,
                limit: None,
                offset: None,
            },
        )
        .unwrap();

        // Both Alpha and Charlie match (Rock genre, Band A artist)
        assert_eq!(result.total_count, 2);
    }

    #[test]
    fn test_group_by_artist() {
        let conn = open_db_in_memory().unwrap();
        seed_tracks(&conn);

        let result = search_tracks(
            &conn,
            &SearchQuery {
                text: None,
                filters: vec![],
                group_by: Some(GroupByField::Artist),
                limit: None,
                offset: None,
            },
        )
        .unwrap();

        assert_eq!(result.groups.len(), 2);
        assert_eq!(result.groups[0].key, "Band A");
        assert_eq!(result.groups[0].tracks.len(), 2);
        assert_eq!(result.groups[1].key, "Band B");
        assert_eq!(result.groups[1].tracks.len(), 1);
    }

    #[test]
    fn test_pagination() {
        let conn = open_db_in_memory().unwrap();
        seed_tracks(&conn);

        let result = search_tracks(
            &conn,
            &SearchQuery {
                text: None,
                filters: vec![],
                group_by: None,
                limit: Some(2),
                offset: Some(0),
            },
        )
        .unwrap();

        assert_eq!(result.total_count, 3);
        assert_eq!(result.groups[0].tracks.len(), 2);

        let result2 = search_tracks(
            &conn,
            &SearchQuery {
                text: None,
                filters: vec![],
                group_by: None,
                limit: Some(2),
                offset: Some(2),
            },
        )
        .unwrap();

        assert_eq!(result2.groups[0].tracks.len(), 1);
    }

    #[test]
    fn test_get_filter_options() {
        let conn = open_db_in_memory().unwrap();
        seed_tracks(&conn);

        let options = get_filter_options(&conn).unwrap();
        assert_eq!(options.artists, vec!["Band A", "Band B"]);
        assert_eq!(options.albums, vec!["Album 1", "Album 2"]);
        assert_eq!(options.genres, vec!["Jazz", "Rock"]);
        assert_eq!(options.years, vec![2020, 2021]);
    }
}
