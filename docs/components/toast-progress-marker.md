# Toast, Progress and Marker

Toast uses an existing live region with polite or assertive priority, a named dismiss action and optional actions. Notifications never time out automatically, giving users time to read and act. Progress names known/unknown progress and validates bounds. Marker anchors a compact textual indicator to its target; its dot is decorative.

## API and ownership

Toast: title, message, priority, bind:open, children, ondismiss. Progress: id, label, value?, max. Marker: label, children.

## Tokens and states

Colours use shared `ink`, opaque `bg-base`/`bg-surface`/`bg-raised`, `accent`/`accent-soft`, `line-strong`, `focus-ring` and the documented status roles. Spacing, radii and typography use the existing foundation tokens. Focus is visible; colour is supplemented by native states, text or shape. Night, Day, Aurora and Fern examples are in the Complete Library catalogue.

## Limits and verification

Mount the Toast live region before changing open/message for reliable announcement. The application chooses urgency and manages queues/persistence. Dismissal restores prior connected focus where possible. Progress is not a process runner. Reduced motion uses static unknown progress.

Tracking: XER-128. See [M3/M4 verification](../verification/m3-m4.md) for executed checks and remaining native/screen-reader limits. Component source is in `reference/svelte/src/lib/`; public props are the generated TypeScript declarations.
