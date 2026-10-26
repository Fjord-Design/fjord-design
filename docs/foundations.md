# Layout foundations

Run `mise run foundations:build` and open `dist/foundations.html`. The specimen exercises shared proportions, icons and motion. Product components remain owned by their M2 and M3 tasks.

## Space and density

The existing 4px spacing scale is 4, 8, 12, 16, 20, 24, 32, 40, 48 and 64px. Use gaps and a single hairline to separate related content. Reading text stays near 70 characters per line; long labels and paths wrap. Tables and line-preserving code may scroll inside a labelled keyboard-accessible region.

| Semantic role | Value | Use |
| --- | --- | --- |
| `layout.page-gutter` | 16px | Narrow page padding |
| `layout.page-gutter-wide` | 32px | Page padding from 640px |
| `layout.content-max` | 1200px | Maximum outer content width |
| `breakpoint.breakpoint-sm` | 640px | Two columns where content fits |
| `breakpoint.breakpoint-md` | 960px | Three-column specimen |
| `breakpoint.breakpoint-lg` | 1280px | Wide application layouts, retaining the content limit |

Paths start with `semantic.`. Breakpoints describe available content space, not device categories. Start with one column, preserve DOM reading order and let a component remain in one column when its content needs the space.

| Density | Control minimum height | Gap | Panel padding |
| --- | --- | --- | --- |
| Comfortable, default | 40px | 16px | 20px |
| Compact | 34px | 12px | 12px |
| Coarse pointer, either density | At least 44px | Selected density | Selected density |

Set `data-fjord-density="compact"` on `.fjord-foundation`. Typography keeps its size and line height. Use minimum heights so enlarged or translated text can grow. Do not clip an action's only label. If an override reduces a general spacing role used by comfortable density, adjust its compact counterpart too; compact gap and padding must not exceed comfortable values.

## Shape and depth

| Token | Value / role |
| --- | --- |
| `radius-none` | 0; full windows and editor/terminal panes |
| `radius-window` | 2px; floating windows |
| `radius-panel` | 4px; content surfaces |
| `radius-control` | 6px; input surfaces |
| `radius-full` | Pill; buttons, switches and toggles |
| `border-hairline` | 1px; separators |
| `shadow-sm` | Control separation |
| `shadow-raised` | Raised content |
| `shadow-overlay` | Floating content |
| `shadow-pressed` | Inset wells |

Use opaque mode-specific `bg-*` surfaces. Shadow composites retain subtle top highlights and contact shadows. Transparency is limited to scrims and shadows. Do not add blur or transparency to panels, windows or terminals. Elevation supports hierarchy without framing every region.

## Icons

Use Phosphor Fill, pinned to `@phosphor-icons/core` 2.1.1. Its filled silhouettes suit Fjord's solid icon direction. Sixteen original SVGs and the original MIT notice are stored locally. The source viewBox is 256 units; it scales into the 24 CSS pixel reference box without changing paths. [Asset provenance](../assets/icons/README.md) records archive integrity and file hashes.

| Semantic size | CSS pixels | Use |
| --- | --- | --- |
| `control` | 16 | Beside control labels |
| `status` | 18 | Alerts, status and navigation |
| `medium` | 20 | Larger inline content |
| `default` | 24 | Reference box |
| `empty` | 28 | Empty states |

Preserve proportions. Flex alignment centres icons beside labels; inline icons use a `-.125em` baseline offset. Inspect the supplied sizes before adding per-icon adjustments. Use `currentColor`, with status colour accompanied by a visible word. Keep the icon's size separate from its clickable area.

The supporting `Icon` primitive is exported from `@fjord/svelte`. It supports the 16 manifest names and five semantic sizes. It supports product components without adding an entry to the 66-component inventory.

```svelte
<script lang="ts">
  import { Icon } from '@fjord/svelte';
  import '@fjord/svelte/typography.css';
  import '@fjord/svelte/foundations.css';
</script>

<button class="fjord-control"><Icon name="add" size="control" />Create theme</button>
<button class="fjord-control fjord-icon-button" aria-label="Close settings">
  <Icon name="close" size="control" />
</button>
<Icon name="info" decorative={false} label="Information" />
```

Wrap these examples in `.fjord-foundation.fjord-type-body`, with `data-fjord-mode="night"`, `day` or `aurora` on the container. Decorative icons are hidden from assistive technology and cannot receive focus. Text or an accessible name on the button describes its action. Meaningful standalone icons require `decorative={false}` and a non-empty localized label, exposed with `role="img"`. Do not repeat adjacent visible text on its icon. TypeScript rejects a meaningful icon without a label.

## Layers, focus and targets

| Layer | Value |
| --- | --- |
| `z-base` | 0 |
| `z-sticky` | 10 |
| `z-popover` | 50 |
| `z-modal` | 60 |
| `z-toast` | 70 |
| `z-tooltip` | 80 |

Values order siblings in a shared stacking context. They cannot escape ancestor contexts or outrank the browser's top layer. Native dialogs and popovers retain top-layer ordering. Put contextual content inside its active modal layer. A high z-index does not implement focus handling or modal semantics.

Use a 2px `border-focus` outline in `focus-ring`, offset by 2px `focus-offset`. Keep focus unobscured and preserve it when content changes. The specimen uses native controls; product dialogs retain their own focus contract.

Default controls exceed the 24 by 24 CSS pixel minimum. Coarse-pointer controls grow to 44 by 44px. Rounded shapes must retain enough effective area or spacing. Inline links follow the [WCAG target-size exceptions](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum/). The 44px coarse-pointer policy is Fjord's choice; it is not an AA requirement for every control.

## Motion and system preferences

| Token | Value | Use |
| --- | --- | --- |
| `duration-none` | 0ms | Immediate update |
| `duration-fast` | 120ms | Hover and press |
| `duration-base` | 180ms | Reveal and switch |
| `duration-slow` | 260ms | Larger transitions |
| `easing-standard` | cubic-bezier(.2, 0, 0, 1) | Default curve |
| `easing-exit` | cubic-bezier(.4, 0, 1, 1) | Exit curve |

Animate properties that explain a change, with transitions within 300ms. Avoid `transition: all`. Loading indicators may repeat while work is pending and must retain text. The specimen derives its rotation period from four slow durations. Other decorative looping, parallax and automatic progression are excluded.

Under [`prefers-reduced-motion: reduce`](https://www.w3.org/TR/mediaqueries-5/#prefers-reduced-motion), transitions, animations and smooth scrolling stop. Content still appears; a static loading icon keeps its status text.

Under [`forced-colors: active`](https://www.w3.org/TR/css-color-adjust-1/#forced-colors-mode), use Canvas/CanvasText, ButtonFace/ButtonText and Highlight/HighlightText. Preserve borders and focus when shadows disappear. Icons inherit system text colours; status words remain visible. Leave automatic colour adjustment enabled. Browser emulation can test this preference without changing OS settings.

## Validation and distribution

`mise run check` validates target minima, positive sizes, density ordering, increasing breakpoints, focus width, layer order and transition timings after each composition layer. Later overrides cannot hide an invalid earlier value. Icon validation checks original bytes/licence, static filled paths and exact agreement with generated typed icon data.

The Svelte archive exports `@fjord/svelte/foundations.css` and includes the SVG subset with its original MIT notice. Import typography separately. Framework-independent consumers can copy `dist/foundations.css` and `dist/icons/`. General token export remains XER-116 work. [XER-106 verification](verification/xer-106.md) records evidence; release-wide browser and assistive-technology testing remains XER-119 work.
