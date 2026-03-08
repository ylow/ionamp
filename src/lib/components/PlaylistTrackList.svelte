<script lang="ts">
  import type { PlaylistEntry, Track } from "$lib/types";
  import { playlistState } from "$lib/stores/playlist.svelte";
  import Sparkline from "./Sparkline.svelte";

  let {
    entries,
    maxEnergy = 0,
    oncontextmenu,
  }: {
    entries: PlaylistEntry[];
    maxEnergy?: number;
    oncontextmenu?: (e: MouseEvent, entry: PlaylistEntry) => void;
  } = $props();

  let dragOverIndex = $state<number | null>(null);

  function formatDuration(secs: number | null): string {
    if (secs === null) return "--:--";
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    // Intra-list reorder uses "move", cross-pane from search uses "copy"
    const isReorder = e.dataTransfer?.types.includes("text/x-entry-id");
    e.dataTransfer!.dropEffect = isReorder ? "move" : "copy";
    dragOverIndex = index;
  }

  function handleDragLeave() {
    dragOverIndex = null;
  }

  function handleDrop(e: DragEvent, dropIndex: number) {
    e.preventDefault();
    dragOverIndex = null;

    const data = e.dataTransfer?.getData("text/x-entry-id");
    if (data) {
      // Intra-list reorder
      const dragEntryId = parseInt(data);
      const currentOrder = entries.map((e) => e.id);
      const fromIndex = currentOrder.indexOf(dragEntryId);
      if (fromIndex === -1) return;

      const newOrder = [...currentOrder];
      newOrder.splice(fromIndex, 1);
      newOrder.splice(dropIndex, 0, dragEntryId);
      playlistState.reorder(newOrder);
      return;
    }

    // Cross-pane drop (track IDs from search)
    const jsonData = e.dataTransfer?.getData("application/json");
    if (jsonData) {
      try {
        const trackIds: number[] = JSON.parse(jsonData);
        playlistState.addTracks(trackIds);
      } catch {
        // ignore
      }
    }
  }

  function handleEntryDragStart(e: DragEvent, entry: PlaylistEntry) {
    e.dataTransfer?.setData("text/x-entry-id", entry.id.toString());
    e.dataTransfer!.effectAllowed = "move";
  }

  function handleContextMenu(e: MouseEvent, entry: PlaylistEntry) {
    e.preventDefault();
    oncontextmenu?.(e, entry);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="overflow-y-auto h-full"
  ondragover={(e) => { e.preventDefault(); e.dataTransfer!.dropEffect = "copy"; }}
  ondrop={(e) => handleDrop(e, entries.length)}
>
  {#each entries as entry, i}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="flex items-center gap-2 px-2 py-0.5 text-xs cursor-default hover:bg-neutral-800 select-none"
      class:border-t-2={dragOverIndex === i}
      class:border-blue-500={dragOverIndex === i}
      draggable="true"
      ondragstart={(e) => handleEntryDragStart(e, entry)}
      ondragover={(e) => handleDragOver(e, i)}
      ondragleave={handleDragLeave}
      ondrop={(e) => handleDrop(e, i)}
      oncontextmenu={(e) => handleContextMenu(e, entry)}
      role="row"
      tabindex="-1"
    >
      <span class="w-5 text-right text-neutral-600">{entry.position}</span>
      <span class="flex-1 truncate text-neutral-100">
        {entry.track.title ?? entry.track.file_path.split("/").pop()}
      </span>
      <span class="w-20 truncate text-neutral-400">{entry.track.artist ?? ""}</span>
      <span class="w-10 text-right text-neutral-500">
        {formatDuration(entry.track.duration_secs)}
      </span>
      <Sparkline energyVector={entry.track.energy_vector} maxValue={maxEnergy} width={60} height={14} />
    </div>
  {/each}

  <!-- Drop zone at the end -->
  {#if entries.length > 0}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="h-8"
      ondragover={(e) => handleDragOver(e, entries.length)}
      ondragleave={handleDragLeave}
      ondrop={(e) => handleDrop(e, entries.length)}
    ></div>
  {/if}
</div>
