<script lang="ts">
  import type { NavigationItem } from "./complete-types.js";
  let {
    id,
    label,
    items,
    compact = $bindable(false),
  }: {
    id: string;
    label: string;
    items: NavigationItem[];
    compact?: boolean;
  } = $props();
  let mobileOpen = $state(false);
  let mobileToggle: HTMLButtonElement;
  let aside: HTMLElement;
</script>

<svelte:window
  onkeydown={(e) => {
    if (
      e.key === "Escape" &&
      mobileOpen &&
      e.target instanceof Node &&
      aside?.contains(e.target)
    ) {
      mobileOpen = false;
      mobileToggle?.focus();
    }
  }}
/>
<aside
  bind:this={aside}
  class="fjord-widget fjord-sidebar"
  data-compact={compact}
  data-mobile-open={mobileOpen}
>
  <button
    type="button"
    class="fjord-sidebar-density"
    aria-pressed={compact}
    onclick={() => (compact = !compact)}>Compact {label}</button
  >
  <button
    type="button"
    class="fjord-sidebar-mobile"
    bind:this={mobileToggle}
    aria-expanded={mobileOpen}
    aria-controls={id}
    onclick={() => (mobileOpen = !mobileOpen)}
    >{mobileOpen ? "Close" : "Open"} {label}</button
  >
  <nav {id} aria-label={label}>
    <ul>
      {#each items as item}<li>
          <a
            href={item.href}
            aria-label={compact ? item.label : undefined}
            title={compact ? item.label : undefined}
            aria-current={item.current ? "page" : undefined}
            aria-disabled={item.disabled || undefined}
            onclick={(e) => {
              if (item.disabled) e.preventDefault();
            }}>{compact ? item.label.slice(0, 2) : item.label}</a
          >
        </li>{/each}
    </ul>
  </nav>
</aside>
