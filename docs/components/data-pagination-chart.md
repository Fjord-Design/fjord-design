# Data Table, Pagination and Chart

DataTable sorts and filters the supplied resident rows and selects stable row IDs. Select-all affects filtered rows only; other selection is preserved. Loading, empty and retryable error states are explicit. Pagination clamps invalid pages, resets to the first page on size changes and handles zero rows. Chart shows signed bars with numeric series labels and a complete visible data table, so colour is supplementary.

## API and ownership

DataTable: label, DataColumn[], DataRow[], bind:selected, selectable, loading, error, onretry. Pagination: total, bind:page, bind:pageSize, sizes, label. Chart: label, ChartDatum[].

## Tokens and states

Colours use shared `ink`, opaque `bg-base`/`bg-surface`/`bg-raised`, `accent`/`accent-soft`, `line-strong`, `focus-ring` and the documented status roles. Spacing, radii and typography use the existing foundation tokens. Focus is visible; colour is supplemented by native states, text or shape. Night, Day, Aurora and Fern examples are in the Complete Library catalogue.

## Limits and verification

The application fetches data, handles server sorting/filtering and slices pages; no network request is made. Chart rejects nonfinite values and handles zero, negative and disproportionate values. Chart labels/values remain in the accessible table. The named table container supports keyboard scrolling.

Tracking: XER-127. See [M3/M4 verification](../verification/m3-m4.md) for executed checks and remaining native/screen-reader limits. Component source is in `reference/svelte/src/lib/`; public props are the generated TypeScript declarations.
