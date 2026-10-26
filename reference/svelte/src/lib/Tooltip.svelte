<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    id,
    text,
    align = "start",
    children,
  }: {
    id: string;
    text: string;
    align?: "start" | "end";
    children: Snippet<[{ "aria-describedby": string }]>;
  } = $props();
  let wrapper: HTMLSpanElement | undefined = $state();
  let hovering = $state(false);
  let focused = $state(false);
  let touched = $state(false);
  let dismissed = $state(false);
  const checkedId = $derived.by(() => {
    if (!id || /\s/.test(id) || !text?.trim())
      throw new Error("Tooltip requires an ID and text.");
    return id;
  });
  const shown = $derived(!dismissed && (hovering || focused || touched));
  function enter(event: PointerEvent) {
    if (event.pointerType !== "touch") {
      hovering = true;
      dismissed = false;
    }
  }
  function leave(event: PointerEvent) {
    if (event.pointerType !== "touch") hovering = false;
  }
  function inside(event: PointerEvent) {
    if (event.pointerType === "touch") {
      const closing = touched && !dismissed;
      touched = !closing;
      dismissed = closing;
    }
  }
  function outside(event: PointerEvent) {
    if (
      event.pointerType === "touch" &&
      !wrapper?.contains(event.target as Node)
    ) {
      touched = false;
      dismissed = true;
    }
  }
  function key(event: KeyboardEvent) {
    if (event.key === "Escape" && shown) {
      event.preventDefault();
      event.stopPropagation();
      dismissed = true;
      touched = false;
    }
  }
</script>

<svelte:window onpointerdown={outside} onkeydown={key} />
<span
  class="fjord-tooltip-wrap"
  role="presentation"
  data-align={align}
  data-touch-active={touched}
  bind:this={wrapper}
  onpointerenter={enter}
  onpointerleave={leave}
  onpointerdown={inside}
  onfocusin={() => {
    focused = true;
    dismissed = false;
  }}
  onfocusout={(event) => {
    if (!wrapper?.contains(event.relatedTarget as Node | null)) focused = false;
  }}
>
  {@render children({ "aria-describedby": checkedId })}
  <span id={checkedId} role="tooltip" class="fjord-tooltip" hidden={!shown}
    >{text}</span
  >
</span>
