<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Mode, ModePreference } from "./types.js";
  import { watchMode } from "./theme-preference.js";

  let {
    mode = "night",
    identity = "fjord",
    children,
  }: {
    mode?: ModePreference;
    identity?: string;
    children?: Snippet;
  } = $props();
  let systemMode = $state<Mode>("night");
  const resolvedMode = $derived(mode === "system" ? systemMode : mode);
  $effect(() =>
    watchMode(mode, (value) => {
      systemMode = value;
    }),
  );
</script>

<div
  data-fjord-mode={resolvedMode}
  data-fjord-preference={mode}
  data-fjord-identity={identity}
>
  {@render children?.()}
</div>
