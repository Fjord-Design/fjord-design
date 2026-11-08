<script lang="ts">
  import type { Snippet } from "svelte";
  import PanelDialog from "./PanelDialog.svelte";
  let {
    id,
    title,
    description,
    open = $bindable(false),
    cancelLabel = "Cancel",
    children,
    actions,
  }: {
    id: string;
    title: string;
    description: string;
    open?: boolean;
    cancelLabel?: string;
    children: Snippet;
    actions: Snippet;
  } = $props();
</script>

<PanelDialog
  {id}
  {title}
  {description}
  bind:open
  kind="alertdialog"
  closeLabel={cancelLabel}
  {actions}>{@render children()}</PanelDialog
>
