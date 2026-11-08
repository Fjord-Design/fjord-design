<script lang="ts">
  import { tick } from "svelte";
  import type { MenuItem } from "./complete-types.js";
  import DropdownMenu from "./DropdownMenu.svelte";
  let {
    id,
    label,
    menus,
    onselect,
  }: {
    id: string;
    label: string;
    menus: { label: string; items: MenuItem[] }[];
    onselect: (id: string) => void;
  } = $props();
  let root: HTMLDivElement;
  let active = $state(0),
    opened = $state(-1);
  async function move(direction: number) {
    if (!menus.length) return;
    active = (active + direction + menus.length) % menus.length;
    if (opened >= 0) opened = active;
    await tick();
    root
      .querySelectorAll<HTMLButtonElement>(":scope > div > button")
      [active]?.focus();
  }
  function key(e: KeyboardEvent) {
    if (e.target instanceof Element && e.target.closest("[role=menu]")) return;
    const rtl = getComputedStyle(root).direction === "rtl";
    if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
      e.preventDefault();
      void move((e.key === "ArrowRight" ? 1 : -1) * (rtl ? -1 : 1));
    } else if (e.key === "Home" || e.key === "End") {
      e.preventDefault();
      void move((e.key === "Home" ? 0 : menus.length - 1) - active);
    }
  }
</script>

<div
  class="fjord-widget fjord-menubar"
  role="menubar"
  tabindex="-1"
  aria-label={label}
  bind:this={root}
  onkeydown={key}
>
  {#each menus as menu, i}<DropdownMenu
      id={`${id}-${i}`}
      label={menu.label}
      items={menu.items}
      {onselect}
      menubar
      tabIndex={i === active ? 0 : -1}
      onfocus={() => (active = i)}
      onhorizontal={move}
      bind:open={
        () => opened === i,
        (next) => {
          if (next) opened = i;
          else if (opened === i) opened = -1;
        }
      }
    />{/each}
</div>
