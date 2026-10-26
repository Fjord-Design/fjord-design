<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLSelectAttributes } from "svelte/elements";
  type Props = Omit<
    HTMLSelectAttributes,
    "value" | "multiple" | "class" | "children"
  > & { value?: string; class?: string; children: Snippet };
  let {
    value = $bindable(),
    class: extraClass = "",
    children,
    ...rest
  }: Props = $props();
</script>

<select {...rest} bind:value class={`fjord-form-control ${extraClass}`}
  >{@render children()}</select
>
