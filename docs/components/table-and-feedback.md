# Table and feedback

Import `@fjord/svelte/data-feedback.css` after the typography, foundation and semantic styles. Components/Table Empty Skeleton Spinner in Storybook demonstrates data, loading, empty and error states.

## Table

Table supplies a native table and visible caption inside a named, keyboard-focusable scroll region. Give it a unique `id`, a non-empty caption and semantic table children. Use `thead`, `tbody`, `tr`, `th` and `td`; set `scope="col"` or `scope="row"` on simple headings. Complex tables may need explicit header associations. Do not use a table solely for layout.

The table has a minimum width and scrolls within its own region on narrow screens. Long unbroken cell values can wrap. Focus on the region to scroll with the keyboard. The application supplies the actual data and determines whether empty, loading or error content replaces the table.

## Empty

Empty shows a decorative icon, a required title and description, and optional application-supplied action. Explain why no content is present and offer an action when useful. Empty is static content and does not announce itself automatically.

## Skeleton and Spinner

Skeleton is a decorative, static placeholder hidden from assistive technology. It has line and block shapes and short, medium or full widths. It never substitutes for a loading label. Spinner includes a visible non-empty label and a polite status role; its mark rotates slowly and becomes static with reduced motion. Use a persistent status region or appropriate application announcement when a change must be announced reliably, and avoid nesting competing live regions.

Table and Empty use `bg-surface`, `ink`, `ink-muted`, `line`, `space-3`, `space-5` and `radius-panel`. Skeleton uses `bg-active` and `radius-control`. Spinner uses `line-strong`, `accent`, `icon-status`, `border-active` and `duration-slow`. Focus uses `focus-ring`, `border-focus` and `focus-offset`. Forced-colour mode uses system colours and visible boundaries.

The example error uses Alert. Applications own retry behaviour, status changes and any announcement needed when data arrives or fails. The complete browser and assistive technology release matrix remains in XER-119.
