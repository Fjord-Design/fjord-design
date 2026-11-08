<script lang="ts">
  import type { ChartDatum } from "./complete-types.js";
  let { label, data }: { label: string; data: ChartDatum[] } = $props();
  const extent = $derived.by(() => {
    if (data.some((d) => !Number.isFinite(d.value) || !d.label.trim()))
      throw new Error("Chart needs labels and finite values.");
    return Math.max(1, ...data.map((d) => Math.abs(d.value)));
  });
</script>

<figure class="fjord-widget fjord-chart">
  <figcaption>{label}</figcaption>
  {#if data.length}<svg
      viewBox={`0 0 360 ${data.length * 32 + 16}`}
      role="img"
      aria-label={`${label}: bar chart; complete values in the table below`}
      ><line
        x1="170"
        y1="0"
        x2="170"
        y2={data.length * 32}
        stroke="var(--fjord-line-strong)"
      />{#each data as datum, i}<text
          x="0"
          y={i * 32 + 20}
          fill="var(--fjord-ink)">{i + 1}</text
        ><rect
          x={170 + Math.min(0, datum.value / extent) * 140}
          y={i * 32 + 5}
          width={Math.max(1, Math.abs(datum.value / extent) * 140)}
          height="20"
          fill={`var(--fjord-chart-${(i % 5) + 1})`}
        /><text x="320" y={i * 32 + 20} fill="var(--fjord-ink)">{i + 1}</text
        >{/each}</svg
    >
    <table>
      <caption>{label}, complete values</caption><thead
        ><tr
          ><th scope="col">Series</th><th scope="col">Label</th><th scope="col"
            >Value</th
          ></tr
        ></thead
      ><tbody
        >{#each data as datum, i}<tr
            ><th scope="row">{i + 1}</th><td>{datum.label}</td><td
              >{datum.value.toLocaleString()}</td
            ></tr
          >{/each}</tbody
      >
    </table>{:else}<p>No data</p>{/if}
</figure>
