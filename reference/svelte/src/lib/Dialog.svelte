<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    id,
    title,
    description,
    closeLabel,
    open = $bindable(false),
    children,
  }: {
    id: string;
    title: string;
    description?: string;
    closeLabel: string;
    open?: boolean;
    children: Snippet;
  } = $props();
  let dialog: HTMLDialogElement | undefined = $state();
  const checkedId = $derived.by(() => {
    if (!id || /\s/.test(id) || !title?.trim() || !closeLabel?.trim())
      throw new Error("Dialog requires an ID, title and close label.");
    return id;
  });
  $effect(() => {
    if (!dialog) return;
    if (open && !dialog.open) dialog.showModal();
    else if (!open && dialog.open) dialog.close();
  });
  function closed() {
    if (!dialog?.open) open = false;
  }
  function cycle(event: KeyboardEvent) {
    if (event.key !== "Tab" || !dialog?.open) return;
    if (
      event.target instanceof Element &&
      event.target.closest("dialog") !== dialog
    )
      return;
    const candidates = Array.from(
      dialog.querySelectorAll<HTMLElement>(
        'a[href], area[href], button, input, select, textarea, summary, [tabindex], [contenteditable="true"]',
      ),
    ).filter(
      (element) =>
        element.tabIndex >= 0 &&
        !element.matches(":disabled") &&
        !element.closest("[inert]") &&
        element.getClientRects().length > 0 &&
        getComputedStyle(element).visibility !== "hidden",
    );
    const tabbable = candidates.filter((element) => {
      if (
        !(element instanceof HTMLInputElement) ||
        element.type !== "radio" ||
        !element.name
      )
        return true;
      const group = candidates.filter(
        (candidate) =>
          candidate instanceof HTMLInputElement &&
          candidate.type === "radio" &&
          candidate.name === element.name &&
          candidate.form === element.form,
      ) as HTMLInputElement[];
      return (
        element === (group.find((candidate) => candidate.checked) ?? group[0])
      );
    });
    const ordered = [
      ...tabbable
        .filter((element) => element.tabIndex > 0)
        .sort((a, b) => a.tabIndex - b.tabIndex),
      ...tabbable.filter((element) => element.tabIndex === 0),
    ];
    const first = ordered[0];
    const last = ordered.at(-1);
    if (!first || !last) return;
    const current = document.activeElement;
    if (
      event.shiftKey &&
      (current === first || !ordered.includes(current as HTMLElement))
    ) {
      event.preventDefault();
      last.focus();
    } else if (
      !event.shiftKey &&
      (current === last || !ordered.includes(current as HTMLElement))
    ) {
      event.preventDefault();
      first.focus();
    }
  }
</script>

<dialog
  bind:this={dialog}
  class="fjord-dialog"
  aria-modal="true"
  aria-labelledby={`${checkedId}-title`}
  aria-describedby={description?.trim()
    ? `${checkedId}-description`
    : undefined}
  onclose={closed}
  oncancel={() => (open = false)}
  onkeydown={cycle}
>
  <div class="fjord-dialog-head">
    <h2 class="fjord-type-title-3" id={`${checkedId}-title`}>{title}</h2>
    <button
      type="button"
      class="fjord-dialog-close"
      onclick={() => {
        open = false;
        dialog?.close();
      }}>{closeLabel}</button
    >
  </div>
  {#if description?.trim()}<p
      id={`${checkedId}-description`}
      class="fjord-dialog-description"
    >
      {description}
    </p>{/if}
  <div class="fjord-dialog-body">{@render children()}</div>
</dialog>
