<script lang="ts">
  import type { NavigationItem } from "./complete-types.js";
  let { label, items }: { label: string; items: NavigationItem[] } = $props();
</script>

<nav class="fjord-widget fjord-navigation" aria-label={label}>
  <ul>
    {#each items as item}<li>
        <a
          href={item.href}
          aria-current={item.current ? "page" : undefined}
          aria-disabled={item.disabled || undefined}
          onclick={(e) => {
            if (item.disabled) e.preventDefault();
          }}>{item.label}</a
        >
      </li>{/each}
  </ul>
</nav>
