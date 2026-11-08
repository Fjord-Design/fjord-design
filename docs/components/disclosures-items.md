# Accordion, Collapsible, Aspect Ratio and Item

Accordion uses heading buttons and labelled persistent panels; Enter/Space toggle, arrows/Home/End move among enabled headings. Multiple expansion is opt-in. Collapsible uses native details/summary. AspectRatio reserves a positive finite ratio before media loads and provides an empty fallback. Item offers a noninteractive row, link or button; interactive rows reject nested action snippets.

## API and ownership

Accordion: id, items, bind:value:string[], multiple, children(value). Collapsible: label, bind:open, children. AspectRatio: ratio, label, children. Item: title, description, metadata, href or onclick, icon, actions.

## Tokens and states

Colours use shared `ink`, opaque `bg-base`/`bg-surface`/`bg-raised`, `accent`/`accent-soft`, `line-strong`, `focus-ring` and the documented status roles. Spacing, radii and typography use the existing foundation tokens. Focus is visible; colour is supplemented by native states, text or shape. Night, Day, Aurora and Fern examples are in the Complete Library catalogue.

## Limits and verification

Long headings and metadata wrap. Disabled accordion items remain unavailable. Native details preserves its content. AspectRatio bounds media; the application supplies meaningful image/video alternatives.

Tracking: XER-121. See [M3/M4 verification](../verification/m3-m4.md) for executed checks and remaining native/screen-reader limits. Component source is in `reference/svelte/src/lib/`; public props are the generated TypeScript declarations.
