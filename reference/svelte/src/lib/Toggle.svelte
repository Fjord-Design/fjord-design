<script lang="ts">
  let {
    label,
    pressed = $bindable(false),
    disabled = false,
    onchange,
  }: {
    label: string;
    pressed?: boolean;
    disabled?: boolean;
    onchange?: (pressed: boolean) => void;
  } = $props();
</script>

<button
  class="fjord-widget fjord-toggle"
  type="button"
  aria-pressed={pressed}
  {disabled}
  onclick={() => {
    pressed = !pressed;
    onchange?.(pressed);
  }}>{label}</button
>
