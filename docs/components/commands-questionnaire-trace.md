# Command, Questionnaire and Trace

Command is a modal searchable command palette with grouped labels, keyboard selection, disabled entries, empty results, shortcuts and focus return. Questionnaire renders application-supplied Field controls, retains controlled answers across steps, validates before navigation/submission and exposes pending/retryable submission. Trace is an ordered process log with native expandable details and explicit state words, timing and error data.

## API and ownership

Command: id, label, items, bind:open, shortcut, onselect. Questionnaire: id, label, Question[], bind:answers, onsubmit, children(question,answer,setAnswer,error). Trace: label, TraceStep[].

## Tokens and states

Colours use shared `ink`, opaque `bg-base`/`bg-surface`/`bg-raised`, `accent`/`accent-soft`, `line-strong`, `focus-ring` and the documented status roles. Spacing, radii and typography use the existing foundation tokens. Focus is visible; colour is supplemented by native states, text or shape. Night, Day, Aurora and Fern examples are in the Complete Library catalogue.

## Limits and verification

Shortcut syntax is Control/Meta/Alt/Shift plus key; OS/browser reserved shortcuts may take precedence. Producers supply trace IDs/order/status/timing/errors. Questionnaire owns no persistence or network transport; onsubmit must resolve only after success. Use the provided error in Field for input association.

Tracking: XER-130. See [M3/M4 verification](../verification/m3-m4.md) for executed checks and remaining native/screen-reader limits. Component source is in `reference/svelte/src/lib/`; public props are the generated TypeScript declarations.
