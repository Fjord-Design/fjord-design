<script lang="ts">
  import { onDestroy } from "svelte";
  import type { Snippet } from "svelte";
  let {
    id,
    label,
    open = $bindable(false),
    hover = false,
    children,
  }: {
    id: string;
    label: string;
    open?: boolean;
    hover?: boolean;
    children: Snippet;
  } = $props();
  let panel: HTMLDivElement;
  let trigger: HTMLButtonElement;
  let position = $state({ left: 0, top: 0 });
  let pointerFocus = false,
    restoringFocus = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  function place() {
    if (!trigger || !panel) return;
    const r = trigger.getBoundingClientRect(),
      p = panel.getBoundingClientRect();
    position = {
      left: Math.max(8, Math.min(r.left, innerWidth - p.width - 8)),
      top:
        r.bottom + p.height + 8 > innerHeight
          ? Math.max(8, r.top - p.height - 8)
          : r.bottom + 8,
    };
  }
  function cancel() {
    clearTimeout(timer);
  }
  function leave() {
    if (!hover) return;
    cancel();
    timer = setTimeout(() => {
      if (
        !panel?.contains(document.activeElement) &&
        document.activeElement !== trigger
      )
        open = false;
    }, 180);
  }
  $effect(() => {
    if (!panel) return;
    if (open && !panel.matches(":popover-open")) panel.showPopover();
    else if (!open && panel.matches(":popover-open")) panel.hidePopover();
    if (!open) return;
    place();
    window.addEventListener("scroll", place, true);
    const observer = new ResizeObserver(place);
    observer.observe(panel);
    return () => {
      window.removeEventListener("scroll", place, true);
      observer.disconnect();
    };
  });
  onDestroy(cancel);
  function toggled(e: ToggleEvent) {
    open = e.newState === "open";
    if (open) place();
  }
</script>

<svelte:window onresize={place} />
<span class="fjord-widget fjord-popover-trigger"
  ><button
    type="button"
    bind:this={trigger}
    popovertarget={id}
    aria-expanded={open}
    onpointerenter={(e) => {
      if (hover && e.pointerType === "mouse") {
        cancel();
        open = true;
      }
    }}
    onpointerleave={leave}
    onpointerdown={() => {
      pointerFocus = true;
    }}
    onclick={() => {
      pointerFocus = false;
    }}
    onpointercancel={() => {
      pointerFocus = false;
    }}
    onfocus={() => {
      if (hover && !pointerFocus && !restoringFocus) {
        cancel();
        open = true;
      }
    }}
    onblur={() => {
      pointerFocus = false;
      leave();
    }}>{label}</button
  ></span
>
<div
  bind:this={panel}
  {id}
  popover="auto"
  class="fjord-widget fjord-popover"
  role="region"
  aria-label={label}
  onbeforetoggle={(e) => {
    if (
      (e as ToggleEvent).newState === "closed" &&
      panel?.contains(document.activeElement)
    ) {
      restoringFocus = true;
      trigger?.focus();
      restoringFocus = false;
    }
  }}
  ontoggle={toggled}
  onpointerenter={cancel}
  onpointerleave={leave}
  onfocusin={cancel}
  onfocusout={leave}
  style:left={`${position.left}px`}
  style:top={`${position.top}px`}
>
  {@render children()}
</div>
