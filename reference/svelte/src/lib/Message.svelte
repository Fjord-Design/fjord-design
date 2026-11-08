<script lang="ts">
  import type { Snippet } from "svelte";
  import Bubble from "./Bubble.svelte";
  let {
    id,
    sender,
    timestamp,
    timeLabel,
    status,
    outgoing = false,
    children,
    attachments,
    actions,
  }: {
    id: string;
    sender: string;
    timestamp: string;
    timeLabel: string;
    status?: string;
    outgoing?: boolean;
    children: Snippet;
    attachments?: Snippet;
    actions?: Snippet;
  } = $props();
</script>

<article class="fjord-widget fjord-message" aria-labelledby={`${id}-sender`}>
  <header>
    <strong id={`${id}-sender`}>{sender}</strong><time datetime={timestamp}
      >{timeLabel}</time
    >
  </header>
  <Bubble {outgoing}>{@render children()}</Bubble
  >{#if attachments}{@render attachments()}{/if}{#if status}<p>
      {status}
    </p>{/if}{#if actions}<footer>{@render actions()}</footer>{/if}
</article>
