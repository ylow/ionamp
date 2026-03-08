# ionamp — Music Playlist Management App Design

## Overview

A local-first music playlist management app that stores track references and metadata in SQLite. The app does not store music files — it indexes them, computes energy profiles, and provides fast search, grouping, tagging, and playlist management in a dense two-pane UI.

Target: macOS primary, cross-platform (Windows, Linux) via Tauri.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Tauri Shell                        │
│  ┌───────────────────────────────────────────────┐  │
│  │              Svelte Frontend                   │  │
│  │  ┌─────────────────────┬─────────────────┐    │  │
│  │  │   Search Pane (L)   │ Playlist Pane(R)│    │  │
│  │  │  ┌───────────────┐  │ ┌─────────────┐ │    │  │
│  │  │  │ Filter Bar    │  │ │Playlist Sel. │ │    │  │
│  │  │  ├───────────────┤  │ ├─────────────┤ │    │  │
│  │  │  │ Track List    │  │ │Playlist Trks│ │    │  │
│  │  │  │ (grouped/flat)│  │ │             │ │    │  │
│  │  │  └───────────────┘  │ └─────────────┘ │    │  │
│  │  └─────────────────────┴─────────────────┘    │  │
│  │  ┌───────────────────────────────────────┐    │  │
│  │  │         Playback Bar (future)         │    │  │
│  │  └───────────────────────────────────────┘    │  │
│  └───────────────────────────────────────────────┘  │
│                                                     │
│  ┌───────────────────────────────────────────────┐  │
│  │              Rust Backend                      │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────┐  │  │
│  │  │  SQLite  │ │ Importer │ │Energy Compute│  │  │
│  │  │  (query) │ │ (ffmpeg) │ │   (DSP)      │  │  │
│  │  └──────────┘ └──────────┘ └──────────────┘  │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

Frontend communicates with backend via Tauri IPC commands. All heavy work (DB queries, FFmpeg, DSP) happens in Rust. The frontend is a thin view layer.

## Technology Stack

| Component | Choice | Rationale |
|---|---|---|
| Backend | Rust | Performance for DSP, safe concurrency, single binary |
| UI framework | Tauri v2 | Native webview, small footprint, cross-platform |
| Frontend | Svelte 5 | Minimal JS overhead, no virtual DOM, fast rendering |
| Database | SQLite via `rusqlite` | Single-file DB, FTS5 for search, zero setup |
| FFmpeg | `ffmpeg-next` crate | Linked library, bundled with app |
| DSP | Hand-rolled Rust | RMS + spectral centroid + onset, ~200 lines |
| K-means++ | Hand-rolled Rust | ~100 lines, no heavy ML dependency |
| Sparklines | Inline SVG in Svelte | Tiny `<path>` elements, renders fast at scale |
| Virtual scrolling | Svelte component | Needed for large track lists |
| Drag and drop | HTML5 DnD API | Native browser support, works in Tauri webview |
| CSS | Tailwind CSS | Fast styling, utility classes, consistent dense UI |

## Database Schema

```sql
CREATE TABLE tracks (
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
    energy_vector BLOB,                  -- 128 x f32 = 512 bytes
    created_at    TEXT DEFAULT (datetime('now'))
);

CREATE TABLE tag_categories (
    id   INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE tag_values (
    id          INTEGER PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES tag_categories(id),
    value       TEXT NOT NULL,
    UNIQUE(category_id, value)
);

CREATE TABLE track_tags (
    track_id     INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    tag_value_id INTEGER NOT NULL REFERENCES tag_values(id) ON DELETE CASCADE,
    PRIMARY KEY (track_id, tag_value_id)
);

CREATE TABLE playlists (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE playlist_tracks (
    id          INTEGER PRIMARY KEY,
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    track_id    INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    position    INTEGER NOT NULL
);

-- Indexes for fast search
CREATE INDEX idx_tracks_artist ON tracks(artist);
CREATE INDEX idx_tracks_album ON tracks(album);
CREATE INDEX idx_tracks_title ON tracks(title);
CREATE INDEX idx_tracks_genre ON tracks(genre);
CREATE INDEX idx_tracks_year ON tracks(year);
CREATE INDEX idx_playlist_tracks_playlist ON playlist_tracks(playlist_id, position);

-- Full-text search
CREATE VIRTUAL TABLE tracks_fts USING fts5(title, artist, album, genre, content=tracks, content_rowid=id);
```

Key decisions:
- `energy_vector` as BLOB: 128 x f32 stored as raw bytes (512 bytes per track)
- Tags are normalized: category -> values -> track mapping
- `playlist_tracks` allows duplicate tracks in the same playlist (no UNIQUE constraint on playlist_id + track_id)
- FTS5 virtual table for instant text search across title/artist/album/genre

## Search & Grouping

**Instant search:** FTS5 query on every keystroke (debounced ~50ms). For short/empty queries, falls back to returning all tracks with optional `LIKE` filtering.

**Filtering:** Filter bar has dropdowns for Artist, Album, Genre, Year, and user-defined tag categories. Filters are additive (AND). Backend constructs SQL dynamically from active filters + search text.

**Grouping:** When a grouping field is selected (e.g. "Artist"), the backend returns results structured as groups:
```json
{
  "groups": [
    { "key": "Daft Punk", "tracks": [...] },
    { "key": "Boards of Canada", "tracks": [...] }
  ]
}
```
Frontend renders each group as a collapsible section.

**Energy clustering:** When grouping by "Energy":
1. Load energy vectors of all currently filtered tracks
2. Run K-means++ in Rust
3. k = clamp(sqrt(n/2), 2, 10)
4. Return numbered clusters sorted by average energy (low to high)

## Import Flow

1. User clicks "Import" -> native file picker (Tauri dialog API) for files/folders
2. Recursive scan in Rust, collect files with audio extensions
3. Deduplicate by `file_path` against DB, skip already-indexed
4. For each new file:
   - Extract metadata via `ffmpeg-next` (libavformat)
   - Decode to mono PCM f32 at 22050 Hz via `ffmpeg-next` (libavcodec)
   - Compute 128-point energy vector (see Energy DSP section)
   - Insert into SQLite
5. Progress events streamed to frontend via Tauri events
6. Runs on background thread for UI responsiveness

## Energy DSP Computation

```
Input:  PCM f32 samples, mono, 22050 Hz
Output: f32[128] energy vector, normalized to [0.0, 1.0]

1. Divide samples into 128 equal-length segments
2. For each segment:
   a. RMS = sqrt(mean(samples^2))                — loudness
   b. Spectral centroid via small FFT             — brightness
   c. Onset strength = mean(|diff(sub-window RMS)|) — rhythmic activity
   d. energy[i] = 0.5 * RMS_norm + 0.3 * centroid_norm + 0.2 * onset_norm
3. Normalize final vector to [0.0, 1.0]
4. Store as BLOB (128 x 4 bytes = 512 bytes)
```

## Frontend UI

**Left Pane (Search, ~65% width, resizable):**
- Top: Text search input + filter dropdowns + "Group by" dropdown
- Track list: Virtualized scrolling. Each row: Title | Artist | Album | Duration | energy sparkline (~80px inline SVG)
- Grouped mode: Collapsible sections with header and track count
- Multi-select: Click, Shift+click, Cmd+click

**Right Pane (Playlists, ~35% width):**
- Top: Playlist selector (pick/create/delete playlists)
- Playlist track list: Ordered, reorderable via drag-and-drop

**Drag and drop:**
- Search pane -> Playlist pane: add tracks to current playlist
- Within playlist pane: reorder tracks

**Right-click context menus:**
- Search pane track: Properties, Tag, Remove from library
- Playlist pane track: Properties, Tag, Remove from playlist
- Multi-select: same menus, batch operation

**Properties dialog:** Modal with all metadata fields, file path, file size, format, larger energy sparkline.

**Playback bar (bottom):** Reserved space, implemented in a future milestone.

## Out of Scope (YAGNI)

- No user accounts or cloud sync (local-only)
- No album art display
- No audio playback (future milestone)
- No auto-watch folders
- No undo/redo system
