<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    left,
    right,
    leftWidth = 65,
  }: { left: Snippet; right: Snippet; leftWidth?: number } = $props();

  let width = $state(leftWidth);
  let dragging = $state(false);
  let container: HTMLDivElement;

  function onMouseDown(e: MouseEvent) {
    e.preventDefault();
    dragging = true;
  }

  function onMouseMove(e: MouseEvent) {
    if (!dragging || !container) return;
    const rect = container.getBoundingClientRect();
    const pct = ((e.clientX - rect.left) / rect.width) * 100;
    width = Math.max(20, Math.min(80, pct));
  }

  function onMouseUp() {
    dragging = false;
  }
</script>

<svelte:window onmousemove={onMouseMove} onmouseup={onMouseUp} />

<div
  bind:this={container}
  class="flex h-full w-full overflow-hidden"
  class:select-none={dragging}
>
  <div class="overflow-hidden" style="width: {width}%; flex-shrink: 0;">
    {@render left()}
  </div>
  <div
    class="w-1 cursor-col-resize bg-neutral-700 hover:bg-blue-500 transition-colors flex-shrink-0"
    class:bg-blue-500={dragging}
    role="separator"
    tabindex="-1"
    onmousedown={onMouseDown}
  ></div>
  <div class="flex-1 overflow-hidden">
    {@render right()}
  </div>
</div>
