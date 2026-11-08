# Slider and selection regressions

Recorded 8 November 2026 for review of the complete-library candidate.

The previous source at `302e9ea` reproduced all three reported defects in Firefox 159.0.1 (build 20261103123426): an initial slider value of 15 with step 10 disagreed with the native range and submitted value; keyboard navigation left active options outside the visible listbox; and a required Select with a disabled selected option allowed submission without the field.

The regression story is `Verification/Control regressions/Stepped Values And Long Lists`. The optional `mise run test:controls:firefox` task builds Storybook, serves it on loopback and uses an existing Firefox in a temporary profile. It installs nothing. The runner writes results, runtime capabilities, a screenshot and its browser log to a new directory under `dist/verification/`.

The fixed source passed 38 browser assertions at a 1200 × 1100 window size:

- Slider initial/external normalization, changed constraints, unchanged normalized values, decimal ties, nonzero minima and partial final steps. Range, output, exact input, bound value, validity and named FormData agree.
- Real keyboard edits preserve intermediate digits in the exact field and commit with Tab or Enter. Enter submission captures the normalized value inside the form's submit handler.
- Required Select rejects initial, dynamically disabled and removed options. Invalid submission retains the count, focuses the visible trigger and exposes its error. Available options submit correctly.
- Editable and select-only comboboxes reveal their active option after End, Home, wrapping and repeated arrow navigation. Disabled options are skipped. Select typeahead and reopening reveal the matching selection. Checks measure option visibility and confirm unchanged document scroll and retained DOM focus.

Twelve server-rendered Slider regressions cover normalized input/output markup, decimal steps and stable large integral indices. Browser checks also caught and corrected two intermediate implementation problems: range sanitization during dynamic constraint updates, and premature normalization while typing into the exact field.

Full `mise run ci` passed, including 194 Python tests, the component SSR checks, Svelte/TypeScript checks, builds and deterministic artifacts. An earlier attempt received SIGTERM during an adapter test without an assertion failure; that interrupted run is not counted as a pass.

Three focused Orca 51.1 / Firefox 159.0.1 cases passed: Select option announcement/commit, editable Combobox filtering/commit, and Slider keyboard value/exact-field naming. The additional unavailable-selection probe announced the validation error, but focus moved before its final assertion. A standalone retry stopped when the owned Firefox window lost focus. Those two probes remain incomplete; they do not establish a passing screen-reader case. Both native sessions cleaned up their owned resources and preserved the default audio sink. The browser regression independently verified the unavailable-selection form behavior.

Local evidence is under `dist/verification/control-regressions-2026-11-08/`. `before/` retains the baseline failures, and `typing/` records the 38 passing browser assertions during development. `built/` records the same 38 passing checks against the built catalogue. `orca/` and `orca-required/` retain the focused screen-reader results and interruptions. An additional interrupted baseline probe is retained separately; it is not counted as a pass. Browser headless checks establish the tested Firefox behavior, not physical touch or other browser support.
