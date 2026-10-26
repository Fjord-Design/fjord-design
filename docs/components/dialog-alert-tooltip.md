# Dialog, Alert and Tooltip

Import `@fjord/svelte/overlay-components.css` after the typography, foundation and semantic styles. Examples are in Components/Dialog Alert Tooltip.

## Dialog

Dialog is a modal native `dialog` with a visible title, optional short description, a close button and application-supplied content and actions. Give it a unique `id`, localized `title` and `closeLabel`, and bind `open` to application state. Opening calls `showModal()`; Escape, Close or setting `open` to false closes it. The browser makes the background inert and normally restores focus to the opener if it still exists. Choose a logical fallback in the application if that opener is removed.

Tab and Shift+Tab cycle through visible enabled controls inside the dialog. The built-in close button is initially focused. The cycle covers native elements in the dialog's light DOM, including radio groups; embedded frames and closed shadow roots need their own focus handling and are outside this component's contract. Avoid positive tabindex values. A dialog can contain navigation and actions, but an urgent decision that must interrupt work belongs to a separately specified Alert Dialog.

```svelte
<Button label="Open" onclick={() => open = true} />
<Dialog id="review" title="Review" description="Check the details." closeLabel="Close" bind:open>
  <p>Dialog content</p>
</Dialog>
```

The surface uses `bg-raised`, `ink`, `line`, `radius-window`, `shadow-overlay` and the `scrim` backdrop. Focus uses `focus-ring`, `border-focus` and `focus-offset`; the title uses the title-3 typography role. The dialog scrolls its own content when it exceeds the viewport.

## Alert

Alert presents a brief status with a visible title and message. Info, success, warning and danger use their semantic foreground and soft background pairs. Text communicates meaning alongside colour. An alert never moves focus or disappears automatically.

`announcement="off"` is suitable for static content. Use `polite` for ordinary dynamic status and `assertive` sparingly for urgent messages; they map to native live-region roles. Keep the alert mounted and update its text to request an announcement. Actual announcements vary by assistive technology, and initial page content is not guaranteed to be announced. An Alert is not a modal decision.

## Tooltip

Tooltip supplies supplementary text for one existing focusable control. Provide a unique `id`, text and a child snippet that spreads the supplied `aria-describedby` onto the trigger. If the trigger already has descriptions, merge their IDs rather than replacing them. The trigger must have its own accessible name and must not depend on the tooltip for essential instructions.

The tooltip opens on focus or pointer hover, remains visible while the pointer moves onto it, and closes on Escape. A touch on the trigger toggles the supplementary text; a touch outside dismisses it. The touch does not cancel the trigger's normal action. Use `align="end"` near the right edge. Tooltip content is non-interactive; use another pattern for links, buttons or content that requires focus. Its surface uses `bg-raised`, `ink`, `line` and `shadow-raised`; forced-colour mode uses system colours.

The local checks exercise native keyboard focus and synthetic touch pointer events. Actual touch hardware and assistive-technology announcements remain part of the release matrix. The [W3C modal dialog](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/), [alert](https://www.w3.org/WAI/ARIA/apg/patterns/alert/) and [tooltip](https://www.w3.org/WAI/ARIA/apg/patterns/tooltip/) guidance describes the interaction patterns; the tooltip pattern is still work in progress.
