# Carousel, Resizable, Scroll Area and Direction

Carousel rotates manually with named native buttons, bounded previous/next states and a concise slide status. Hidden slides retain local content. Resizable is a keyboard/pointer window splitter with labelled separator, min/max/current percentage and Home/End/arrow controls. ScrollArea is a named keyboard-focusable region. Direction applies native dir and relies on logical spacing rather than changing source order.

## API and ownership

Carousel: label, slides, bind:index, children(index). Resizable: label, bind:value, min/max, first/second snippets. ScrollArea: label, blockSize, children. Direction: direction (ltr/rtl), children.

## Tokens and states

Colours use shared `ink`, opaque `bg-base`/`bg-surface`/`bg-raised`, `accent`/`accent-soft`, `line-strong`, `focus-ring` and the documented status roles. Spacing, radii and typography use the existing foundation tokens. Focus is visible; colour is supplemented by native states, text or shape. Night, Day, Aurora and Fern examples are in the Complete Library catalogue.

## Limits and verification

No automatic carousel rotation or motion is enabled. Resize controls reverse logical arrows in RTL. Contents must wrap or provide their own named scroll regions. Direction does not translate text or infer language; application locale and reading order remain explicit.

Tracking: XER-131. See [M3/M4 verification](../verification/m3-m4.md) for executed checks and remaining native/screen-reader limits. Component source is in `reference/svelte/src/lib/`; public props are the generated TypeScript declarations.
