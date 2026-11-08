# Alert Dialog, Drawer, Sheet, Popover and Hover Card

Panels use native dialog with explicit modal/nonmodal behavior, named headings, Escape and focus return. AlertDialog initially focuses Cancel and exposes an explicit response. Sheet is controlled from an edge. Drawer adds a pointer handle that closes toward its edge; the visible close buttons remain equivalent alternatives. Native popovers use the top layer, dismiss on Escape/outside interaction and clamp to viewport edges. HoverCard adds hover/focus opening while retaining a click/touch trigger.

## API and ownership

AlertDialog: id, title, description, bind:open, children, actions, cancelLabel. Drawer/Sheet: id, title, bind:open, edge, modal, children. Popover/HoverCard: id, label, children; Popover also bind:open.

## Tokens and states

Colours use shared `ink`, opaque `bg-base`/`bg-surface`/`bg-raised`, `accent`/`accent-soft`, `line-strong`, `focus-ring` and the documented status roles. Spacing, radii and typography use the existing foundation tokens. Focus is visible; colour is supplemented by native states, text or shape. Night, Day, Aurora and Fern examples are in the Complete Library catalogue.

## Limits and verification

Use unique IDs and avoid moving modal triggers while open. A nested panel returns focus to the underlying panel. No motion is required. Native dialog/popover support is required. Content reflows and scrolls inside the viewport; positioning is updated for scroll/resize.

Tracking: XER-123. See [M3/M4 verification](../verification/m3-m4.md) for executed checks and remaining native/screen-reader limits. Component source is in `reference/svelte/src/lib/`; public props are the generated TypeScript declarations.
