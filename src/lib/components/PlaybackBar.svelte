<script lang="ts">
  import { playbackState } from "$lib/stores/playback.svelte";

  function formatTime(secs: number): string {
    if (!secs || secs < 0) return "0:00";
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  function handleSeek(e: MouseEvent) {
    const bar = e.currentTarget as HTMLElement;
    const rect = bar.getBoundingClientRect();
    const fraction = (e.clientX - rect.left) / rect.width;
    playbackState.seek(Math.max(0, Math.min(1, fraction)));
  }

  function loopLabel(mode: string): string {
    if (mode === "single") return "1";
    if (mode === "playlist") return "All";
    return "Off";
  }
</script>

<div
  class="flex items-center gap-3 px-3 py-1.5 bg-neutral-800 border-t border-neutral-700 select-none"
>
  <!-- Track info -->
  <div class="w-48 truncate">
    {#if playbackState.hasTrack}
      <div class="text-xs text-neutral-100 truncate">
        {playbackState.currentTitle ?? "Unknown"}
      </div>
      <div class="text-[10px] text-neutral-400 truncate">
        {playbackState.currentArtist ?? ""}
      </div>
    {:else}
      <div class="text-xs text-neutral-500">No track playing</div>
    {/if}
  </div>

  <!-- Controls -->
  <div class="flex items-center gap-1.5">
    <button
      onclick={() => playbackState.skipPrev()}
      class="text-neutral-400 hover:text-neutral-100 text-sm px-1"
      title="Previous"
    >
      &#x23EE;
    </button>
    <button
      onclick={() => playbackState.togglePlayPause()}
      disabled={!playbackState.hasTrack}
      class="w-7 h-7 flex items-center justify-center rounded-full bg-neutral-600 hover:bg-neutral-500 text-neutral-100 disabled:opacity-30 text-sm"
      title={playbackState.playing ? "Pause" : "Play"}
    >
      {#if playbackState.playing}
        &#x23F8;
      {:else}
        &#x25B6;
      {/if}
    </button>
    <button
      onclick={() => playbackState.skipNext()}
      class="text-neutral-400 hover:text-neutral-100 text-sm px-1"
      title="Next"
    >
      &#x23ED;
    </button>
  </div>

  <!-- Time + progress -->
  <span class="text-[10px] text-neutral-500 w-8 text-right">
    {formatTime(playbackState.positionSecs)}
  </span>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="flex-1 h-1.5 bg-neutral-700 rounded cursor-pointer group"
    onclick={handleSeek}
  >
    <div
      class="h-full bg-blue-500 rounded transition-[width] duration-100 group-hover:bg-blue-400"
      style="width: {playbackState.progress * 100}%"
    ></div>
  </div>

  <span class="text-[10px] text-neutral-500 w-8">
    {formatTime(playbackState.durationSecs)}
  </span>

  <!-- Loop -->
  <button
    onclick={() => playbackState.cycleLoopMode()}
    class="text-[10px] px-1.5 py-0.5 rounded"
    class:text-neutral-500={playbackState.loopMode === "off"}
    class:bg-blue-900={playbackState.loopMode !== "off"}
    class:text-blue-300={playbackState.loopMode !== "off"}
    title="Loop: {playbackState.loopMode}"
  >
    &#x1F501; {loopLabel(playbackState.loopMode)}
  </button>
</div>
