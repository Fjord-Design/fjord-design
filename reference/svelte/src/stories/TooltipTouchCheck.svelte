<script lang="ts">
  import { Tooltip, ThemeScope } from "../lib/index.js";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  import "../lib/overlay-components.css";
  let trigger: HTMLButtonElement | undefined = $state();
  function touch(target: Element | undefined) {
    target?.dispatchEvent(
      new PointerEvent("pointerdown", { bubbles: true, pointerType: "touch" }),
    );
  }
</script>

<ThemeScope
  ><main class="fjord-foundation fjord-semantic fjord-type-body">
    <div class="fjord-page fjord-stack">
      <h1>Tooltip touch event check</h1>
      <p>
        This fixture dispatches touch pointer events. Press T on the trigger to
        test while it retains focus.
      </p>
      <div>
        <Tooltip id="touch-tip" text="Supplementary description"
          >{#snippet children(attributes)}<button
              {...attributes}
              bind:this={trigger}
              onkeydown={(event) => {
                if (event.key === "t") touch(trigger);
              }}>Example trigger</button
            >{/snippet}</Tooltip
        >
      </div>
      <div>
        <button onclick={() => touch(trigger)}>Simulate trigger touch</button>
        <button onclick={() => touch(document.body)}
          >Simulate outside touch</button
        >
      </div>
    </div>
  </main></ThemeScope
>
