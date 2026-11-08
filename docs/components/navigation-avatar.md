# Avatar, Breadcrumb, Navigation Menu and Sidebar

Avatar uses a meaningful image name or an explicitly decorative image, then initials on missing/error sources. A changed source is retried. Breadcrumb and NavigationMenu use navigation landmarks and links, never application-menu roles. Current destinations expose aria-current. Sidebar has expanded and compact labels, plus a separate mobile disclosure; Escape closes the mobile navigation and returns focus to its toggle.

## API and ownership

Avatar: src, label, fallback, decorative. Breadcrumb/NavigationMenu: label, NavigationItem[]. Sidebar: id, label, items, bind:compact.

## Tokens and states

Colours use shared `ink`, opaque `bg-base`/`bg-surface`/`bg-raised`, `accent`/`accent-soft`, `line-strong`, `focus-ring` and the documented status roles. Spacing, radii and typography use the existing foundation tokens. Focus is visible; colour is supplemented by native states, text or shape. Night, Day, Aurora and Fern examples are in the Complete Library catalogue.

## Limits and verification

Route matching, authorization and navigation remain application work. Disabled destinations prevent activation. Compact links retain full accessible labels and tooltips. Mobile navigation is an inline disclosure, not a modal.

Tracking: XER-122. See [M3/M4 verification](../verification/m3-m4.md) for executed checks and remaining native/screen-reader limits. Component source is in `reference/svelte/src/lib/`; public props are the generated TypeScript declarations.
