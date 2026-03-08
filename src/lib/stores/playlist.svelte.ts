import {
  listPlaylists,
  createPlaylist,
  deletePlaylist,
  renamePlaylist,
  getPlaylistTracks,
  addToPlaylist,
  removeFromPlaylist,
  reorderPlaylist,
} from "$lib/api";
import type { Playlist, PlaylistEntry } from "$lib/types";

class PlaylistState {
  playlists = $state<Playlist[]>([]);
  selectedPlaylistId = $state<number | null>(null);
  entries = $state<PlaylistEntry[]>([]);
  loading = $state(false);

  get maxEnergy(): number {
    let max = 0;
    for (const entry of this.entries) {
      if (entry.track.energy_vector) {
        for (const v of entry.track.energy_vector) {
          if (v > max) max = v;
        }
      }
    }
    return max;
  }

  get selectedPlaylist(): Playlist | null {
    return (
      this.playlists.find((p) => p.id === this.selectedPlaylistId) ?? null
    );
  }

  async loadPlaylists() {
    try {
      this.playlists = await listPlaylists();
    } catch (e) {
      console.error("Failed to load playlists:", e);
    }
  }

  async selectPlaylist(id: number | null) {
    this.selectedPlaylistId = id;
    if (id !== null) {
      await this.loadEntries();
    } else {
      this.entries = [];
    }
  }

  async loadEntries() {
    if (this.selectedPlaylistId === null) return;
    this.loading = true;
    try {
      this.entries = await getPlaylistTracks(this.selectedPlaylistId);
    } catch (e) {
      console.error("Failed to load playlist tracks:", e);
    } finally {
      this.loading = false;
    }
  }

  async create(name: string) {
    try {
      const id = await createPlaylist(name);
      await this.loadPlaylists();
      await this.selectPlaylist(id);
    } catch (e) {
      console.error("Failed to create playlist:", e);
    }
  }

  async remove(id: number) {
    try {
      await deletePlaylist(id);
      if (this.selectedPlaylistId === id) {
        this.selectedPlaylistId = null;
        this.entries = [];
      }
      await this.loadPlaylists();
    } catch (e) {
      console.error("Failed to delete playlist:", e);
    }
  }

  async rename(id: number, newName: string) {
    try {
      await renamePlaylist(id, newName);
      await this.loadPlaylists();
    } catch (e) {
      console.error("Failed to rename playlist:", e);
    }
  }

  async addTracks(trackIds: number[]) {
    if (this.selectedPlaylistId === null) return;
    try {
      await addToPlaylist(this.selectedPlaylistId, trackIds);
      await this.loadEntries();
    } catch (e) {
      console.error("Failed to add tracks:", e);
    }
  }

  async removeEntries(entryIds: number[]) {
    try {
      await removeFromPlaylist(entryIds);
      await this.loadEntries();
    } catch (e) {
      console.error("Failed to remove entries:", e);
    }
  }

  async reorder(entryIds: number[]) {
    if (this.selectedPlaylistId === null) return;
    try {
      await reorderPlaylist(this.selectedPlaylistId, entryIds);
      await this.loadEntries();
    } catch (e) {
      console.error("Failed to reorder:", e);
    }
  }
}

export const playlistState = new PlaylistState();
