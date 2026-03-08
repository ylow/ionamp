<script lang="ts">
  import SplitPane from "./SplitPane.svelte";
  import Toolbar from "./Toolbar.svelte";
  import SearchPane from "./SearchPane.svelte";
  import PlaylistPane from "./PlaylistPane.svelte";
  import PropertiesDialog from "./PropertiesDialog.svelte";
  import TagDialog from "./TagDialog.svelte";

  let propertiesTrackId = $state<number | null>(null);
  let tagTrackIds = $state<number[] | null>(null);

  $effect(() => {
    const handleProperties = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      if (detail?.trackId) {
        propertiesTrackId = detail.trackId;
      }
    };

    const handleTag = (e: Event) => {
      const detail = (e as CustomEvent).detail;
      if (detail?.trackIds) {
        tagTrackIds = detail.trackIds;
      }
    };

    document.addEventListener("show-properties", handleProperties);
    document.addEventListener("show-tag-dialog", handleTag);

    return () => {
      document.removeEventListener("show-properties", handleProperties);
      document.removeEventListener("show-tag-dialog", handleTag);
    };
  });
</script>

<div class="h-screen flex flex-col bg-neutral-900 text-neutral-100">
  <Toolbar />
  <div class="flex-1 overflow-hidden">
    <SplitPane>
      {#snippet left()}
        <SearchPane />
      {/snippet}
      {#snippet right()}
        <PlaylistPane />
      {/snippet}
    </SplitPane>
  </div>
</div>

{#if propertiesTrackId !== null}
  <PropertiesDialog
    trackId={propertiesTrackId}
    onclose={() => (propertiesTrackId = null)}
  />
{/if}

{#if tagTrackIds !== null}
  <TagDialog
    trackIds={tagTrackIds}
    onclose={() => (tagTrackIds = null)}
  />
{/if}
