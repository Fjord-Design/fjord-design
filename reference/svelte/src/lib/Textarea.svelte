<script lang="ts">
  import type { HTMLTextareaAttributes } from "svelte/elements";
  type Props = Omit<HTMLTextareaAttributes, "value" | "class" | "children"> & {
    value?: string;
    class?: string;
  };
  let {
    value = $bindable(),
    class: extraClass = "",
    ...rest
  }: Props = $props();
</script>

<textarea {...rest} bind:value class={`fjord-form-control ${extraClass}`}
></textarea>
