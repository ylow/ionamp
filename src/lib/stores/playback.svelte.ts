import {
  playFile,
  pausePlayback,
  resumePlayback,
  stopPlayback,
  seekPlayback,
  getPlaybackStatus,
  type PlaybackStatus,
} from "$lib/api";
import type { Track, PlaylistEntry } from "$lib/types";

export type LoopMode = "off" | "single" | "playlist";

class PlaybackState {
  playing = $state(false);
  paused = $state(false);
  positionSecs = $state(0);
  durationSecs = $state(0);
  currentPath = $state<string | null>(null);
  currentTitle = $state<string | null>(null);
  currentArtist = $state<string | null>(null);
  ended = $state(false);

  loopMode = $state<LoopMode>("off");

  // Queue for playlist playback
  queue = $state<Track[]>([]);
  queueIndex = $state(-1);

  private pollTimer: ReturnType<typeof setInterval> | null = null;

  get hasTrack(): boolean {
    return this.currentPath !== null;
  }

  get progress(): number {
    return this.durationSecs > 0 ? this.positionSecs / this.durationSecs : 0;
  }

  startPolling() {
    if (this.pollTimer) return;
    this.pollTimer = setInterval(() => this.poll(), 200);
  }

  stopPolling() {
    if (this.pollTimer) {
      clearInterval(this.pollTimer);
      this.pollTimer = null;
    }
  }

  private async poll() {
    try {
      const status: PlaybackStatus = await getPlaybackStatus();
      this.playing = status.playing;
      this.paused = status.paused;
      this.positionSecs = status.position_secs;
      this.durationSecs = status.duration_secs;
      this.currentPath = status.current_path;
      this.currentTitle = status.current_title;
      this.currentArtist = status.current_artist;

      if (status.ended && !this.ended) {
        this.ended = true;
        this.handleTrackEnded();
      } else if (!status.ended) {
        this.ended = false;
      }
    } catch {
      // ignore polling errors
    }
  }

  private handleTrackEnded() {
    if (this.loopMode === "single") {
      // Replay the same track
      if (this.currentPath) {
        playFile(
          this.currentPath,
          this.currentTitle,
          this.currentArtist,
          this.durationSecs,
        );
      }
    } else if (this.queue.length > 0) {
      const nextIndex = this.queueIndex + 1;
      if (nextIndex < this.queue.length) {
        this.playFromQueue(nextIndex);
      } else if (this.loopMode === "playlist") {
        this.playFromQueue(0);
      }
    }
  }

  /** Play a single track (e.g. double-click in search). Clears the queue. */
  async playSingle(track: Track) {
    this.queue = [];
    this.queueIndex = -1;
    await this.playTrack(track);
  }

  /** Play from a playlist queue starting at a given index. */
  async playFromPlaylist(tracks: Track[], startIndex: number) {
    this.queue = tracks;
    this.queueIndex = startIndex;
    await this.playTrack(tracks[startIndex]);
  }

  private async playFromQueue(index: number) {
    if (index < 0 || index >= this.queue.length) return;
    this.queueIndex = index;
    await this.playTrack(this.queue[index]);
  }

  private async playTrack(track: Track) {
    try {
      await playFile(
        track.file_path,
        track.title,
        track.artist,
        track.duration_secs ?? 0,
      );
      this.ended = false;
      this.startPolling();
    } catch (e) {
      console.error("Playback error:", e);
    }
  }

  async togglePlayPause() {
    if (this.playing) {
      await pausePlayback();
    } else if (this.paused) {
      await resumePlayback();
    }
  }

  async stop() {
    await stopPlayback();
    this.queue = [];
    this.queueIndex = -1;
    this.stopPolling();
  }

  async seek(fraction: number) {
    const pos = fraction * this.durationSecs;
    await seekPlayback(pos);
  }

  cycleLoopMode() {
    if (this.loopMode === "off") {
      this.loopMode = "single";
    } else if (this.loopMode === "single") {
      this.loopMode = "playlist";
    } else {
      this.loopMode = "off";
    }
  }

  async skipNext() {
    if (this.queue.length === 0) return;
    const nextIndex = this.queueIndex + 1;
    if (nextIndex < this.queue.length) {
      await this.playFromQueue(nextIndex);
    } else if (this.loopMode === "playlist") {
      await this.playFromQueue(0);
    }
  }

  async skipPrev() {
    // If more than 3 seconds in, restart current track
    if (this.positionSecs > 3) {
      await seekPlayback(0);
      return;
    }
    if (this.queue.length === 0) return;
    const prevIndex = this.queueIndex - 1;
    if (prevIndex >= 0) {
      await this.playFromQueue(prevIndex);
    } else if (this.loopMode === "playlist") {
      await this.playFromQueue(this.queue.length - 1);
    }
  }
}

export const playbackState = new PlaybackState();
