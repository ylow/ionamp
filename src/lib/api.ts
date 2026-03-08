import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  Track,
  Playlist,
  PlaylistEntry,
  SearchQuery,
  SearchResult,
  FilterOptions,
  TagCategory,
  TagValue,
  ImportEvent,
  EnergyClusterGroup,
} from "./types";

// ── Import ──────────────────────────────────────────────────────────────

export async function importFiles(
  paths: string[],
  onEvent: (event: ImportEvent) => void,
): Promise<void> {
  const channel = new Channel<ImportEvent>();
  channel.onmessage = onEvent;
  await invoke("import_files", { paths, onEvent: channel });
}

// ── Search ──────────────────────────────────────────────────────────────

export async function searchTracks(query: SearchQuery): Promise<SearchResult> {
  return invoke("search_tracks", { query });
}

export async function getFilterOptions(): Promise<FilterOptions> {
  return invoke("get_filter_options");
}

// ── Tracks ──────────────────────────────────────────────────────────────

export async function getTrack(id: number): Promise<Track | null> {
  return invoke("get_track", { id });
}

export async function deleteTracks(ids: number[]): Promise<void> {
  return invoke("delete_tracks", { ids });
}

// ── Playlists ───────────────────────────────────────────────────────────

export async function listPlaylists(): Promise<Playlist[]> {
  return invoke("list_playlists");
}

export async function createPlaylist(name: string): Promise<number> {
  return invoke("create_playlist", { name });
}

export async function renamePlaylist(
  id: number,
  newName: string,
): Promise<void> {
  return invoke("rename_playlist", { id, newName });
}

export async function deletePlaylist(id: number): Promise<void> {
  return invoke("delete_playlist", { id });
}

export async function getPlaylistTracks(
  playlistId: number,
): Promise<PlaylistEntry[]> {
  return invoke("get_playlist_tracks", { playlistId });
}

export async function addToPlaylist(
  playlistId: number,
  trackIds: number[],
): Promise<void> {
  return invoke("add_to_playlist", { playlistId, trackIds });
}

export async function removeFromPlaylist(entryIds: number[]): Promise<void> {
  return invoke("remove_from_playlist", { entryIds });
}

export async function reorderPlaylist(
  playlistId: number,
  entryIds: number[],
): Promise<void> {
  return invoke("reorder_playlist", { playlistId, entryIds });
}

// ── Tags ────────────────────────────────────────────────────────────────

export async function listTagCategories(): Promise<TagCategory[]> {
  return invoke("list_tag_categories");
}

export async function createTagCategory(name: string): Promise<number> {
  return invoke("create_tag_category", { name });
}

export async function deleteTagCategory(id: number): Promise<void> {
  return invoke("delete_tag_category", { id });
}

export async function tagTracks(
  trackIds: number[],
  tagValueId: number,
): Promise<void> {
  return invoke("tag_tracks", { trackIds, tagValueId });
}

export async function untagTracks(
  trackIds: number[],
  tagValueId: number,
): Promise<void> {
  return invoke("untag_tracks", { trackIds, tagValueId });
}

export async function getTrackTags(trackId: number): Promise<TagValue[]> {
  return invoke("get_track_tags", { trackId });
}

// ── Energy Clustering ───────────────────────────────────────────────────

export async function clusterByEnergy(
  query: SearchQuery,
): Promise<EnergyClusterGroup[]> {
  return invoke("cluster_by_energy", { query });
}
