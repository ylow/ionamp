<script lang="ts">
  import { getTrack, getTrackTags } from "$lib/api";
  import type { Track, TagValue } from "$lib/types";
  import Sparkline from "./Sparkline.svelte";

  let {
    trackId,
    onclose,
  }: {
    trackId: number;
    onclose: () => void;
  } = $props();

  let track = $state<Track | null>(null);
  let tags = $state<TagValue[]>([]);

  $effect(() => {
    loadTrack();
  });

  async function loadTrack() {
    track = await getTrack(trackId);
    tags = await getTrackTags(trackId);
  }

  function formatBytes(bytes: number | null): string {
    if (bytes === null) return "—";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDuration(secs: number | null): string {
    if (secs === null) return "—";
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center"
  onclick={onclose}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="bg-neutral-800 rounded-lg shadow-xl p-4 w-96 max-h-[80vh] overflow-y-auto"
    onclick={(e) => e.stopPropagation()}
  >
    {#if track}
      <h3 class="text-sm font-semibold text-neutral-100 mb-3">
        {track.title ?? "Unknown Title"}
      </h3>

      <div class="space-y-1.5 text-xs">
        <div class="flex">
          <span class="w-24 text-neutral-400">Artist</span>
          <span class="text-neutral-200">{track.artist ?? "—"}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-neutral-400">Album</span>
          <span class="text-neutral-200">{track.album ?? "—"}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-neutral-400">Album Artist</span>
          <span class="text-neutral-200">{track.album_artist ?? "—"}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-neutral-400">Genre</span>
          <span class="text-neutral-200">{track.genre ?? "—"}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-neutral-400">Year</span>
          <span class="text-neutral-200">{track.year ?? "—"}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-neutral-400">Track</span>
          <span class="text-neutral-200">{track.track_number ?? "—"}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-neutral-400">Disc</span>
          <span class="text-neutral-200">{track.disc_number ?? "—"}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-neutral-400">Duration</span>
          <span class="text-neutral-200"
            >{formatDuration(track.duration_secs)}</span
          >
        </div>

        <div class="border-t border-neutral-700 my-2"></div>

        <div class="flex">
          <span class="w-24 text-neutral-400">Format</span>
          <span class="text-neutral-200"
            >{track.format?.toUpperCase() ?? "—"}</span
          >
        </div>
        <div class="flex">
          <span class="w-24 text-neutral-400">Sample Rate</span>
          <span class="text-neutral-200">
            {track.sample_rate ? `${track.sample_rate} Hz` : "—"}
          </span>
        </div>
        <div class="flex">
          <span class="w-24 text-neutral-400">Bitrate</span>
          <span class="text-neutral-200">
            {track.bitrate ? `${track.bitrate} kbps` : "—"}
          </span>
        </div>
        <div class="flex">
          <span class="w-24 text-neutral-400">File Size</span>
          <span class="text-neutral-200">{formatBytes(track.file_size)}</span>
        </div>
        <div class="flex">
          <span class="w-24 text-neutral-400 flex-shrink-0">File Path</span>
          <span class="text-neutral-200 break-all">{track.file_path}</span>
        </div>

        {#if tags.length > 0}
          <div class="border-t border-neutral-700 my-2"></div>
          <div class="flex">
            <span class="w-24 text-neutral-400">Tags</span>
            <div class="flex flex-wrap gap-1">
              {#each tags as tag}
                <span
                  class="px-1.5 py-0.5 text-xs bg-blue-900 rounded text-blue-200"
                >
                  {tag.category_name}: {tag.value}
                </span>
              {/each}
            </div>
          </div>
        {/if}

        <div class="border-t border-neutral-700 my-2"></div>
        <div>
          <span class="text-neutral-400">Energy Profile</span>
          <div class="mt-1">
            <Sparkline
              energyVector={track.energy_vector}
              width={350}
              height={60}
            />
          </div>
        </div>
      </div>

      <div class="mt-3 flex justify-end">
        <button
          onclick={onclose}
          class="px-3 py-1 text-xs bg-neutral-700 hover:bg-neutral-600 rounded text-neutral-300"
        >
          Close
        </button>
      </div>
    {:else}
      <div class="text-neutral-500 text-sm">Loading...</div>
    {/if}
  </div>
</div>
