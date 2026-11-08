<script lang="ts">
  import type { Snippet } from "svelte";
  import Popover from "./Popover.svelte";
  let {
    id,
    label,
    children,
  }: { id: string; label: string; children: Snippet } = $props();
</script>

<Popover {id} {label} hover>{@render children()}</Popover>
