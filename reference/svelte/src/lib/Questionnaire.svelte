<script lang="ts">
  import { tick } from "svelte";
  import type { Snippet } from "svelte";
  import type { Question } from "./complete-types.js";
  import Progress from "./Progress.svelte";
  let {
    id,
    label,
    questions,
    answers = $bindable<Record<string, string>>({}),
    onsubmit,
    children,
  }: {
    id: string;
    label: string;
    questions: Question[];
    answers?: Record<string, string>;
    onsubmit: (answers: Record<string, string>) => void | Promise<void>;
    children: Snippet<
      [Question, string, (value: string) => void, string | undefined]
    >;
  } = $props();
  let step = $state(0),
    error = $state(""),
    busy = $state(false),
    complete = $state(false);
  let heading = $state<HTMLHeadingElement>();
  const current = $derived.by(() => {
    if (
      !questions.length ||
      new Set(questions.map((q) => q.id)).size !== questions.length
    )
      throw new Error("Questionnaire requires unique questions.");
    return questions[Math.min(step, questions.length - 1)];
  });
  async function next() {
    if (busy || complete) return;
    error = current.validate?.(answers[current.id] ?? "") ?? "";
    if (error) return;
    if (step < questions.length - 1) {
      step++;
      await tick();
      heading?.focus();
      return;
    }
    for (let i = 0; i < questions.length; i++) {
      const problem = questions[i].validate?.(answers[questions[i].id] ?? "");
      if (problem) {
        step = i;
        error = problem;
        await tick();
        heading?.focus();
        return;
      }
    }
    busy = true;
    try {
      await onsubmit({ ...answers });
      complete = true;
    } catch (cause) {
      error =
        cause instanceof Error
          ? cause.message
          : "Could not submit. Your answers are retained.";
    } finally {
      busy = false;
    }
  }
</script>

<section
  class="fjord-widget fjord-questionnaire"
  aria-label={label}
  aria-busy={busy}
>
  {#if complete}<p role="status">Responses submitted</p>{:else}<Progress
      id={`${id}-progress`}
      label={`${label}, question ${step + 1} of ${questions.length}`}
      value={step + 1}
      max={questions.length}
    />
    <form
      onsubmit={(e) => {
        e.preventDefault();
        void next();
      }}
    >
      <h3 tabindex="-1" bind:this={heading}>{current.title}</h3>
      {#if current.description}<p>{current.description}</p>{/if}
      <div aria-describedby={error ? `${id}-error` : undefined}>
        {@render children(
          current,
          answers[current.id] ?? "",
          (value) => (answers = { ...answers, [current.id]: value }),
          error || undefined,
        )}
      </div>
      {#if error}<p id={`${id}-error`} role="alert">{error}</p>{/if}
      <div class="fjord-button-group">
        <button
          type="button"
          disabled={step === 0 || busy}
          onclick={async () => {
            step--;
            error = "";
            await tick();
            heading?.focus();
          }}>Previous</button
        ><button type="submit" aria-disabled={busy || undefined}
          >{busy
            ? "Submitting…"
            : step === questions.length - 1
              ? "Submit"
              : "Next"}</button
        >
      </div>
    </form>{/if}
</section>
