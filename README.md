# ionamp

A playlist builder for DJs and music curators who organize by *feel*, not just metadata.

ionamp analyzes your music library's audio — loudness, brightness, rhythmic intensity — and lets you search, group, and build playlists using the actual sonic character of each track. Every track gets an inline energy sparkline so you can see its dynamic shape at a glance. Group by "Energy" to have K-means clustering sort your library into sonic neighborhoods. Then drag tracks into playlists, reorder them, and play them back — all from a fast, dense, keyboard-driven two-pane UI.

Local-first. No cloud. No accounts. Your files stay on disk; ionamp just indexes them.

## Features

- **Energy analysis** — Each track is decoded and analyzed for RMS (loudness), spectral centroid (brightness), and onset strength (rhythmic intensity), producing a 128-point profile displayed as an inline sparkline
- **Energy clustering** — Group your library by sonic similarity using K-means++, not just artist/album/genre
- **Instant search** — Full-text search across title, artist, album, genre via SQLite FTS5
- **Filtering and grouping** — Filter by artist, album, genre, year, or custom tags; group results into collapsible sections
- **Custom tagging** — Create your own tag categories and values (mood, energy level, set position — whatever you need)
- **Playlist building** — Drag tracks from search into playlists, multi-select reorder, batch delete
- **Audio playback** — Double-click to play; playlist auto-advance with loop modes (single track, full playlist)
- **Two-pane UI** — Resizable split layout: search/browse left, playlist right — everything on one screen
- **Fast import** — Recursive folder scanning with progress, automatic deduplication

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
