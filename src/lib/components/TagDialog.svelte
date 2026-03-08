<script lang="ts">
  import {
    listTagCategories,
    createTagCategory,
    tagTracks,
    getTrackTags,
  } from "$lib/api";
  import type { TagCategory, TagValue } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let {
    trackIds,
    onclose,
  }: {
    trackIds: number[];
    onclose: () => void;
  } = $props();

  let categories = $state<TagCategory[]>([]);
  let selectedCategoryId = $state<number | null>(null);
  let values = $state<TagValue[]>([]);
  let newCategoryName = $state("");
  let newValueName = $state("");
  let existingTags = $state<TagValue[]>([]);

  $effect(() => {
    loadData();
  });

  async function loadData() {
    categories = await listTagCategories();
    if (trackIds.length === 1) {
      existingTags = await getTrackTags(trackIds[0]);
    }
  }

  async function loadValues() {
    if (selectedCategoryId === null) {
      values = [];
      return;
    }
    // We need to get values for this category
    values = await invoke("get_values_for_category", {
      categoryId: selectedCategoryId,
    });
  }

  async function handleCreateCategory() {
    const name = newCategoryName.trim();
    if (!name) return;
    const id = await createTagCategory(name);
    newCategoryName = "";
    await loadData();
    selectedCategoryId = id;
    await loadValues();
  }

  async function handleCreateValueAndTag() {
    const value = newValueName.trim();
    if (!value || selectedCategoryId === null) return;
    const valueId: number = await invoke("create_tag_value", {
      categoryId: selectedCategoryId,
      value,
    });
    await tagTracks(trackIds, valueId);
    newValueName = "";
    await loadValues();
    if (trackIds.length === 1) {
      existingTags = await getTrackTags(trackIds[0]);
    }
  }

  async function handleApplyTag(valueId: number) {
    await tagTracks(trackIds, valueId);
    if (trackIds.length === 1) {
      existingTags = await getTrackTags(trackIds[0]);
    }
  }

  function selectCategory(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    selectedCategoryId = val ? parseInt(val) : null;
    loadValues();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center"
  onclick={onclose}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="bg-neutral-800 rounded-lg shadow-xl p-4 w-80 max-h-96 overflow-y-auto"
    onclick={(e) => e.stopPropagation()}
  >
    <h3 class="text-sm font-semibold text-neutral-100 mb-3">
      Tag {trackIds.length} track{trackIds.length > 1 ? "s" : ""}
    </h3>

    {#if existingTags.length > 0}
      <div class="mb-3">
        <div class="text-xs text-neutral-400 mb-1">Current tags:</div>
        <div class="flex flex-wrap gap-1">
          {#each existingTags as tag}
            <span class="px-1.5 py-0.5 text-xs bg-blue-900 rounded text-blue-200">
              {tag.category_name}: {tag.value}
            </span>
          {/each}
        </div>
      </div>
    {/if}

    <div class="space-y-2">
      <div>
        <label class="text-xs text-neutral-400">Category</label>
        <div class="flex gap-1">
          <select
            onchange={selectCategory}
            class="flex-1 px-1 py-0.5 text-xs bg-neutral-700 border border-neutral-600 rounded text-neutral-300"
          >
            <option value="">Select...</option>
            {#each categories as cat}
              <option value={cat.id}>{cat.name}</option>
            {/each}
          </select>
        </div>
        <div class="flex gap-1 mt-1">
          <input
            type="text"
            placeholder="New category..."
            bind:value={newCategoryName}
            onkeydown={(e) => e.key === "Enter" && handleCreateCategory()}
            class="flex-1 px-1 py-0.5 text-xs bg-neutral-700 border border-neutral-600 rounded text-neutral-300 placeholder-neutral-500"
          />
          <button
            onclick={handleCreateCategory}
            class="px-1.5 py-0.5 text-xs bg-blue-700 hover:bg-blue-600 rounded text-neutral-200"
          >
            +
          </button>
        </div>
      </div>

      {#if selectedCategoryId !== null}
        <div>
          <label class="text-xs text-neutral-400">Value</label>
          {#if values.length > 0}
            <div class="flex flex-wrap gap-1 mb-1">
              {#each values as val}
                <button
                  onclick={() => handleApplyTag(val.id)}
                  class="px-1.5 py-0.5 text-xs bg-neutral-700 hover:bg-neutral-600 rounded text-neutral-300"
                >
                  {val.value}
                </button>
              {/each}
            </div>
          {/if}
          <div class="flex gap-1">
            <input
              type="text"
              placeholder="New value..."
              bind:value={newValueName}
              onkeydown={(e) =>
                e.key === "Enter" && handleCreateValueAndTag()}
              class="flex-1 px-1 py-0.5 text-xs bg-neutral-700 border border-neutral-600 rounded text-neutral-300 placeholder-neutral-500"
            />
            <button
              onclick={handleCreateValueAndTag}
              class="px-1.5 py-0.5 text-xs bg-blue-700 hover:bg-blue-600 rounded text-neutral-200"
            >
              Add
            </button>
          </div>
        </div>
      {/if}
    </div>

    <div class="mt-3 flex justify-end">
      <button
        onclick={onclose}
        class="px-3 py-1 text-xs bg-neutral-700 hover:bg-neutral-600 rounded text-neutral-300"
      >
        Close
      </button>
    </div>
  </div>
</div>
