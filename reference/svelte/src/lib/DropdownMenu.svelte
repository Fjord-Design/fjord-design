<script lang="ts">
  import { tick } from "svelte";
  import type { MenuItem } from "./complete-types.js";
  import MenuPanel from "./MenuPanel.svelte";
  let {
    id,
    label,
    items,
    onselect,
    open = $bindable(false),
    menubar = false,
    tabIndex = 0,
    onfocus,
    onhorizontal,
  }: {
    id: string;
    label: string;
    items: MenuItem[];
    onselect: (id: string) => void;
    open?: boolean;
    menubar?: boolean;
    tabIndex?: number;
    onfocus?: () => void;
    onhorizontal?: (direction: number) => void;
  } = $props();
  let trigger: HTMLButtonElement;
  let root: HTMLDivElement;
  let menu: MenuPanel;
  let panel: HTMLDivElement;
  let position = $state({ left: 0, top: 0 });
  function place() {
    if (!trigger || !panel) return;
    const r = trigger.getBoundingClientRect(),
      p = panel.getBoundingClientRect();
    position = {
      left: Math.max(8, Math.min(r.left, innerWidth - p.width - 8)),
      top:
        r.bottom + p.height > innerHeight
          ? Math.max(8, r.top - p.height)
          : r.bottom,
    };
  }
  async function show() {
    open = true;
    await tick();
    place();
    menu?.focusFirst();
  }
  function close() {
    open = false;
    trigger?.focus();
  }
  function outside(e: PointerEvent) {
    if (open && e.target instanceof Node && !root.contains(e.target))
      open = false;
  }
  $effect(() => {
    if (!panel) return;
    if (open && !panel.matches(":popover-open")) {
      panel.showPopover();
      place();
      void tick().then(() => menu?.focusFirst());
    } else if (!open && panel.matches(":popover-open")) panel.hidePopover();
    if (!open) return;
    window.addEventListener("scroll", place, true);
    return () => window.removeEventListener("scroll", place, true);
  });
</script>

<svelte:window onpointerdown={outside} onresize={place} />
<div class="fjord-widget fjord-dropdown" bind:this={root}>
  <button
    type="button"
    bind:this={trigger}
    role={menubar ? "menuitem" : undefined}
    tabindex={tabIndex}
    {onfocus}
    aria-haspopup="menu"
    aria-expanded={open}
    aria-controls={id}
    onclick={() => (open ? close() : show())}
    onkeydown={(e) => {
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        void show();
      }
    }}>{label}</button
  >
  <div
    {id}
    bind:this={panel}
    popover="auto"
    class="fjord-popover fjord-widget"
    style:left={`${position.left}px`}
    style:top={`${position.top}px`}
    ontoggle={(e) => {
      if ((e as ToggleEvent).newState === "closed") open = false;
    }}
  >
    <MenuPanel
      bind:this={menu}
      {label}
      {items}
      onselect={(id) => {
        onselect(id);
        close();
      }}
      onclose={close}
      {onhorizontal}
    />
  </div>
</div>
