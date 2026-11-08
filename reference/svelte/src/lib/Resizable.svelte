<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    label,
    value = $bindable(50),
    min = 20,
    max = 80,
    first,
    second,
  }: {
    label: string;
    value?: number;
    min?: number;
    max?: number;
    first: Snippet;
    second: Snippet;
  } = $props();
  let root: HTMLDivElement;
  const split = $derived.by(() => {
    if (
      ![min, max, value].every(Number.isFinite) ||
      min < 0 ||
      max > 100 ||
      min >= max
    )
      throw new Error(
        "Resizable needs 0 <= min < max <= 100 and a finite value.",
      );
    return Math.min(max, Math.max(min, value));
  });
  function move(e: PointerEvent) {
    if (e.buttons !== 1) return;
    const r = root.getBoundingClientRect();
    const rtl = getComputedStyle(root).direction === "rtl";
    value = Math.max(
      min,
      Math.min(
        max,
        ((rtl ? r.right - e.clientX : e.clientX - r.left) / r.width) * 100,
      ),
    );
  }
  function key(e: KeyboardEvent) {
    let target = split;
    const rtl = getComputedStyle(root).direction === "rtl";
    if (e.key === "ArrowLeft") target += rtl ? 2 : -2;
    else if (e.key === "ArrowRight") target += rtl ? -2 : 2;
    else if (e.key === "Home") target = min;
    else if (e.key === "End") target = max;
    else return;
    e.preventDefault();
    value = Math.min(max, Math.max(min, target));
  }
</script>

<div
  class="fjord-widget fjord-resizable"
  bind:this={root}
  style:grid-template-columns={`minmax(0,${split}fr) auto minmax(0,${100 - split}fr)`}
>
  <div>{@render first()}</div>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions (APG window splitter is a focusable range separator.) -->
  <div
    role="separator"
    aria-label={label}
    aria-orientation="vertical"
    aria-valuenow={Math.round(split)}
    aria-valuemin={min}
    aria-valuemax={max}
    tabindex="0"
    class="fjord-resize-handle"
    onkeydown={key}
    onpointerdown={(e) => {
      e.preventDefault();
      e.currentTarget.focus();
      e.currentTarget.setPointerCapture(e.pointerId);
    }}
    onpointermove={move}
  >
    <span aria-hidden="true">⋮</span>
  </div>
  <div>{@render second()}</div>
</div>
