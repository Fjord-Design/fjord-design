# Grouped controls, OTP and toggles

ButtonGroup and InputGroup name related controls without changing their tab order. Input addons are adjacent text; controls retain individual names. InputOTP deliberately uses one text input with numeric inputmode and one-time-code autofill, so paste, backspace and mobile entry need no segmented navigation. Toggle exposes aria-pressed; ToggleGroup uses individually tabbable pressed buttons, with single or multiple selection.

## API and ownership

ButtonGroup/InputGroup: label, children; InputGroup also prefix, suffix, error. InputOTP: id, label, name, bind:value, length, disabled, required, error. Toggle: label, bind:pressed, disabled. ToggleGroup: label, options, bind:value:string[], multiple.

## Tokens and states

Colours use shared `ink`, opaque `bg-base`/`bg-surface`/`bg-raised`, `accent`/`accent-soft`, `line-strong`, `focus-ring` and the documented status roles. Spacing, radii and typography use the existing foundation tokens. Focus is visible; colour is supplemented by native states, text or shape. Night, Day, Aurora and Fern examples are in the Complete Library catalogue.

## Limits and verification

OTP accepts ASCII digits and preserves leading zeroes; the server validates any real code. Length is 1–32. Group error descriptions do not replace the control-specific Field associations. Buttons remain native controls.

Tracking: XER-125. See [M3/M4 verification](../verification/m3-m4.md) for executed checks and remaining native/screen-reader limits. Component source is in `reference/svelte/src/lib/`; public props are the generated TypeScript declarations.
