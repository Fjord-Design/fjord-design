<script lang="ts">
  import type { Snippet } from "svelte";
  type Element = "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "span";
  type Role =
    | "display"
    | "title-1"
    | "title-2"
    | "title-3"
    | "title-4"
    | "lead"
    | "body"
    | "body-strong"
    | "label"
    | "small"
    | "caption"
    | "overline";
  let {
    as = "p",
    role = "body",
    children,
  }: { as?: Element; role?: Role; children: Snippet } = $props();
</script>

<svelte:element this={as} class={`fjord-type-${role}`}
  >{@render children()}</svelte:element
>
