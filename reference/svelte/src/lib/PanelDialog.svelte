<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    id,
    title,
    description,
    open = $bindable(false),
    modal = true,
    edge = "center",
    kind = "dialog",
    gesture = false,
    closeLabel = "Close",
    children,
    actions,
  }: {
    id: string;
    title: string;
    description?: string;
    open?: boolean;
    modal?: boolean;
    edge?: "center" | "start" | "end" | "bottom";
    kind?: "dialog" | "alertdialog";
    gesture?: boolean;
    closeLabel?: string;
    children: Snippet;
    actions?: Snippet;
  } = $props();
  let dialog: HTMLDialogElement;
  let previous: HTMLElement | null = null;
  let start = 0;
  $effect(() => {
    if (!dialog) return;
    if (open && !dialog.open) {
      previous = document.activeElement as HTMLElement;
      if (modal) dialog.showModal();
      else dialog.show();
      dialog.querySelector<HTMLButtonElement>("[data-close]")?.focus();
    } else if (!open && dialog.open) dialog.close();
  });
  function closed() {
    if (dialog?.open) return;
    open = false;
    if (previous?.isConnected) previous.focus();
  }
  function key(e: KeyboardEvent) {
    if (e.key === "Escape" && !modal) {
      e.preventDefault();
      open = false;
    }
    if (
      e.key !== "Tab" ||
      !modal ||
      !(e.target instanceof Element) ||
      e.target.closest("dialog") !== dialog
    )
      return;
    const all = Array.from(
      dialog.querySelectorAll<HTMLElement>(
        "button,a[href],input,select,textarea,[tabindex]",
      ),
    ).filter(
      (el) =>
        el.tabIndex >= 0 &&
        !el.matches(":disabled") &&
        el.getClientRects().length > 0 &&
        !el.closest("[inert]"),
    );
    if (!all.length) return;
    const first = all[0],
      last = all[all.length - 1];
    if (
      e.shiftKey &&
      (document.activeElement === first ||
        !all.includes(document.activeElement as HTMLElement))
    ) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }
  function drag(e: PointerEvent) {
    const distance =
      edge === "bottom"
        ? e.clientY - start
        : edge === "start"
          ? start - e.clientX
          : e.clientX - start;
    if (distance > 80) open = false;
  }
</script>

<dialog
  bind:this={dialog}
  class="fjord-widget fjord-panel-dialog"
  data-edge={edge}
  role={kind}
  aria-modal={modal || undefined}
  aria-labelledby={`${id}-title`}
  aria-describedby={description ? `${id}-description` : undefined}
  onclose={closed}
  oncancel={() => (open = false)}
  onkeydown={key}
>
  <header>
    <h2 id={`${id}-title`}>{title}</h2>
    <button data-close type="button" onclick={() => (open = false)}
      >{closeLabel}</button
    >
  </header>
  {#if description}<p id={`${id}-description`}>{description}</p>{/if}
  {#if gesture}<button
      type="button"
      class="fjord-drag-handle"
      aria-label="Close panel; drag toward the edge or activate"
      onclick={() => (open = false)}
      onpointerdown={(e) => {
        start = edge === "bottom" ? e.clientY : e.clientX;
        e.currentTarget.setPointerCapture(e.pointerId);
      }}
      onpointerup={drag}>Drag to close <span aria-hidden="true">⋮</span></button
    >{/if}
  <div>{@render children()}</div>
  {#if actions}<footer>{@render actions()}</footer>{/if}
</dialog>
