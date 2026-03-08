<script lang="ts">
  import type { PlaylistEntry } from "$lib/types";
  import { playlistState } from "$lib/stores/playlist.svelte";
  import { playlistSelectionState } from "$lib/stores/selection.svelte";
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
  let allEntryIds = $derived(entries.map((e) => e.id));

  function formatDuration(secs: number | null): string {
    if (secs === null) return "--:--";
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  function handleClick(e: MouseEvent, entry: PlaylistEntry) {
    if (e.shiftKey) {
      playlistSelectionState.rangeSelect(entry.id, allEntryIds);
    } else if (e.metaKey || e.ctrlKey) {
      playlistSelectionState.toggle(entry.id);
    } else {
      playlistSelectionState.select(entry.id);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Backspace" || e.key === "Delete") {
      const ids = playlistSelectionState.ids;
      if (ids.length > 0) {
        e.preventDefault();
        playlistState.removeEntries(ids);
        playlistSelectionState.clear();
      }
    }
  }

  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    const isReorder = e.dataTransfer?.types.includes("text/x-entry-id");
    e.dataTransfer!.dropEffect = isReorder ? "move" : "copy";
    dragOverIndex = index;
  }

  function handleDragLeave() {
    dragOverIndex = null;
  }

  function handleDrop(e: DragEvent, dropIndex: number) {
    e.preventDefault();
    e.stopPropagation();
    dragOverIndex = null;

    const data = e.dataTransfer?.getData("text/x-entry-id");
    if (data) {
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
    if (!playlistSelectionState.isSelected(entry.id)) {
      playlistSelectionState.select(entry.id);
    }
    oncontextmenu?.(e, entry);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="overflow-y-auto h-full outline-none"
  tabindex="0"
  onkeydown={handleKeyDown}
  ondragover={(e) => { e.preventDefault(); e.dataTransfer!.dropEffect = "copy"; }}
  ondrop={(e) => handleDrop(e, entries.length)}
>
  {#each entries as entry, i}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="flex items-center gap-2 px-2 py-0.5 text-xs cursor-default hover:bg-neutral-800 select-none"
      class:bg-blue-900={playlistSelectionState.isSelected(entry.id)}
      class:hover:bg-blue-800={playlistSelectionState.isSelected(entry.id)}
      class:border-t-2={dragOverIndex === i}
      class:border-blue-500={dragOverIndex === i}
      draggable="true"
      onclick={(e) => handleClick(e, entry)}
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
