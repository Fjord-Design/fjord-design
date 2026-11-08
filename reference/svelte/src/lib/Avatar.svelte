<script lang="ts">
  let {
    src,
    label,
    fallback,
    decorative = false,
  }: {
    src?: string;
    label: string;
    fallback?: string;
    decorative?: boolean;
  } = $props();
  let failedSrc = $state<string>();
</script>

<span class="fjord-avatar" aria-hidden={decorative || undefined}
  >{#if src && failedSrc !== src}<img
      {src}
      alt={decorative ? "" : label}
      onerror={() => (failedSrc = src)}
    />{:else}<span
      role={decorative ? undefined : "img"}
      aria-label={decorative ? undefined : label}
      >{fallback ??
        label
          .split(/\s+/)
          .map((p) => p[0])
          .slice(0, 2)
          .join("")}</span
    >{/if}</span
>
