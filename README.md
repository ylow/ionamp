# ionamp

A local-first music playlist management app. Indexes audio files on disk, computes per-track energy profiles, and provides fast search, filtering, grouping, tagging, and playlist management in a dense two-pane UI.

ionamp does not store or play music files — it indexes them, extracts metadata, and organizes them.

## Features

- **Fast search** — Full-text search across title, artist, album, genre via SQLite FTS5
- **Filtering** — Filter by artist, album, genre, year, or user-defined tags
- **Grouping** — Group results by artist, album, genre, year, or energy clusters
- **Energy profiling** — Computes a 128-point energy vector per track (RMS + spectral centroid + onset strength), displayed as inline sparklines
- **Energy clustering** — K-means++ groups tracks by energy similarity
- **Tagging** — Create custom tag categories and values, apply to tracks
- **Playlists** — Create playlists, drag tracks from search, reorder via drag-and-drop, multi-select batch operations
- **Two-pane UI** — Resizable split layout: search/browse on the left, playlist on the right
- **Import** — Recursive folder scanning with progress, automatic deduplication by file path

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust (Tauri v2) |
| Frontend | Svelte 5 (SvelteKit), Tailwind CSS v4 |
| Database | SQLite via rusqlite (bundled, zero setup) |
| Metadata | lofty (pure Rust) |
| Audio decoding | symphonia (pure Rust) |
| FFT | realfft |
| Desktop shell | Tauri v2 (WKWebView on macOS) |

No C dependencies. Single self-contained binary.

## Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (stable)
- macOS, Windows, or Linux

On macOS, Xcode Command Line Tools are required:
```
xcode-select --install
```

## Building

### Development

```bash
npm install
npm run tauri dev
```

The first build compiles all Rust dependencies (~2 minutes). Subsequent builds are incremental.

### Release

```bash
npm run tauri build
```

Produces:
- Binary: `src-tauri/target/release/ionamp`
- macOS installer: `src-tauri/target/release/bundle/dmg/`

### Running tests

```bash
cd src-tauri && cargo test
```

## Usage

1. **Import music** — Click "Import" in the toolbar, select a folder. ionamp recursively scans for audio files (mp3, flac, ogg, wav, aiff, aac, m4a, opus), extracts metadata, decodes audio, and computes energy profiles.

2. **Search** — Type in the search bar for instant full-text search. Use the filter dropdowns to narrow by artist, album, genre, or year.

3. **Group** — Select "Group by" to organize results into collapsible sections. "Energy" groups tracks by K-means clustering of their energy profiles.

4. **Create playlists** — Type a name in the playlist pane and click "+". Drag tracks from the search pane to add them.

5. **Reorder** — Drag entries within the playlist to reorder. Multi-select with Cmd+click or Shift+click, then drag to move as a group.

6. **Delete** — Select entries and press Backspace, or right-click and choose "Remove from playlist".

7. **Tag** — Right-click tracks and choose "Tag..." to create and apply custom tags.

8. **Properties** — Right-click a track and choose "Properties" to see full metadata and a large energy sparkline.

## Data Storage

The SQLite database is stored at:
- macOS: `~/Library/Application Support/ionamp/ionamp.db`
- Linux: `~/.local/share/ionamp/ionamp.db`
- Windows: `%APPDATA%/ionamp/ionamp.db`

## License

MIT
