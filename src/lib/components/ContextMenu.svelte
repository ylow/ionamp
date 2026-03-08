<script lang="ts">
  import type { PlaylistEntry } from "$lib/types";
  import { selectionState, playlistSelectionState } from "$lib/stores/selection.svelte";
  import { playlistState } from "$lib/stores/playlist.svelte";
  import { searchState } from "$lib/stores/search.svelte";
  import { deleteTracks } from "$lib/api";

  let {
    x,
    y,
    context,
    entry,
    onclose,
  }: {
    x: number;
    y: number;
    context: "search" | "playlist";
    entry?: PlaylistEntry;
    onclose: () => void;
  } = $props();

  async function handleRemoveFromLibrary() {
    const ids = selectionState.ids;
    if (ids.length === 0) return;
    await deleteTracks(ids);
    selectionState.clear();
    searchState.search();
    searchState.loadFilterOptions();
    onclose();
  }

  async function handleRemoveFromPlaylist() {
    const ids = playlistSelectionState.ids;
    if (ids.length === 0) return;
    await playlistState.removeEntries(ids);
    playlistSelectionState.clear();
    onclose();
  }

  function handleProperties() {
    onclose();
    let trackId: number | undefined;
    if (context === "search") {
      trackId = selectionState.ids[0];
    } else {
      trackId = entry?.track.id;
    }
    if (trackId) {
      document.dispatchEvent(
        new CustomEvent("show-properties", { detail: { trackId }, bubbles: true }),
      );
    }
  }

  function handleTag() {
    let trackIds: number[];
    if (context === "search") {
      trackIds = selectionState.ids;
    } else {
      // For playlist context, get track IDs from selected entries
      const selectedEntryIds = playlistSelectionState.ids;
      const selectedEntries = (playlistState.entries ?? []).filter((e) =>
        selectedEntryIds.includes(e.id),
      );
      trackIds = selectedEntries.map((e) => e.track_id);
    }
    document.dispatchEvent(
      new CustomEvent("show-tag-dialog", { detail: { trackIds }, bubbles: true }),
    );
    onclose();
  }

  let menuStyle = $derived.by(() => {
    const maxX = typeof window !== "undefined" ? window.innerWidth - 180 : x;
    const maxY = typeof window !== "undefined" ? window.innerHeight - 120 : y;
    return `left: ${Math.min(x, maxX)}px; top: ${Math.min(y, maxY)}px;`;
  });

  let selectionCount = $derived(
    context === "search" ? selectionState.count : playlistSelectionState.count,
  );
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed z-50 bg-neutral-800 border border-neutral-600 rounded shadow-lg py-1 min-w-[160px]"
  style={menuStyle}
  onclick={(e) => e.stopPropagation()}
>
  <button
    class="w-full text-left px-3 py-1 text-xs text-neutral-200 hover:bg-neutral-700"
    onclick={handleProperties}
  >
    Properties
  </button>
  <button
    class="w-full text-left px-3 py-1 text-xs text-neutral-200 hover:bg-neutral-700"
    onclick={handleTag}
  >
    Tag...
  </button>
  <div class="border-t border-neutral-700 my-1"></div>
  {#if context === "search"}
    <button
      class="w-full text-left px-3 py-1 text-xs text-red-400 hover:bg-neutral-700"
      onclick={handleRemoveFromLibrary}
    >
      Remove from library
      {#if selectionCount > 1}({selectionCount}){/if}
    </button>
  {:else if context === "playlist"}
    <button
      class="w-full text-left px-3 py-1 text-xs text-red-400 hover:bg-neutral-700"
      onclick={handleRemoveFromPlaylist}
    >
      Remove from playlist
      {#if selectionCount > 1}({selectionCount}){/if}
    </button>
  {/if}
</div>
