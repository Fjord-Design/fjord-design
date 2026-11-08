<script lang="ts">
  import Progress from "./Progress.svelte";
  let {
    id,
    filename,
    type,
    size,
    status = "ready",
    progress,
    href,
    onretry,
  }: {
    id: string;
    filename: string;
    type: string;
    size: number;
    status?: "ready" | "uploading" | "error";
    progress?: number;
    href?: string;
    onretry?: () => void;
  } = $props();
  const bytes = $derived.by(() => {
    if (!Number.isFinite(size) || size < 0)
      throw new Error("Attachment size must be non-negative.");
    return size < 1024 ? `${size} B` : `${(size / 1024).toFixed(1)} KB`;
  });
</script>

<article class="fjord-widget fjord-attachment" aria-label={filename}>
  <strong>{filename}</strong><span>{type} · {bytes}</span
  >{#if status === "uploading"}<Progress
      id={`${id}-progress`}
      label={`Uploading ${filename}`}
      value={progress}
    />{:else if status === "error"}<p role="alert">Upload failed</p>
    {#if onretry}<button type="button" onclick={onretry}
        >Retry {filename}</button
      >{/if}{:else if href}<a {href} download={filename}>Download {filename}</a
    >{:else}<span>Ready</span>{/if}
</article>
