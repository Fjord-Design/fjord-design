# Component meanings and boundaries

Use these definitions in specifications, the Svelte reference and adapter documentation. The [inventory](inventory.md) assigns each of the 66 source components to a delivery issue.

## Content and communication

| Component | Responsibility | Consumer-owned behaviour |
| --- | --- | --- |
| Attachment | Display file name, type, size, preview availability, upload progress and failure state, with named actions | File access, upload, download authorisation, retry and storage |
| Bubble | Present the visual body of one message, including long text or code | Message identity, sender metadata and delivery state belong to Message or the application |
| Message | Group sender, time, content, attachments, delivery state and actions into one semantic message | Transport, persistence, permissions and action effects |
| Message Scroller | Present message history, preserve the reading position, offer jump-to-latest and announce new content without replaying history | Loading pages, ordering and storing messages |

Bubble is a content treatment; Message is the complete unit. A message history may use an appropriate log/list structure. Individual message bodies must not all become live regions. Scrolling must respect a reader who has moved away from the newest message.

## Structure, status and processes

| Component | Responsibility | Consumer-owned behaviour |
| --- | --- | --- |
| Item | Compose a row with title, icon, description, metadata and named actions | Selection, navigation and domain actions |
| Marker | Attach a compact indicator to a target, such as unread, current or attention state, with text or a named shape | The state being indicated and any change to it |
| Questionnaire | Compose Field controls into a form sequence with validation, progress and retained answers when navigating | Question definitions, controlled answer state, submission and persistence |
| Trace | Display ordered process steps with status, start/end or duration, errors and expandable details | Producing events, running steps, retrying and storing logs |

Item adopts the semantics of its context: a list row, link or button needs the corresponding element and keyboard behaviour. It must not create nested interactive controls with conflicting activation.

Marker is attached to a target; Badge is a standalone label. A Marker always conveys its meaning beyond colour. It is not a text-highlighting control.

Trace consumers supply stable step identifiers, labels, order, status, available timing and error/details data. A Trace renders that supplied state. Questionnaire builds on the shared field controls and preserves answers through its controlled state. Both components are reusable outside chat and assistant interfaces.

## Controls with related names

| Pair | Contract |
| --- | --- |
| Native Select / Select | Native Select uses the platform's HTML `select` for ordinary choices. Select implements a custom trigger and option popup with documented selection, focus and keyboard behaviour. Searchable input belongs to Combobox. |
| Dialog / Alert Dialog | Dialog contains general modal content. Alert Dialog communicates an important interruption and requires an explicit response, with safe initial focus and clear actions. Both define dismissal and focus return. |
| Drawer / Sheet | Drawer is a gesture-operated edge panel with a visible alternative close control. Sheet is a positioned edge panel with explicit controls. Both specify modality, Escape behaviour, focus, scrolling and responsive placement; gestures must never be the only means of operation. |

Use native HTML where it supplies the required behaviour, then apply the relevant [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/patterns/) for custom controls. Detailed component APIs and state tests belong to their delivery issues. Platform limitations must be recorded with the implementation.
