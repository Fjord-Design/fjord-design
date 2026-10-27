<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    id,
    caption,
    children,
  }: { id: string; caption: string; children: Snippet } = $props();
  const checkedId = $derived.by(() => {
    if (!id || /\s/.test(id) || !caption?.trim())
      throw new Error("Table requires an ID and a caption.");
    return id;
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex (The named scroll region must be keyboard accessible.) -->
<div
  id={checkedId}
  class="fjord-table-scroll"
  role="region"
  aria-labelledby={`${checkedId}-caption`}
  tabindex="0"
>
  <table class="fjord-table">
    <caption id={`${checkedId}-caption`}>{caption}</caption>{@render children()}
  </table>
</div>
