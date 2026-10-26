<script lang="ts">
  import type { RadioOption } from "./types.js";
  let {
    id,
    label,
    name,
    options,
    value = $bindable(),
    disabled = false,
    required = false,
    help,
    error,
  }: {
    id: string;
    label: string;
    name: string;
    options: RadioOption[];
    value?: string;
    disabled?: boolean;
    required?: boolean;
    help?: string;
    error?: string;
  } = $props();
  const checkedId = $derived.by(() => {
    if (!id || /\s/.test(id))
      throw new Error("RadioGroup requires an ID without whitespace.");
    if (!label?.trim() || !name?.trim())
      throw new Error("RadioGroup requires a non-empty label and name.");
    if (
      !options.length ||
      options.some(
        (option) => !option.label?.trim() || !option.value?.trim(),
      ) ||
      new Set(options.map((option) => option.value)).size !== options.length
    )
      throw new Error(
        "RadioGroup options require unique non-empty values and labels.",
      );
    return id;
  });
  const hasHelp = $derived(Boolean(help?.trim()));
  const hasError = $derived(Boolean(error?.trim()));
  const description = $derived(
    [hasHelp && `${checkedId}-help`, hasError && `${checkedId}-error`]
      .filter(Boolean)
      .join(" ") || undefined,
  );
</script>

<fieldset
  id={checkedId}
  class="fjord-radio-group"
  role="radiogroup"
  {disabled}
  aria-describedby={description}
  aria-invalid={hasError ? "true" : undefined}
  aria-required={required ? "true" : undefined}
>
  <legend
    >{label}{#if required}<span aria-hidden="true"> *</span>{/if}</legend
  >
  {#each options as option, index (option.value)}
    <label class="fjord-choice">
      <input
        id={`${checkedId}-${index}`}
        type="radio"
        class="fjord-choice-input"
        {name}
        value={option.value}
        disabled={option.disabled}
        {required}
        aria-describedby={description}
        bind:group={value}
      />
      <span class="fjord-choice-label">{option.label}</span>
    </label>
  {/each}
  {#if hasHelp}<p id={`${checkedId}-help`} class="help">{help}</p>{/if}
  {#if hasError}<p id={`${checkedId}-error`} class="error">{error}</p>{/if}
</fieldset>
