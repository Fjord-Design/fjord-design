<script lang="ts">
  import type { TraceStep } from "./complete-types.js";
  let { label, steps }: { label: string; steps: TraceStep[] } = $props();
  const checked = $derived.by(() => {
    if (
      new Set(steps.map((s) => s.id)).size !== steps.length ||
      steps.some(
        (s) =>
          s.durationMs !== undefined &&
          (!Number.isFinite(s.durationMs) || s.durationMs < 0),
      )
    )
      throw new Error("Trace needs unique steps and non-negative timing.");
    return steps;
  });
</script>

<section class="fjord-widget fjord-trace" aria-label={label}>
  <p role="status">
    {steps.filter((s) => s.status === "complete").length} of {steps.length} steps
    complete
  </p>
  <ol>
    {#each checked as step (step.id)}<li data-status={step.status}>
        <details>
          <summary
            >{step.label} — {step.status}{step.durationMs !== undefined
              ? ` · ${step.durationMs} ms`
              : ""}</summary
          >{#if step.startedAt}<p>
              Started <time datetime={step.startedAt}>{step.startedAt}</time>
            </p>{/if}{#if step.error}<p class="fjord-trace-error">
              Error: {step.error}
            </p>{/if}{#if step.detail}<pre>{step.detail}</pre>{:else}<p>
              No additional details
            </p>{/if}
        </details>
      </li>{/each}
  </ol>
</section>
