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
  let range = $state<HTMLInputElement>();
  function decimalPlaces(n: number) {
    const [digits, exponent = "0"] = n.toString().split("e");
    return Math.max(0, (digits.split(".")[1]?.length ?? 0) - Number(exponent));
  }
  function normalize(n: number) {
    const precision = Math.max(decimalPlaces(min), decimalPlaces(step));
    const atStep = (index: number, places = precision) => {
      const result = min + index * step;
      return places <= 100 ? Number(result.toFixed(places)) : result;
    };
    let lastStep = Math.floor((max - min) / step);
    // Decimal division can put an aligned maximum just below its step index.
    if (atStep(lastStep + 1) <= max) lastStep += 1;
    const bounded = Math.min(max, Math.max(min, n));
    const position = (bounded - min) / step;
    if (Number.isInteger(position)) return atStep(Math.min(lastStep, position));
    const lowerStep = Math.floor(position);
    const midpoint = atStep(lowerStep + 0.5, precision + 1);
    // Match native ranges: nearest valid step, with ties toward the larger value.
    return atStep(
      Math.min(lastStep, bounded < midpoint ? lowerStep : lowerStep + 1),
    );
  }
  const checked = $derived.by(() => {
    if (
      ![min, max, step, value].every(Number.isFinite) ||
      min >= max ||
      step <= 0
    )
      throw new Error("Slider needs finite min < max and positive step.");
    return normalize(value);
  });
  $effect(() => {
    if (value !== checked) value = checked;
    // Bounds can sanitize the DOM even when the normalized value is unchanged.
    if (range) range.value = String(normalize(value));
  });
  function update(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const n = Number(input.value);
    if (Number.isFinite(n)) {
      value = normalize(n);
      // An edit may normalize to the existing state, without a reactive update.
      input.value = String(value);
    }
  }
</script>

<div class="fjord-widget fjord-slider">
  <label for={id}>{label}</label><input
    {id}
    {name}
    type="range"
    bind:this={range}
    {min}
    {max}
    {step}
    value={checked}
    {disabled}
    aria-describedby={`${id}-value`}
    oninput={update}
  /><output id={`${id}-value`} for={id}>{checked}</output>{#if precise}<label
      for={`${id}-exact`}
      >Exact {label}<input
        id={`${id}-exact`}
        type="number"
        {min}
        {max}
        {step}
        value={checked}
        {disabled}
        onchange={update}
      /></label
    >{/if}
</div>
