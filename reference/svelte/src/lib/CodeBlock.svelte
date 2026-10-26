<script lang="ts">
  import type { CodeSegment } from "./types.js";
  let {
    label,
    code,
    segments,
    copyLabel = "Copy",
    copiedMessage = "Copied",
    copyFailedMessage = "Copy failed. Select and copy the code manually.",
  }: {
    label: string;
    code: string;
    segments?: CodeSegment[];
    copyLabel?: string;
    copiedMessage?: string;
    copyFailedMessage?: string;
  } = $props();
  let feedback = $state("");
  let attempt = 0;
  const checked = $derived.by(() => {
    if (
      !label?.trim() ||
      !copyLabel?.trim() ||
      !copiedMessage?.trim() ||
      !copyFailedMessage?.trim()
    )
      throw new Error("CodeBlock requires a label and copy messages.");
    if (segments && segments.map((part) => part.text).join("") !== code)
      throw new Error("CodeBlock segments must match code exactly.");
    return segments ?? [{ text: code, role: "text" as const }];
  });
  async function copy() {
    const current = ++attempt;
    feedback = "";
    try {
      await navigator.clipboard.writeText(code);
      if (current === attempt) feedback = copiedMessage;
    } catch {
      if (current === attempt) feedback = copyFailedMessage;
    }
  }
</script>

<section class="fjord-code-block" aria-label={label}>
  <div class="fjord-code-block-header">
    <span>{label}</span><button type="button" onclick={copy}>{copyLabel}</button
    >
  </div>
  <span class="fjord-code-feedback" role="status">{feedback}</span>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex: a named scroll region must be keyboard-scrollable -->
  <div class="fjord-code-scroll" role="region" aria-label={label} tabindex="0">
    <pre class="fjord-type-code fjord-code"><code
        >{#each checked as part}<span data-syntax={part.role}>{part.text}</span
          >{/each}</code
      ></pre>
  </div>
</section>
