<script lang="ts">
  import { tick } from "svelte";
  import type { Snippet } from "svelte";
  let {
    label,
    count,
    children,
  }: { label: string; count: number; children: Snippet } = $props();
  let box: HTMLDivElement;
  let atBottom = $state(true),
    unread = $state(0),
    previous = 0;
  $effect(() => {
    const next = count;
    if (!box) return;
    const added = Math.max(0, next - previous);
    previous = next;
    if (atBottom) {
      void tick().then(() => {
        box.scrollTop = box.scrollHeight;
      });
    } else unread += added;
  });
  function scroll() {
    atBottom = box.scrollHeight - box.scrollTop - box.clientHeight < 8;
    if (atBottom) unread = 0;
  }
  function latest() {
    box.scrollTop = box.scrollHeight;
    atBottom = true;
    unread = 0;
    box.focus();
  }
</script>

<div class="fjord-widget">
  <!-- svelte-ignore a11y_no_noninteractive_tabindex (Named history regions need keyboard scrolling without announcing the full history.) -->
  <div
    bind:this={box}
    class="fjord-message-scroller"
    role="region"
    aria-label={label}
    tabindex="0"
    onscroll={scroll}
  >
    {@render children()}
  </div>
  <span role="status" aria-live="polite"
    >{unread
      ? `${unread} new ${unread === 1 ? "message" : "messages"}`
      : ""}</span
  >{#if !atBottom}<button type="button" onclick={latest}
      >Jump to latest{unread ? ` (${unread})` : ""}</button
    >{/if}
</div>
