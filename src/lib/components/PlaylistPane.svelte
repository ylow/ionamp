<script lang="ts">
  import { playlistState } from "$lib/stores/playlist.svelte";
  import PlaylistSelector from "./PlaylistSelector.svelte";
  import PlaylistTrackList from "./PlaylistTrackList.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import type { PlaylistEntry } from "$lib/types";

  let contextMenu = $state<{
    x: number;
    y: number;
    entry: PlaylistEntry;
  } | null>(null);

  function handleContextMenu(e: MouseEvent, entry: PlaylistEntry) {
    contextMenu = { x: e.clientX, y: e.clientY, entry };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    const types = Array.from(e.dataTransfer?.types ?? []);
    const data = e.dataTransfer?.getData("application/json");
    console.log("[PlaylistPane] drop, types:", types, "data:", data, "selectedPlaylistId:", playlistState.selectedPlaylistId);
    if (data && playlistState.selectedPlaylistId !== null) {
      try {
        const trackIds: number[] = JSON.parse(data);
        console.log("[PlaylistPane] adding tracks:", trackIds);
        playlistState.addTracks(trackIds);
      } catch (err) {
        console.error("[PlaylistPane] drop parse error:", err);
      }
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    e.dataTransfer!.dropEffect = "copy";
  }
</script>

<svelte:window onclick={closeContextMenu} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="h-full flex flex-col bg-neutral-850"
  ondrop={handleDrop}
  ondragover={handleDragOver}
>
  <div class="px-2 py-1.5 border-b border-neutral-700">
    <PlaylistSelector />
  </div>
  <div class="flex-1 overflow-hidden">
    {#if playlistState.selectedPlaylistId === null}
      <div
        class="flex items-center justify-center h-full text-neutral-500 text-xs"
      >
        Select or create a playlist
      </div>
    {:else if playlistState.entries.length === 0}
      <div
        class="flex items-center justify-center h-full text-neutral-500 text-xs"
      >
        Drag tracks here to add them
      </div>
    {:else}
      <PlaylistTrackList
        entries={playlistState.entries}
        maxEnergy={playlistState.maxEnergy}
        oncontextmenu={handleContextMenu}
      />
    {/if}
  </div>
  <div
    class="px-2 py-0.5 text-xs text-neutral-500 border-t border-neutral-700"
  >
    {playlistState.entries.length} tracks
  </div>
</div>

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    context="playlist"
    entry={contextMenu.entry}
    onclose={closeContextMenu}
  />
{/if}
