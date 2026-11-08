<script lang="ts">
  let {
    id,
    label,
    value = $bindable(0),
    name,
    min = 0,
    max = 100,
    step = 1,
    disabled = false,
    precise = true,
  }: {
    id: string;
    label: string;
    value?: number;
    name?: string;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
    precise?: boolean;
  } = $props();
  const checked = $derived.by(() => {
    if (
      ![min, max, step, value].every(Number.isFinite) ||
      min >= max ||
      step <= 0
    )
      throw new Error("Slider needs finite min < max and positive step.");
    return Math.min(max, Math.max(min, value));
  });
  function update(e: Event) {
    const n = Number((e.currentTarget as HTMLInputElement).value);
    if (Number.isFinite(n))
      value = Math.min(
        max,
        Math.max(min, min + Math.round((n - min) / step) * step),
      );
  }
</script>

<div class="fjord-widget fjord-slider">
  <label for={id}>{label}</label><input
    {id}
    {name}
    type="range"
    value={checked}
    {min}
    {max}
    {step}
    {disabled}
    aria-describedby={`${id}-value`}
    oninput={update}
  /><output id={`${id}-value`} for={id}>{checked}</output>{#if precise}<label
      for={`${id}-exact`}
      >Exact {label}<input
        id={`${id}-exact`}
        type="number"
        value={checked}
        {min}
        {max}
        {step}
        {disabled}
        oninput={update}
      /></label
    >{/if}
</div>
