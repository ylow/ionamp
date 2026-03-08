export interface Track {
  id: number;
  file_path: string;
  title: string | null;
  artist: string | null;
  album: string | null;
  album_artist: string | null;
  genre: string | null;
  year: number | null;
  track_number: number | null;
  disc_number: number | null;
  duration_secs: number | null;
  sample_rate: number | null;
  bitrate: number | null;
  format: string | null;
  file_size: number | null;
  energy_vector: number[] | null;
  created_at: string | null;
}

export interface Playlist {
  id: number;
  name: string;
  created_at: string | null;
}

export interface PlaylistEntry {
  id: number;
  track_id: number;
  position: number;
  track: Track;
}

export interface TagCategory {
  id: number;
  name: string;
}

export interface TagValue {
  id: number;
  category_id: number;
  category_name: string;
  value: string;
}

export interface SearchQuery {
  text: string | null;
  filters: FilterPredicate[];
  group_by: GroupByField | null;
  limit: number | null;
  offset: number | null;
}

export type FilterPredicate =
  | { Artist: string }
  | { Album: string }
  | { Genre: string }
  | { Year: number }
  | { Tag: { category_id: number; value_id: number } };

export type GroupByField = "Artist" | "Album" | "Genre" | "Year";

export interface SearchResult {
  groups: TrackGroup[];
  total_count: number;
}

export interface TrackGroup {
  key: string;
  tracks: Track[];
}

export interface FilterOptions {
  artists: string[];
  albums: string[];
  genres: string[];
  years: number[];
}

export interface ImportEvent {
  type: "ScanComplete" | "Progress" | "Skipped" | "Complete";
  total_files?: number;
  current?: number;
  total?: number;
  file_name?: string;
  reason?: string;
  imported?: number;
  skipped?: number;
  errors?: number;
}

export interface EnergyClusterGroup {
  label: string;
  tracks: Track[];
}
