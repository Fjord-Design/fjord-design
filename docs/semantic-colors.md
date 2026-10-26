# Semantic colours and states

Run `mise run semantics:build` and open `dist/semantics.html`. The preview covers Night, Day, Aurora and an explicit System preference. Fjord is the default identity; Fern demonstrates a semantic override with the same component bindings.

## Meaning and ownership

`tokens/usage.json` maps 34 UI meanings and eight states to existing semantic/component paths. It owns descriptions, applicable backgrounds and contrast minima. Palette values remain in DTCG sources. The generated role table shows exact Night/Day/Aurora HEX values; `dist/semantics.json` includes both identities, all 81 semantic colour values per mode and the complete allowed-pair evidence.

| Meaning | Existing role | Use |
| --- | --- | --- |
| Canvas | `bg-base` | Page, window, editor |
| Surface | `bg-surface` | Panels, inputs and content |
| Floating surface | `bg-raised` | Menus and dialogs |
| Sunken surface | `bg-sunken` | Code and terminal wells |
| Primary / secondary / help text | `ink` / `ink-muted` / `ink-muted` | Reading text and instructions |
| Short metadata | `ink-subtle` | Timestamps and placeholders with a persistent label |
| Disabled labels | `ink-disabled` | Unavailable controls only |
| Decorative / control boundary | `line` / `line-strong` | Separators / identifiable controls |
| Focus | `focus-ring` | Offset keyboard outline |
| Links | `accent` / `accent-hover` | Underlined default/hover text |
| Primary fill and label | `accent` / `on-accent` | Primary action |
| Selected item | `accent-soft` / `accent-ink` | Persistent choice, with outline or marker |
| Text selection | `selection` / `ink` | Selected reading text |
| Overlay | `scrim` + `bg-raised` / `ink` | Translucent separator behind an opaque reading surface |
| Status | `info`, `success`, `warning`, `danger` and their `*-soft` fills | Visible status words and optional icons |

These short names belong to `semantic.color`. Component bindings keep their full `component.*` paths and reference semantic roles directly. Avoid literal colours in component code. Decorative dividers cannot be the only control boundary. Scrims and shadows may be translucent; content surfaces remain opaque.

## State contract

| State | Visual binding | Behaviour and applicability |
| --- | --- | --- |
| Default | Component foreground/background | Available action with a name |
| Hover | Component hover background; link hover role | Pointer feedback; retain link underline |
| Pressed | Component pressed fill and inset shadow | Action while held; shared hover colour may be distinguished with the shadow |
| Focus-visible | Existing state plus offset focus outline | Keyboard location; preserve focus during state/theme changes |
| Selected | `accent-soft` + `accent-ink`, outline/marker | Persistent selection with native selected/pressed semantics |
| Disabled | `ink-disabled` on a neutral surface | Unavailable control; suppress activation; exclude from normal tab order where native disabled applies |
| Invalid | `danger` boundary and associated error text | Validated fields/data; use `aria-invalid` and an error association; retain the focus outline |
| Loading | Existing action foreground/background | Keep the label readable and focus stable; expose busy state and suppress repeated activation |

Apply a state only where it makes sense. Selection describes a persistent choice; pressed describes an action being held. Loading is pending work, so its label keeps ordinary contrast. An unavailable native control uses `disabled`; a pending action can use `aria-disabled` while keeping keyboard focus, with activation explicitly guarded. The preview demonstrates this distinction and performs no data persistence.

Theme foregrounds and backgrounds change together without colour interpolation. This avoids temporary low contrast between a new text colour and an old fill. Controls may transition their shadow for press feedback; semantic colour changes are immediate.

Disabled foreground is never help text, a timestamp, a loading label or ordinary content. The validator rejects its direct use in active roles and aliases that reach it indirectly. It also rejects lower contrast thresholds or reclassifying a text role to bypass checks. Disabled controls retain an accessible name even though their inactive colours have a contrast exemption.

## Allowed pairs and contrast

The authoritative pair list is the union of `tokens/usage.json`, code/terminal policy and component bindings. The validator deduplicates pairs and retains the strongest minimum. Each identity has 143 pairs in Night, 144 in Day and 143 in Aurora: 430 per identity. Both source sRGB and final HEX components must pass without rounding the comparison.

| Pair | Minimum |
| --- | --- |
| Ordinary/help/link/active-state text on its listed backgrounds | 4.5:1 |
| Status text on its soft fill, canvas, panel or raised surface | 4.5:1 |
| Selected and text-selection foregrounds on their fills | 4.5:1 |
| Focus and specified control borders on their listed surfaces | 3:1 |
| Code comments on base | 5:1; Night increases to 7.5:1 |
| Disabled labels and decorative dividers | Explicit limited-use exemption |

An allowed foreground on one surface does not approve it on every surface. `ink-subtle`, for example, is listed on base/surface/raised, not all fills. Alpha is rejected in checked pairs; a scrim is not treated as a flat reading background. Open the preview's contrast disclosure or inspect the JSON report for exact permitted combinations and measured ratios.

## Mode preference and scoped identity

The product default is **Night**, even when the OS is light. Day and Aurora are explicit choices. System is an explicit preference that maps light to Day and dark to Night. Aurora is never selected automatically. Explicit choices take precedence over OS changes.

```svelte
<script lang="ts">
  import { ThemeScope, type ModePreference } from '@fjord/svelte';
  import '@fjord/svelte/typography.css';
  import '@fjord/svelte/foundations.css';
  import '@fjord/svelte/semantics.css';
  let preference: ModePreference = 'night';
</script>

<ThemeScope mode={preference} identity="fjord">
  <main class="fjord-foundation fjord-semantic fjord-type-body">
    <p>Theme-aware content</p>
  </main>
</ThemeScope>
```

Set `mode="system"` to follow the OS. `ThemeScope` exposes `data-fjord-mode`, `data-fjord-preference` and `data-fjord-identity` on its own wrapper. Nested scopes remain independent. The helper observes system changes only while System is active and removes its listener when the preference changes or the component unmounts. It does not change document globals, OS settings or stored preferences. Persistence is an application responsibility.

Server rendering uses a stable Night attribute until client resolution. For an explicit System preference, scoped [`prefers-color-scheme`](https://www.w3.org/TR/mediaqueries-5/#prefers-color-scheme) CSS applies Day immediately in light environments, including before hydration or without JavaScript. The same mapping covers native [`color-scheme`](https://www.w3.org/TR/css-color-adjust-1/#color-scheme-prop) and shadow values. This rendering default is separate from token completeness: missing mode values are always rejected and never substituted from Night.

## Identity override proof

`examples/themes/fern.tokens.json` overrides five existing semantic roles: accent, accent hover, selected fill, selected foreground and text selection. They alias approved green/status palette roles. Primary labels and focus continue to follow their existing semantic bindings. Component paths, types and aliases remain identical in all six identity/mode combinations; the semantic build verifies this invariant and all contrast pairs.

Use `identity="fern"` with the generated semantic stylesheet to inspect it. Identity is a string, so applications can supply their own validated CSS identity without changing component props. Adding an identity does not require copying a component implementation. Production theme export and the full authoring guide remain XER-116 and XER-118 work.

```sh
mise exec -- python3 -m fjord_tokens build --mode day \
  --theme examples/themes/fern.tokens.json --output dist/day-fern.tokens.json
```

## Verification and delivery

`mise run check` includes usage/state validation, both identity builds, system-listener tests and actual Svelte server rendering. `mise run ci` verifies distributed type declarations, including the System preference, and byte-identical report/CSS output. The package exports `@fjord/svelte/semantics.css`. The static preview shares the same mode-observer source as Svelte.

[XER-107 verification](verification/xer-107.md) records browser and automated results. The complete release browser and assistive-technology matrix remains XER-119 work. This delivery defines state contracts and reference samples; the 23 MVP product components retain their implementation tasks.
