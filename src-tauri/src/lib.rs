mod audio;
mod commands;
mod db;
mod import;
mod models;
mod playback;
mod state;

use state::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ionamp");
    std::fs::create_dir_all(&app_dir).expect("Failed to create app data directory");
    let db_path = app_dir.join("ionamp.db");

    let conn = db::open_db(&db_path).expect("Failed to open database");
    let playback_manager =
        playback::PlaybackManager::new().expect("Failed to initialize audio output");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            db: Mutex::new(conn),
            db_path: db_path.clone(),
        })
        .manage(Mutex::new(playback_manager) as playback::SharedPlayback)
        .invoke_handler(tauri::generate_handler![
            commands::import_files,
            commands::search_tracks,
            commands::get_filter_options,
            commands::get_track,
            commands::delete_tracks,
            commands::list_playlists,
            commands::create_playlist,
            commands::rename_playlist,
            commands::delete_playlist,
            commands::get_playlist_tracks,
            commands::add_to_playlist,
            commands::remove_from_playlist,
            commands::reorder_playlist,
            commands::list_tag_categories,
            commands::create_tag_category,
            commands::delete_tag_category,
            commands::get_values_for_category,
            commands::create_tag_value,
            commands::tag_tracks,
            commands::untag_tracks,
            commands::get_track_tags,
            commands::cluster_by_energy,
            commands::play_file,
            commands::pause_playback,
            commands::resume_playback,
            commands::stop_playback,
            commands::seek_playback,
            commands::set_volume,
            commands::get_playback_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
