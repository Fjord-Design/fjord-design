<script lang="ts">
  let {
    id,
    label,
    name,
    value = $bindable(""),
    length = 6,
    disabled = false,
    error,
    required = false,
  }: {
    id: string;
    label: string;
    name?: string;
    value?: string;
    length?: number;
    disabled?: boolean;
    error?: string;
    required?: boolean;
  } = $props();
  const count = $derived.by(() => {
    if (!Number.isInteger(length) || length < 1 || length > 32)
      throw new Error("InputOTP length must be 1–32.");
    return length;
  });
  function input(e: Event) {
    const el = e.currentTarget as HTMLInputElement;
    value = el.value.replace(/\D/g, "").slice(0, count);
    el.value = value;
  }
</script>

<div class="fjord-widget fjord-otp">
  <label for={id}>{label}</label><input
    {id}
    {name}
    {value}
    type="text"
    inputmode="numeric"
    autocomplete="one-time-code"
    pattern={`[0-9]{${count}}`}
    maxlength={count}
    {disabled}
    {required}
    aria-invalid={error ? "true" : undefined}
    aria-describedby={error ? `${id}-error` : undefined}
    oninput={input}
  />{#if error}<p id={`${id}-error`}>{error}</p>{/if}
</div>
