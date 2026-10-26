<script lang="ts">
  import { ThemeScope, type Mode } from "../lib/index.js";

  let { mode = "night" }: { mode?: Mode } = $props();
</script>

<ThemeScope {mode}>
  <main>
    <h1 class="fjord-type-title-1">Fjord reference</h1>
    <p class="fjord-type-body">
      Svelte and TypeScript are ready. Selected mode: {mode}.
    </p>
    <p class="fjord-type-body">
      Component styling and theme controls are delivered with the component
      catalogue.
    </p>
  </main>
</ThemeScope>
