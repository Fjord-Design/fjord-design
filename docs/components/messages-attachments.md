# Attachment and message history

Attachment presents filename, MIME type, byte size, progress and retryable errors. Bubble is a visual content surface. Message is an article with sender, timestamp, content, attachments, status and named actions. MessageScroller follows appended messages only when already at the bottom; otherwise it preserves the reading position and announces only the new count, with a jump-to-latest action.

## API and ownership

Attachment: id, filename, type, size, status, progress, href, onretry. Bubble: children, outgoing. Message: id, sender, timestamp, timeLabel, status, outgoing, children, attachments, actions. MessageScroller: label, count, children.

## Tokens and states

Colours use shared `ink`, opaque `bg-base`/`bg-surface`/`bg-raised`, `accent`/`accent-soft`, `line-strong`, `focus-ring` and the documented status roles. Spacing, radii and typography use the existing foundation tokens. Focus is visible; colour is supplemented by native states, text or shape. Night, Day, Aurora and Fern examples are in the Complete Library catalogue.

## Limits and verification

File access, upload, scanning, transport and storage belong to the application. Supply safe URLs and preformatted timestamps. History announcements do not reread the conversation. Prepending older history requires the application to retain an anchor; this API covers appended messages.

Tracking: XER-129. See [M3/M4 verification](../verification/m3-m4.md) for executed checks and remaining native/screen-reader limits. Component source is in `reference/svelte/src/lib/`; public props are the generated TypeScript declarations.
