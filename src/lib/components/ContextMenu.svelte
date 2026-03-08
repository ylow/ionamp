<script lang="ts">
  import type { PlaylistEntry } from "$lib/types";
  import { selectionState } from "$lib/stores/selection.svelte";
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

  let showTagDialog = $state(false);
  let showProperties = $state(false);

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
    if (!entry) return;
    await playlistState.removeEntries([entry.id]);
    onclose();
  }

  function handleProperties() {
    showProperties = true;
    onclose();
    // Properties dialog is opened via a dispatched event — handled by parent
    const event = new CustomEvent("show-properties", {
      detail: { trackId: selectionState.ids[0] ?? entry?.track.id },
      bubbles: true,
    });
    document.dispatchEvent(event);
  }

  function handleTag() {
    const event = new CustomEvent("show-tag-dialog", {
      detail: { trackIds: selectionState.ids },
      bubbles: true,
    });
    document.dispatchEvent(event);
    onclose();
  }

  // Position the menu to stay within viewport
  let menuStyle = $derived.by(() => {
    const maxX = typeof window !== "undefined" ? window.innerWidth - 180 : x;
    const maxY = typeof window !== "undefined" ? window.innerHeight - 120 : y;
    return `left: ${Math.min(x, maxX)}px; top: ${Math.min(y, maxY)}px;`;
  });
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
      {#if selectionState.count > 1}({selectionState.count}){/if}
    </button>
  {:else if context === "playlist"}
    <button
      class="w-full text-left px-3 py-1 text-xs text-red-400 hover:bg-neutral-700"
      onclick={handleRemoveFromPlaylist}
    >
      Remove from playlist
    </button>
  {/if}
</div>
