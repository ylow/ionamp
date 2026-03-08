<script lang="ts">
  import { searchState } from "$lib/stores/search.svelte";
  import { playbackState } from "$lib/stores/playback.svelte";
  import SearchInput from "./SearchInput.svelte";
  import FilterBar from "./FilterBar.svelte";
  import TrackList from "./TrackList.svelte";
  import type { Track } from "$lib/types";
  import ContextMenu from "./ContextMenu.svelte";

  function handleDoubleClick(track: Track) {
    playbackState.playSingle(track);
  }

  let contextMenu = $state<{
    x: number;
    y: number;
    track: Track;
  } | null>(null);

  function handleContextMenu(e: MouseEvent, track: Track) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, track };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  $effect(() => {
    searchState.search();
    searchState.loadFilterOptions();
  });
</script>

<svelte:window onclick={closeContextMenu} />

<div class="h-full flex flex-col">
  <div class="px-2 py-1.5 space-y-1 border-b border-neutral-700">
    <SearchInput />
    <FilterBar />
  </div>
  <div class="flex-1 overflow-hidden relative">
    {#if searchState.loading}
      <div
        class="absolute inset-0 flex items-center justify-center text-neutral-500 text-sm"
      >
        Loading...
      </div>
    {:else if searchState.results.length === 0}
      <div
        class="flex items-center justify-center h-full text-neutral-500 text-sm"
      >
        No tracks found. Import some music to get started.
      </div>
    {:else}
      <TrackList
        groups={searchState.results}
        maxEnergy={searchState.maxEnergy}
        oncontextmenu={handleContextMenu}
        ondblclick={handleDoubleClick}
      />
    {/if}
  </div>
  <div
    class="px-2 py-0.5 text-xs text-neutral-500 border-t border-neutral-700"
  >
    {searchState.totalCount} tracks
  </div>
</div>

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    context="search"
    onclose={closeContextMenu}
  />
{/if}
