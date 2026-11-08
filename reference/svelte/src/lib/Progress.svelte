<script lang="ts">
  let {
    id,
    label,
    value,
    max = 100,
  }: { id: string; label: string; value?: number; max?: number } = $props();
  const checked = $derived.by(() => {
    if (
      !Number.isFinite(max) ||
      max <= 0 ||
      (value !== undefined &&
        (!Number.isFinite(value) || value < 0 || value > max))
    )
      throw new Error(
        "Progress needs a positive max and a value within range.",
      );
    return value;
  });
</script>

<div class="fjord-widget fjord-progress">
  {#if checked === undefined}<span id={`${id}-label`}>{label}</span>
    <div
      {id}
      class="fjord-progress-unknown"
      role="progressbar"
      aria-labelledby={`${id}-label`}
      aria-valuemin={0}
      aria-valuemax={max}
    ></div>
    <span>In progress</span>
  {:else}<label for={id}>{label}</label><progress {id} value={checked} {max}
    ></progress><span>{Math.round((checked / max) * 100)}%</span>{/if}
</div>
