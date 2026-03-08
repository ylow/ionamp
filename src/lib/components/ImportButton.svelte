<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { importState } from "$lib/stores/import.svelte";
  import { searchState } from "$lib/stores/search.svelte";

  async function handleImport() {
    const selected = await open({
      directory: true,
      multiple: true,
      title: "Select music folders to import",
    });

    if (!selected) return;

    const paths = Array.isArray(selected) ? selected : [selected];
    await importState.runImport(paths);
    searchState.search();
    searchState.loadFilterOptions();
  }
</script>

{#if importState.importing}
  <div class="flex items-center gap-2">
    <div class="w-24 h-1.5 bg-neutral-700 rounded overflow-hidden">
      <div
        class="h-full bg-blue-500 transition-all"
        style="width: {importState.progress * 100}%"
      ></div>
    </div>
    <span class="text-xs text-neutral-400">
      {importState.current}/{importState.total}
    </span>
  </div>
{:else}
  <button
    onclick={handleImport}
    class="px-3 py-1 text-xs bg-neutral-700 hover:bg-neutral-600 text-neutral-200 rounded"
  >
    Import
  </button>
  {#if importState.imported > 0}
    <span class="text-xs text-neutral-500">
      Last: {importState.imported} imported, {importState.skipped} skipped
    </span>
  {/if}
{/if}
