<script lang="ts">
  import type { MenuItem } from "./complete-types.js";
  import DropdownMenu from "./DropdownMenu.svelte";
  let {
    id,
    label,
    items,
    onselect,
  }: {
    id: string;
    label: string;
    items: MenuItem[];
    onselect: (id: string) => void;
  } = $props();
  let open = $state(false);
</script>

<div
  class="fjord-widget"
  role="group"
  aria-label={label}
  oncontextmenu={(e) => {
    e.preventDefault();
    open = true;
  }}
>
  <DropdownMenu {id} {label} {items} {onselect} bind:open />
</div>
