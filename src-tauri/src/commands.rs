use std::path::PathBuf;

use tauri::{ipc::Channel, State};

use crate::audio::kmeans::cluster_energy_vectors;
use crate::db::{playlists, search, tags, tracks};
use crate::import::{self, ImportEvent};
use crate::models::*;
use crate::playback::{PlaybackStatus, SharedPlayback};
use crate::state::AppState;

type CmdResult<T> = Result<T, String>;

fn map_err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

// ── Import ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn import_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
    on_event: Channel<ImportEvent>,
) -> CmdResult<()> {
    let db_path = state.db_path.clone();
    let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();

    // Run on a blocking thread with its own DB connection so channel
    // events can be delivered to the frontend while import is in progress.
    tauri::async_runtime::spawn_blocking(move || {
        let conn = crate::db::open_db(&db_path).map_err(map_err)?;
        let files = import::scan_audio_files(&path_bufs);
        import::run_import(&conn, &files, |event| {
            let _ = on_event.send(event);
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

// ── Search ──────────────────────────────────────────────────────────────

#[tauri::command]
pub fn search_tracks(
    state: State<'_, AppState>,
    query: search::SearchQuery,
) -> CmdResult<search::SearchResult> {
    let conn = state.db.lock().map_err(map_err)?;
    search::search_tracks(&conn, &query).map_err(map_err)
}

#[tauri::command]
pub fn get_filter_options(state: State<'_, AppState>) -> CmdResult<search::FilterOptions> {
    let conn = state.db.lock().map_err(map_err)?;
    search::get_filter_options(&conn).map_err(map_err)
}

// ── Tracks ──────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_track(state: State<'_, AppState>, id: i64) -> CmdResult<Option<Track>> {
    let conn = state.db.lock().map_err(map_err)?;
    tracks::get_track_by_id(&conn, id).map_err(map_err)
}

#[tauri::command]
pub fn delete_tracks(state: State<'_, AppState>, ids: Vec<i64>) -> CmdResult<()> {
    let conn = state.db.lock().map_err(map_err)?;
    for id in ids {
        tracks::delete_track(&conn, id).map_err(map_err)?;
    }
    Ok(())
}

// ── Playlists ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_playlists(state: State<'_, AppState>) -> CmdResult<Vec<Playlist>> {
    let conn = state.db.lock().map_err(map_err)?;
    playlists::list_playlists(&conn).map_err(map_err)
}

#[tauri::command]
pub fn create_playlist(state: State<'_, AppState>, name: String) -> CmdResult<i64> {
    let conn = state.db.lock().map_err(map_err)?;
    playlists::create_playlist(&conn, &name).map_err(map_err)
}

#[tauri::command]
pub fn rename_playlist(state: State<'_, AppState>, id: i64, new_name: String) -> CmdResult<()> {
    let conn = state.db.lock().map_err(map_err)?;
    playlists::rename_playlist(&conn, id, &new_name).map_err(map_err)
}

#[tauri::command]
pub fn delete_playlist(state: State<'_, AppState>, id: i64) -> CmdResult<()> {
    let conn = state.db.lock().map_err(map_err)?;
    playlists::delete_playlist(&conn, id).map_err(map_err)
}

#[tauri::command]
pub fn get_playlist_tracks(
    state: State<'_, AppState>,
    playlist_id: i64,
) -> CmdResult<Vec<PlaylistEntry>> {
    let conn = state.db.lock().map_err(map_err)?;
    playlists::get_playlist_tracks(&conn, playlist_id).map_err(map_err)
}

#[tauri::command]
pub fn add_to_playlist(
    state: State<'_, AppState>,
    playlist_id: i64,
    track_ids: Vec<i64>,
) -> CmdResult<()> {
    let conn = state.db.lock().map_err(map_err)?;
    playlists::add_tracks_to_playlist(&conn, playlist_id, &track_ids).map_err(map_err)
}

#[tauri::command]
pub fn remove_from_playlist(state: State<'_, AppState>, entry_ids: Vec<i64>) -> CmdResult<()> {
    let conn = state.db.lock().map_err(map_err)?;
    playlists::remove_from_playlist(&conn, &entry_ids).map_err(map_err)
}

#[tauri::command]
pub fn reorder_playlist(
    state: State<'_, AppState>,
    playlist_id: i64,
    entry_ids: Vec<i64>,
) -> CmdResult<()> {
    let conn = state.db.lock().map_err(map_err)?;
    playlists::reorder_playlist(&conn, playlist_id, &entry_ids).map_err(map_err)
}

// ── Tags ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_tag_categories(state: State<'_, AppState>) -> CmdResult<Vec<TagCategory>> {
    let conn = state.db.lock().map_err(map_err)?;
    tags::list_categories(&conn).map_err(map_err)
}

#[tauri::command]
pub fn create_tag_category(state: State<'_, AppState>, name: String) -> CmdResult<i64> {
    let conn = state.db.lock().map_err(map_err)?;
    tags::create_category(&conn, &name).map_err(map_err)
}

#[tauri::command]
pub fn delete_tag_category(state: State<'_, AppState>, id: i64) -> CmdResult<()> {
    let conn = state.db.lock().map_err(map_err)?;
    tags::delete_category(&conn, id).map_err(map_err)
}

#[tauri::command]
pub fn get_values_for_category(
    state: State<'_, AppState>,
    category_id: i64,
) -> CmdResult<Vec<TagValue>> {
    let conn = state.db.lock().map_err(map_err)?;
    tags::get_values_for_category(&conn, category_id).map_err(map_err)
}

#[tauri::command]
pub fn create_tag_value(
    state: State<'_, AppState>,
    category_id: i64,
    value: String,
) -> CmdResult<i64> {
    let conn = state.db.lock().map_err(map_err)?;
    tags::create_tag_value(&conn, category_id, &value).map_err(map_err)
}

#[tauri::command]
pub fn tag_tracks(
    state: State<'_, AppState>,
    track_ids: Vec<i64>,
    tag_value_id: i64,
) -> CmdResult<()> {
    let conn = state.db.lock().map_err(map_err)?;
    tags::tag_tracks(&conn, &track_ids, tag_value_id).map_err(map_err)
}

#[tauri::command]
pub fn untag_tracks(
    state: State<'_, AppState>,
    track_ids: Vec<i64>,
    tag_value_id: i64,
) -> CmdResult<()> {
    let conn = state.db.lock().map_err(map_err)?;
    tags::untag_tracks(&conn, &track_ids, tag_value_id).map_err(map_err)
}

#[tauri::command]
pub fn get_track_tags(state: State<'_, AppState>, track_id: i64) -> CmdResult<Vec<TagValue>> {
    let conn = state.db.lock().map_err(map_err)?;
    tags::get_tags_for_track(&conn, track_id).map_err(map_err)
}

// ── Energy Clustering ───────────────────────────────────────────────────

#[tauri::command]
pub fn cluster_by_energy(
    state: State<'_, AppState>,
    query: search::SearchQuery,
    seed: u64,
) -> CmdResult<Vec<EnergyClusterGroup>> {
    let conn = state.db.lock().map_err(map_err)?;

    // Search with no grouping to get all matching tracks
    let mut flat_query = query;
    flat_query.group_by = None;
    flat_query.limit = None;
    flat_query.offset = None;

    let result = search::search_tracks(&conn, &flat_query).map_err(map_err)?;
    let all_tracks: Vec<Track> = result.groups.into_iter().flat_map(|g| g.tracks).collect();

    // Collect concatenated [rms ++ centroid ++ onset] vectors (384-dim) for clustering
    let mut vectors: Vec<Vec<f32>> = Vec::new();
    let mut indices: Vec<usize> = Vec::new();

    for (i, track) in all_tracks.iter().enumerate() {
        if let (Some(rms), Some(cent), Some(onset)) =
            (&track.energy_rms, &track.energy_centroid, &track.energy_onset)
        {
            let mut concat = Vec::with_capacity(rms.len() + cent.len() + onset.len());
            concat.extend(rms);
            concat.extend(cent);
            concat.extend(onset);
            vectors.push(concat);
            indices.push(i);
        }
    }

    if vectors.len() < 2 {
        // Not enough energy data to cluster, return all as one group
        return Ok(vec![EnergyClusterGroup {
            label: "All Tracks".to_string(),
            tracks: all_tracks,
        }]);
    }

    let clusters = cluster_energy_vectors(&vectors, seed);

    let groups: Vec<EnergyClusterGroup> = clusters
        .into_iter()
        .map(|c| {
            let cluster_tracks: Vec<Track> = c
                .member_indices
                .iter()
                .map(|&mi| all_tracks[indices[mi]].clone())
                .collect();
            EnergyClusterGroup {
                label: c.label,
                tracks: cluster_tracks,
            }
        })
        .collect();

    Ok(groups)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnergyClusterGroup {
    pub label: String,
    pub tracks: Vec<Track>,
}

// ── Playback ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn play_file(
    playback: State<'_, SharedPlayback>,
    path: String,
    title: Option<String>,
    artist: Option<String>,
    duration_secs: f64,
) -> CmdResult<()> {
    let mut pb = playback.lock().map_err(map_err)?;
    pb.play_file(&path, title, artist, duration_secs)
}

#[tauri::command]
pub fn pause_playback(playback: State<'_, SharedPlayback>) -> CmdResult<()> {
    let pb = playback.lock().map_err(map_err)?;
    pb.pause();
    Ok(())
}

#[tauri::command]
pub fn resume_playback(playback: State<'_, SharedPlayback>) -> CmdResult<()> {
    let pb = playback.lock().map_err(map_err)?;
    pb.resume();
    Ok(())
}

#[tauri::command]
pub fn stop_playback(playback: State<'_, SharedPlayback>) -> CmdResult<()> {
    let mut pb = playback.lock().map_err(map_err)?;
    pb.stop();
    Ok(())
}

#[tauri::command]
pub fn seek_playback(playback: State<'_, SharedPlayback>, position_secs: f64) -> CmdResult<()> {
    let pb = playback.lock().map_err(map_err)?;
    pb.seek(position_secs)
}

#[tauri::command]
pub fn set_volume(playback: State<'_, SharedPlayback>, volume: f32) -> CmdResult<()> {
    let pb = playback.lock().map_err(map_err)?;
    pb.set_volume(volume);
    Ok(())
}

#[tauri::command]
pub fn get_playback_status(playback: State<'_, SharedPlayback>) -> CmdResult<PlaybackStatus> {
    let pb = playback.lock().map_err(map_err)?;
    Ok(pb.status())
}
