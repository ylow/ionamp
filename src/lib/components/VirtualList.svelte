<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    items,
    itemHeight = 24,
    keyFn = (item: any, index: number) => index,
    row,
  }: {
    items: any[];
    itemHeight?: number;
    keyFn?: (item: any, index: number) => any;
    row: Snippet<[item: any, index: number]>;
  } = $props();

  let container: HTMLDivElement;
  let scrollTop = $state(0);
  let containerHeight = $state(600);
  const buffer = 5;

  let totalHeight = $derived(items.length * itemHeight);
  let startIndex = $derived(Math.max(0, Math.floor(scrollTop / itemHeight) - buffer));
  let endIndex = $derived(
    Math.min(
      items.length,
      Math.ceil((scrollTop + containerHeight) / itemHeight) + buffer,
    ),
  );
  let visibleItems = $derived(items.slice(startIndex, endIndex));
  let offsetY = $derived(startIndex * itemHeight);

  function onScroll() {
    if (container) {
      scrollTop = container.scrollTop;
    }
  }

  $effect(() => {
    if (container) {
      containerHeight = container.clientHeight;
      const observer = new ResizeObserver(() => {
        containerHeight = container.clientHeight;
      });
      observer.observe(container);
      return () => observer.disconnect();
    }
  });
</script>

<div
  bind:this={container}
  class="overflow-y-auto h-full"
  onscroll={onScroll}
>
  <div style="height: {totalHeight}px; position: relative;">
    <div style="transform: translateY({offsetY}px);">
      {#each visibleItems as item, i (keyFn(item, startIndex + i))}
        {@render row(item, startIndex + i)}
      {/each}
    </div>
  </div>
</div>
