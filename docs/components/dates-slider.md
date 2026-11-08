# Calendar, Date Picker and Slider

Calendar supports a locale, week start, bounds, disabled-date predicate, single or range selection. Dates are YYYY-MM-DD calendar values; UTC arithmetic is used only to calculate calendar cells, avoiding local timezone conversion. Arrows move days/weeks, Home/End move within a week and PageUp/Down move months (Shift moves years). DatePicker keeps manual native date input alongside the calendar. Slider combines a native range with an optional exact numeric input.

## API and ownership

Calendar: label, bind:value, bind:endValue, range, locale, weekStartsOn, min, max, disabledDate, onchange. DatePicker: id, label, bind:value, name, min/max, locale/weekStartsOn, required/disabled/error. Slider: id, label, bind:value, min, max, step, precise.

## Tokens and states

Colours use shared `ink`, opaque `bg-base`/`bg-surface`/`bg-raised`, `accent`/`accent-soft`, `line-strong`, `focus-ring` and the documented status roles. Spacing, radii and typography use the existing foundation tokens. Focus is visible; colour is supplemented by native states, text or shape. Night, Day, Aurora and Fern examples are in the Complete Library catalogue.

## Limits and verification

Leap years, invalid dates and month boundaries are checked. Ranges restart after a completed pair. Native manual date editing follows browser/OS presentation. Initial and external Slider values are clamped and snapped to the nearest valid step from `min`, with ties toward the larger value. A partial final step cannot exceed `max`. The bound value, range, output and exact field agree after normalization. Range edits update immediately; exact numeric edits normalize when committed with Enter or when leaving the field, so partial typing is preserved. Unit formatting and business validation remain application work. See the [control regression checks](../verification/control-regressions.md).

Tracking: XER-126. See [M3/M4 verification](../verification/m3-m4.md) for executed checks and remaining native/screen-reader limits. Component source is in `reference/svelte/src/lib/`; public props are the generated TypeScript declarations.
