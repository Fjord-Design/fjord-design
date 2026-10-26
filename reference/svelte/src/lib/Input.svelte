<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  type Props = Omit<
    HTMLInputAttributes,
    "type" | "value" | "class" | "children"
  > & {
    type?: "text" | "email" | "password" | "search" | "tel" | "url";
    value?: string;
    class?: string;
  };
  let {
    type = "text",
    value = $bindable(),
    class: extraClass = "",
    ...rest
  }: Props = $props();
</script>

<input {...rest} {type} bind:value class={`fjord-form-control ${extraClass}`} />
