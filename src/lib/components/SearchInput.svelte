<script lang="ts">
  import { searchState } from "$lib/stores/search.svelte";

  function onInput(e: Event) {
    searchState.text = (e.target as HTMLInputElement).value;
    searchState.debouncedSearch();
  }
</script>

<input
  type="text"
  placeholder="Search tracks..."
  value={searchState.text}
  oninput={onInput}
  class="flex-1 px-2 py-1 text-sm bg-neutral-800 border border-neutral-600 rounded text-neutral-100 placeholder-neutral-500 focus:outline-none focus:border-blue-500"
/>
