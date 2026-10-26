# Badge

A Badge is a static, non-interactive status label. Its anatomy is a required non-empty localized label and an optional decorative icon. Use words that communicate the status without depending on colour. A badge is not a button and does not announce changes automatically; applications own any necessary live announcement.

| Variant | Foreground | Background |
| --- | --- | --- |
| Neutral | `ink` | `bg-surface` |
| Info | `info` | `info-soft` |
| Success | `success` | `success-soft` |
| Warning | `warning` | `warning-soft` |
| Danger | `danger` | `danger-soft` |

Status foreground/background pairs are validated at 4.5:1 in Night, Day and Aurora. Typography follows `component.button.primary.font-family` and `font-size`; spacing and shape use `space-1`, `space-2` and `radius-control`. Long labels wrap within available space. Forced-colour mode retains a visible border and text.

```svelte
<Badge label="Ready" variant="success" icon="success" />
```

Import `Badge` from `@fjord/svelte` and the shared styles listed in the [Svelte reference](../../reference/svelte/README.md). Use a Button for an action and a separate live region when a status change needs an announcement.
