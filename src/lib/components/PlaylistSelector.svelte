<script lang="ts">
  import { onMount } from "svelte";
  import { playlistState } from "$lib/stores/playlist.svelte";

  let newName = $state("");
  let selectValue = $state("");

  onMount(() => {
    console.log("[PlaylistSelector] mounted, loading playlists");
    playlistState.loadPlaylists();
  });

  // Sync selectValue from store
  $effect(() => {
    const storeVal = playlistState.selectedPlaylistId;
    const newVal = storeVal?.toString() ?? "";
    console.log("[PlaylistSelector] syncing selectValue from store:", storeVal, "->", newVal);
    selectValue = newVal;
  });

  function handleSelect(e: Event) {
    const value = (e.target as HTMLSelectElement).value;
    console.log("[PlaylistSelector] handleSelect:", value);
    selectValue = value;
    playlistState.selectPlaylist(value ? parseInt(value) : null);
  }

  async function handleCreate() {
    const name = newName.trim();
    if (!name) return;
    console.log("[PlaylistSelector] creating playlist:", name);
    await playlistState.create(name);
    newName = "";
  }

  async function handleDelete() {
    if (playlistState.selectedPlaylistId === null) return;
    console.log("[PlaylistSelector] deleting playlist:", playlistState.selectedPlaylistId);
    await playlistState.remove(playlistState.selectedPlaylistId);
  }
</script>

<div class="flex items-center gap-1">
  <select
    bind:value={selectValue}
    onchange={handleSelect}
    class="flex-1 px-1 py-0.5 text-xs bg-neutral-800 border border-neutral-600 rounded text-neutral-300"
  >
    <option value="">Select playlist...</option>
    {#each playlistState.playlists as playlist}
      <option value={playlist.id.toString()}>{playlist.name}</option>
    {/each}
  </select>
  <button
    onclick={handleDelete}
    disabled={playlistState.selectedPlaylistId === null}
    class="px-1.5 py-0.5 text-xs bg-neutral-700 hover:bg-neutral-600 rounded text-neutral-300 disabled:opacity-30"
    title="Delete playlist"
  >
    ×
  </button>
</div>
<div class="flex items-center gap-1 mt-1">
  <input
    type="text"
    placeholder="New playlist..."
    bind:value={newName}
    onkeydown={(e) => e.key === "Enter" && handleCreate()}
    class="flex-1 px-1 py-0.5 text-xs bg-neutral-800 border border-neutral-600 rounded text-neutral-300 placeholder-neutral-500"
  />
  <button
    onclick={handleCreate}
    class="px-1.5 py-0.5 text-xs bg-blue-700 hover:bg-blue-600 rounded text-neutral-200"
  >
    +
  </button>
</div>
