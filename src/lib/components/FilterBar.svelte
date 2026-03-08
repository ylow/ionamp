<script lang="ts">
  import { searchState } from "$lib/stores/search.svelte";
  import type { GroupByField } from "$lib/types";

  function addArtistFilter(e: Event) {
    const value = (e.target as HTMLSelectElement).value;
    if (value) {
      searchState.setFilter({ Artist: value });
      (e.target as HTMLSelectElement).value = "";
    }
  }

  function addAlbumFilter(e: Event) {
    const value = (e.target as HTMLSelectElement).value;
    if (value) {
      searchState.setFilter({ Album: value });
      (e.target as HTMLSelectElement).value = "";
    }
  }

  function addGenreFilter(e: Event) {
    const value = (e.target as HTMLSelectElement).value;
    if (value) {
      searchState.setFilter({ Genre: value });
      (e.target as HTMLSelectElement).value = "";
    }
  }

  function addYearFilter(e: Event) {
    const value = (e.target as HTMLSelectElement).value;
    if (value) {
      searchState.setFilter({ Year: parseInt(value) });
      (e.target as HTMLSelectElement).value = "";
    }
  }

  function setGroupBy(e: Event) {
    const value = (e.target as HTMLSelectElement).value;
    searchState.setGroupBy(
      value ? (value as GroupByField | "Energy") : null,
    );
  }
</script>

<div class="flex items-center gap-1 flex-wrap">
  <select
    onchange={addArtistFilter}
    class="px-1 py-0.5 text-xs bg-neutral-800 border border-neutral-600 rounded text-neutral-300"
  >
    <option value="">Artist</option>
    {#each searchState.filterOptions.artists as artist}
      <option value={artist}>{artist}</option>
    {/each}
  </select>

  <select
    onchange={addAlbumFilter}
    class="px-1 py-0.5 text-xs bg-neutral-800 border border-neutral-600 rounded text-neutral-300"
  >
    <option value="">Album</option>
    {#each searchState.filterOptions.albums as album}
      <option value={album}>{album}</option>
    {/each}
  </select>

  <select
    onchange={addGenreFilter}
    class="px-1 py-0.5 text-xs bg-neutral-800 border border-neutral-600 rounded text-neutral-300"
  >
    <option value="">Genre</option>
    {#each searchState.filterOptions.genres as genre}
      <option value={genre}>{genre}</option>
    {/each}
  </select>

  <select
    onchange={addYearFilter}
    class="px-1 py-0.5 text-xs bg-neutral-800 border border-neutral-600 rounded text-neutral-300"
  >
    <option value="">Year</option>
    {#each searchState.filterOptions.years as year}
      <option value={year}>{year}</option>
    {/each}
  </select>

  <div class="w-px h-4 bg-neutral-600 mx-1"></div>

  <select
    onchange={setGroupBy}
    class="px-1 py-0.5 text-xs bg-neutral-800 border border-neutral-600 rounded text-neutral-300"
  >
    <option value="">Group by</option>
    <option value="Artist">Artist</option>
    <option value="Album">Album</option>
    <option value="Genre">Genre</option>
    <option value="Year">Year</option>
    <option value="Energy">Energy</option>
  </select>

  {#if searchState.groupBy === "Energy"}
    <input
      type="number"
      value={searchState.clusterSeed}
      onchange={(e) => {
        searchState.clusterSeed = parseInt((e.target as HTMLInputElement).value) || 0;
        searchState.search();
      }}
      class="w-14 px-1 py-0.5 text-xs bg-neutral-800 border border-neutral-600 rounded text-neutral-300 text-center"
      title="Cluster seed"
    />
  {/if}

  {#if searchState.filters.length > 0}
    <button
      onclick={() => searchState.clearFilters()}
      class="px-1 py-0.5 text-xs bg-red-900 hover:bg-red-800 rounded text-neutral-300"
    >
      Clear filters
    </button>
  {/if}
</div>

{#if searchState.filters.length > 0}
  <div class="flex gap-1 flex-wrap mt-1">
    {#each searchState.filters as filter, i}
      <button
        onclick={() => searchState.removeFilter(i)}
        class="px-1.5 py-0.5 text-xs bg-blue-900 rounded text-blue-200 hover:bg-blue-800"
      >
        {#if "Artist" in filter}Artist: {filter.Artist}
        {:else if "Album" in filter}Album: {filter.Album}
        {:else if "Genre" in filter}Genre: {filter.Genre}
        {:else if "Year" in filter}Year: {filter.Year}
        {:else if "Tag" in filter}Tag
        {/if}
        ×
      </button>
    {/each}
  </div>
{/if}
