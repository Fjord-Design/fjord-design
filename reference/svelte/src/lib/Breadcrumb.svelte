<script lang="ts">
  import type { NavigationItem } from "./complete-types.js";
  let { label, items }: { label: string; items: NavigationItem[] } = $props();
  const current = $derived.by(() => {
    const indices = items
      .map((item, index) => (item.current ? index : -1))
      .filter((index) => index >= 0);
    if (indices.length > 1)
      throw new Error("Breadcrumb supports one current destination.");
    return indices[0] ?? items.length - 1;
  });
</script>

<nav class="fjord-widget fjord-breadcrumb" aria-label={label}>
  <ol>
    {#each items as item, i}<li>
        {#if i > 0}<span aria-hidden="true">/</span
          >{/if}{#if i === current}<span aria-current="page">{item.label}</span
          >{:else}<a href={item.href}>{item.label}</a>{/if}
      </li>{/each}
  </ol>
</nav>
