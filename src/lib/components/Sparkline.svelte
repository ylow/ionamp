<script lang="ts">
  let {
    energyVector,
    width = 80,
    height = 20,
    color = "#60a5fa",
  }: {
    energyVector: number[] | null;
    width?: number;
    height?: number;
    color?: string;
  } = $props();

  let pathD = $derived.by(() => {
    if (!energyVector || energyVector.length === 0) return "";
    const points = energyVector;
    const stepX = width / (points.length - 1 || 1);

    let d = `M 0 ${height}`;
    for (let i = 0; i < points.length; i++) {
      const x = i * stepX;
      const y = height - points[i] * height;
      d += ` L ${x.toFixed(1)} ${y.toFixed(1)}`;
    }
    d += ` L ${width} ${height} Z`;
    return d;
  });
</script>

{#if energyVector && energyVector.length > 0}
  <svg {width} {height} viewBox="0 0 {width} {height}" class="inline-block">
    <path d={pathD} fill={color} fill-opacity="0.4" stroke={color} stroke-width="1" />
  </svg>
{:else}
  <div style="width: {width}px; height: {height}px;" class="inline-block"></div>
{/if}
