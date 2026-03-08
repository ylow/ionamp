<script lang="ts">
  import type { Track } from "$lib/types";
  import { selectionState } from "$lib/stores/selection.svelte";
  import Sparkline from "./Sparkline.svelte";

  let {
    track,
    allTrackIds = [],
    maxEnergy = 0,
    oncontextmenu,
  }: {
    track: Track;
    allTrackIds?: number[];
    maxEnergy?: number;
    oncontextmenu?: (e: MouseEvent, track: Track) => void;
  } = $props();

  let selected = $derived(selectionState.isSelected(track.id));

  function formatDuration(secs: number | null): string {
    if (secs === null) return "--:--";
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  function handleClick(e: MouseEvent) {
    if (e.shiftKey) {
      selectionState.rangeSelect(track.id, allTrackIds);
    } else if (e.metaKey || e.ctrlKey) {
      selectionState.toggle(track.id);
    } else {
      selectionState.select(track.id);
    }
  }

  function handleContextMenu(e: MouseEvent) {
    if (!selectionState.isSelected(track.id)) {
      selectionState.select(track.id);
    }
    oncontextmenu?.(e, track);
  }

  function handleDragStart(e: DragEvent) {
    if (!selectionState.isSelected(track.id)) {
      selectionState.select(track.id);
    }
    const ids = selectionState.ids;
    e.dataTransfer?.setData("application/json", JSON.stringify(ids));
    e.dataTransfer!.effectAllowed = "copy";
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="flex items-center gap-2 px-2 py-0.5 text-xs cursor-default hover:bg-neutral-800 select-none"
  class:bg-blue-900={selected}
  class:hover:bg-blue-800={selected}
  onclick={handleClick}
  oncontextmenu={handleContextMenu}
  draggable="true"
  ondragstart={handleDragStart}
  role="row"
  tabindex="-1"
>
  <span class="flex-1 truncate text-neutral-100">
    {track.title ?? track.file_path.split("/").pop()}
  </span>
  <span class="w-28 truncate text-neutral-400">{track.artist ?? ""}</span>
  <span class="w-28 truncate text-neutral-400">{track.album ?? ""}</span>
  <span class="w-12 text-right text-neutral-500">
    {formatDuration(track.duration_secs)}
  </span>
  <Sparkline energyVector={track.energy_vector} maxValue={maxEnergy} width={80} height={16} />
</div>
