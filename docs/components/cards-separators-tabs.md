# Card, Separator and Tabs

Import `@fjord/svelte/layout-components.css` after the foundation and semantic styles. The Components/Card Separator Tabs story shows examples in every Fjord mode and the Fern identity.

## Card

Card groups related content on an opaque surface with restrained depth. It defaults to a `div`; choose `as="article"` for self-contained content. Its child content supplies its own headings and actions. A Card is not automatically clickable or keyboard interactive. Use a link or button inside it when an action is needed.

The surface uses `bg-surface`, `ink`, `space-5`, `radius-panel` and `shadow-raised`. Forced-colour mode replaces depth with a border.

## Separator

Separator defaults to a decorative horizontal line hidden from assistive technology. Set `decorative={false}` for a thematic break exposed as a native horizontal rule. A vertical separator can be decorative or semantic; the latter has a separator role and vertical orientation. The line uses `line` and `border-hairline`; it must not be the only boundary for a control or status.

## Tabs

Tabs pairs each tab with a labelled panel. Provide a unique `id`, a non-empty label for the list, unique non-empty item values and labels, and at least one enabled item. The child snippet receives the current panel's value. A supplied value that is missing or disabled falls back to the first enabled item; the bound value is synchronized in the client. Panels stay mounted while hidden, so their local state is retained. Make IDs derived from the supplied ID unique across the page.

```svelte
<Tabs id="details" label="Details" items={items} activation="manual" bind:value>
  {#snippet children(panel)}
    {#if panel === 'summary'}<p>Summary</p>{:else}<p>Other details</p>{/if}
  {/snippet}
</Tabs>
```

Automatic activation selects a tab when it receives focus and is appropriate for panels already available without noticeable delay. Manual activation uses Enter or Space. Arrow keys move focus, wrap and skip disabled tabs; Home and End move to the first and last enabled tabs. Horizontal lists use Left and Right; vertical lists use Up and Down. Tab leaves the list for the active panel. Panels are keyboard-focusable by default; set `panelTabbable={false}` when their first meaningful content is already focusable. See the [W3C tabs pattern](https://www.w3.org/WAI/ARIA/apg/patterns/tabs/).

Long horizontal lists scroll within their own container; focusing a tab brings it into view. Orientation is explicit so visual placement and keyboard behaviour stay aligned at all widths. Labels wrap within a maximum width. Selected tabs use `accent-soft`, `accent-ink` and `accent`; focus uses `focus-ring`, `border-focus` and `focus-offset`. Disabled text uses `ink-disabled`. Styles use system colours in forced-colour mode and do not animate tab changes.
