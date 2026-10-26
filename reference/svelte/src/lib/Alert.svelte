<script lang="ts">
  let {
    title,
    message,
    variant = "info",
    announcement = "off",
  }: {
    title: string;
    message: string;
    variant?: "info" | "success" | "warning" | "danger";
    announcement?: "off" | "polite" | "assertive";
  } = $props();
  const checkedTitle = $derived.by(() => {
    if (!title?.trim() || !message?.trim())
      throw new Error("Alert requires a title and message.");
    return title;
  });
</script>

<div
  class="fjord-alert"
  data-variant={variant}
  role={announcement === "assertive"
    ? "alert"
    : announcement === "polite"
      ? "status"
      : undefined}
>
  <strong>{checkedTitle}</strong>
  <p>{message}</p>
</div>
