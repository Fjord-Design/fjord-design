<script lang="ts">
  import type { Snippet } from "svelte";
  import PanelDialog from "./PanelDialog.svelte";
  let {
    id,
    title,
    open = $bindable(false),
    edge = "bottom",
    modal = true,
    children,
  }: {
    id: string;
    title: string;
    open?: boolean;
    edge?: "start" | "end" | "bottom";
    modal?: boolean;
    children: Snippet;
  } = $props();
</script>

<PanelDialog {id} {title} bind:open {edge} {modal} gesture
  >{@render children()}</PanelDialog
>
