<script lang="ts">
  import type { TrackGroup, Track } from "$lib/types";
  import TrackRow from "./TrackRow.svelte";
  import VirtualList from "./VirtualList.svelte";

  let {
    groups,
    maxEnergy = 0,
    oncontextmenu,
    ondblclick,
  }: {
    groups: TrackGroup[];
    maxEnergy?: number;
    oncontextmenu?: (e: MouseEvent, track: Track) => void;
    ondblclick?: (track: Track) => void;
  } = $props();

  let collapsedGroups = $state<Set<string>>(new Set());

  function toggleGroup(key: string) {
    const next = new Set(collapsedGroups);
    if (next.has(key)) {
      next.delete(key);
    } else {
      next.add(key);
    }
    collapsedGroups = next;
  }

  // For ungrouped (single group with empty key), render flat
  let isGrouped = $derived(
    groups.length > 1 || (groups.length === 1 && groups[0].key !== ""),
  );

  let flatTracks = $derived(groups.flatMap((g) => g.tracks));
  let allTrackIds = $derived(flatTracks.map((t) => t.id));
</script>

{#if isGrouped}
  <div class="overflow-y-auto h-full">
    {#each groups as group}
      <div>
        <button
          onclick={() => toggleGroup(group.key)}
          class="flex items-center gap-2 w-full px-2 py-1 text-xs font-semibold text-neutral-300 bg-neutral-800 hover:bg-neutral-750 sticky top-0 z-10"
        >
          <span class="text-neutral-500">
            {collapsedGroups.has(group.key) ? "▶" : "▼"}
          </span>
          <span>{group.key}</span>
          <span class="text-neutral-500">({group.tracks.length})</span>
        </button>
        {#if !collapsedGroups.has(group.key)}
          {#each group.tracks as track}
            <TrackRow {track} {allTrackIds} {maxEnergy} {oncontextmenu} {ondblclick} />
          {/each}
        {/if}
      </div>
    {/each}
  </div>
{:else}
  <VirtualList items={flatTracks} itemHeight={24} keyFn={(item) => item.id}>
    {#snippet row(item: Track)}
      <TrackRow track={item} {allTrackIds} {maxEnergy} {oncontextmenu} {ondblclick} />
    {/snippet}
  </VirtualList>
{/if}
