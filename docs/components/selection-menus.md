# Select, Combobox and action menus

Select is a select-only combobox; Combobox filters a discrete set while DOM focus remains on its input. Arrow keys, Home/End, Enter, Escape and typeahead operate the popup. Options skip disabled choices; empty/loading/error states are explicit. NativeSelect remains the preferred simple native form control. DropdownMenu and ContextMenu provide keyboard-openable triggers, nested actions and check/radio states. Menubar uses roving focus and horizontal arrows.

## API and ownership

Select/Combobox: id, label, options, bind:value, name, required, disabled, loading, error. DropdownMenu/ContextMenu: id, label, MenuItem[], onselect. Menubar: id, label, menus, onselect.

## Tokens and states

Colours use shared `ink`, opaque `bg-base`/`bg-surface`/`bg-raised`, `accent`/`accent-soft`, `line-strong`, `focus-ring` and the documented status roles. Spacing, radii and typography use the existing foundation tokens. Focus is visible; colour is supplemented by native states, text or shape. Night, Day, Aurora and Fern examples are in the Complete Library catalogue.

## Limits and verification

The application updates checked/radio states after onselect. Menu shortcuts are labels; the application registers actions. Context actions also open from a normal button. Large lists scroll; virtualization is not provided. IDs and option values must be stable. Required Select uses a visually hidden native select for form constraint validation; an invalid submission focuses the visible trigger and displays a message. Editing Combobox clears its previous submitted value until an option is explicitly chosen. NativeSelect remains the simpler native option.

Tracking: XER-124. See [M3/M4 verification](../verification/m3-m4.md) for executed checks and remaining native/screen-reader limits. Component source is in `reference/svelte/src/lib/`; public props are the generated TypeScript declarations.
