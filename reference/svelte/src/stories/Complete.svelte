<script lang="ts">
  import {
    ThemeScope,
    Field,
    Input,
    Accordion,
    Collapsible,
    AspectRatio,
    Item,
    Avatar,
    Breadcrumb,
    NavigationMenu,
    Sidebar,
    AlertDialog,
    Drawer,
    Sheet,
    Popover,
    HoverCard,
    Select,
    Combobox,
    DropdownMenu,
    ContextMenu,
    Menubar,
    ButtonGroup,
    InputGroup,
    InputOTP,
    Toggle,
    ToggleGroup,
    Calendar,
    DatePicker,
    Slider,
    DataTable,
    Pagination,
    Chart,
    Toast,
    Progress,
    Marker,
    Attachment,
    Bubble,
    Message,
    MessageScroller,
    Command,
    Questionnaire,
    Trace,
    Carousel,
    Resizable,
    ScrollArea,
    Direction,
    type ModePreference,
    type MenuItem,
  } from "../lib/index.js";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  import "../lib/form-controls.css";
  import "../lib/complete-components.css";
  let {
    family = "disclosures",
    mode = "night",
    identity = "fjord",
  }: { family?: string; mode?: ModePreference; identity?: string } = $props();
  let accordion = $state<string[]>([]),
    alertOpen = $state(false),
    sheetOpen = $state(false),
    drawerOpen = $state(false),
    commandOpen = $state(false),
    decision = $state("No action");
  let selection = $state(""),
    combo = $state(""),
    otp = $state(""),
    toggle = $state(false),
    toggles = $state<string[]>([]),
    date = $state("2028-02-29"),
    rangeStart = $state("2028-02-10"),
    rangeEnd = $state("2028-02-14"),
    slider = $state(40),
    page = $state(1),
    pageSize = $state(10),
    toast = $state(true),
    slide = $state(0),
    split = $state(50),
    menuChecked = $state(false),
    radio = $state("work");
  let failSubmission = $state(false);
  let messages = $state(
    Array.from(
      { length: 20 },
      (_, i) =>
        `Message ${i + 1}: a retained conversation with long text that wraps.`,
    ),
  );
  const nav = [
    { label: "Overview", href: "#overview", current: true },
    {
      label: "An intentionally long project navigation destination",
      href: "#details",
    },
    { label: "Settings", href: "#settings" },
  ];
  const options = [
    { value: "one", label: "First option", group: "Available" },
    {
      value: "two",
      label: "Second option with a longer translated name",
      group: "Available",
    },
    { value: "disabled", label: "Unavailable", disabled: true },
    { value: "three", label: "Third option" },
  ];
  const menuItems = $derived<MenuItem[]>([
    { id: "new", label: "New document", shortcut: "Control+n" },
    {
      id: "check",
      label: "Show details",
      kind: "checkbox",
      checked: menuChecked,
    },
    {
      id: "work",
      label: "Work",
      kind: "radio",
      checked: radio === "work",
      group: "mode",
    },
    {
      id: "play",
      label: "Gaming",
      kind: "radio",
      checked: radio === "play",
      group: "mode",
    },
    {
      id: "export",
      label: "Export",
      children: [
        { id: "json", label: "Export JSON" },
        { id: "csv", label: "Export CSV" },
      ],
    },
    { id: "locked", label: "Unavailable action", disabled: true },
  ]);
  function action(id: string) {
    if (id === "check") menuChecked = !menuChecked;
    if (id === "work" || id === "play") radio = id;
    decision = id;
  }
</script>

<ThemeScope {mode} {identity}
  ><main class="fjord-foundation fjord-semantic fjord-widget fjord-complete">
    <h1>Complete library: {family}</h1>
    <p>
      Keyboard, long-text and state examples. Actions stay in this local
      example.
    </p>
    <div class="fjord-example-settings">
      <label
        >Mode <select bind:value={mode}
          ><option value="night">Night</option><option value="day">Day</option
          ><option value="aurora">Aurora</option><option value="system"
            >System</option
          ></select
        ></label
      ><label
        >Identity <select bind:value={identity}
          ><option value="fjord">Fjord</option><option value="fern">Fern</option
          ><option value="work">Work</option><option value="gaming"
            >Gaming</option
          ></select
        ></label
      >
    </div>
    {#if family === "disclosures"}
      <h2>Accordion</h2>
      <Accordion
        id="complete-accordion"
        items={[
          { value: "first", label: "First section" },
          {
            value: "second",
            label:
              "A longer section heading with enough words to check wrapping",
          },
          { value: "disabled", label: "Unavailable section", disabled: true },
        ]}
        bind:value={accordion}
        >{#snippet children(value)}<p>Panel {value}</p>
          <label>Retained note <input value="Keep this note" /></label
          >{/snippet}</Accordion
      >
      <Collapsible label="Show supporting details"
        ><p>Collapsible content remains in the document while closed.</p>
        <button type="button">Supporting action</button></Collapsible
      ><AspectRatio><p>Reserved 16:9 media area</p></AspectRatio><AspectRatio
        ratio={4 / 3}
      /><Item
        title="Project item"
        description="A long description that keeps the title and metadata readable at narrow widths."
        metadata="Updated today"
        >{#snippet actions()}<button
            type="button"
            onclick={() => (decision = "Item action")}>Edit item</button
          >{/snippet}</Item
      ><Item title="Linked item" href="#details" />
    {:else if family === "navigation"}
      <h2>Identity and location</h2>
      <Avatar label="Alex Fjord" /><Avatar
        label="Image fallback"
        src="data:image/png;base64,invalid"
        fallback="IF"
      /><Avatar label="Decorative" decorative /><Breadcrumb
        label="Project breadcrumb"
        items={nav.map((item) => ({ ...item, current: false }))}
      /><NavigationMenu label="Primary navigation" items={nav} /><Sidebar
        id="complete-sidebar"
        label="Project navigation"
        items={nav}
      /><Direction direction="rtl"
        ><NavigationMenu label="RTL navigation" items={nav} /></Direction
      >
    {:else if family === "overlays"}
      <ButtonGroup label="Open panels"
        ><button type="button" onclick={() => (alertOpen = true)}
          >Delete project</button
        ><button type="button" onclick={() => (sheetOpen = true)}
          >Open sheet</button
        ><button type="button" onclick={() => (drawerOpen = true)}
          >Open drawer</button
        ></ButtonGroup
      >
      <AlertDialog
        id="complete-alert"
        title="Delete this project?"
        description="This local example does not delete data."
        bind:open={alertOpen}
        ><p>Choose a response.</p>
        {#snippet actions()}<button
            type="button"
            onclick={() => {
              decision = "Confirmed";
              alertOpen = false;
            }}>Confirm deletion</button
          >{/snippet}</AlertDialog
      >
      <Sheet id="complete-sheet" title="Project settings" bind:open={sheetOpen}
        ><label>Project name <input value="Fjord" /></label><button
          type="button"
          onclick={() => (alertOpen = true)}>Open nested confirmation</button
        >
        <p>Explicit controls close this edge panel.</p></Sheet
      >
      <Drawer
        id="complete-drawer"
        title="Supporting information"
        bind:open={drawerOpen}
        ><p>
          Drag the handle toward the screen edge, or activate either close
          control.
        </p>
        <button type="button">Drawer action</button></Drawer
      >
      <Popover id="complete-popover" label="Open details"
        ><p>Interactive supplementary information.</p>
        <a href="#details">Read details</a></Popover
      ><HoverCard id="complete-hovercard" label="Profile preview"
        ><p>Profile information also opens by keyboard or touch.</p>
        <a href="#profile">Open profile</a></HoverCard
      >
    {:else if family === "menus"}
      <Select
        id="complete-select"
        label="Select an option"
        {options}
        bind:value={selection}
        name="selection"
      /><Combobox
        id="complete-combo"
        label="Find an option"
        {options}
        bind:value={combo}
        name="combo"
      /><Combobox
        id="loading-combo"
        label="Loading options"
        options={[]}
        loading
      /><Select
        id="error-select"
        label="Unavailable options"
        options={[]}
        error="Options could not be loaded"
      />
      <DropdownMenu
        id="complete-menu"
        label="Actions"
        items={menuItems}
        onselect={action}
      /><ContextMenu
        id="complete-context"
        label="Context actions"
        items={menuItems}
        onselect={action}
      /><Menubar
        id="complete-menubar"
        label="Editor commands"
        menus={[
          { label: "File", items: menuItems },
          { label: "View", items: menuItems },
        ]}
        onselect={action}
      />
      <p role="status">
        Selected {selection || "none"}; combobox {combo || "none"}; action {decision}
      </p>
    {:else if family === "forms"}
      <form
        onsubmit={(e) => {
          e.preventDefault();
          decision = `Code ${otp}`;
        }}
      >
        <ButtonGroup label="Document actions"
          ><button type="button">Save draft</button><button type="button"
            >Preview</button
          ><button type="button" disabled>Publish unavailable</button
          ></ButtonGroup
        ><InputGroup label="Website address" prefix="https://" suffix=".example"
          ><label>Site name <input name="site" value="fjord" /></label
          ></InputGroup
        ><InputOTP
          id="complete-otp"
          label="Verification code"
          name="code"
          bind:value={otp}
          required
        /><button type="submit">Verify code</button>
      </form>
      <Toggle label="Bold" bind:pressed={toggle} /><ToggleGroup
        label="Text alignment"
        {options}
        bind:value={toggles}
      /><ToggleGroup label="Multiple styles" {options} multiple />
      <p role="status">{decision}; bold {toggle ? "on" : "off"}</p>
    {:else if family === "dates"}
      <Calendar
        label="Appointment date"
        bind:value={date}
        min="2028-01-01"
        max="2029-12-31"
      /><Calendar
        label="Date range"
        range
        bind:value={rangeStart}
        bind:endValue={rangeEnd}
      /><DatePicker
        id="complete-date"
        label="Start date"
        bind:value={date}
        min="2028-01-01"
        max="2029-12-31"
      /><Slider
        id="complete-slider"
        label="Volume"
        bind:value={slider}
        min={0}
        max={100}
        step={5}
      />
      <p role="status">Selected date {date}; value {slider}</p>
    {:else if family === "data"}
      <DataTable
        label="Project records"
        columns={[
          { key: "name", label: "Name", sortable: true },
          { key: "count", label: "Count", sortable: true },
        ]}
        rows={[
          { id: "one", name: "Fjord", count: 12 },
          { id: "two", name: "An intentionally long record title", count: 3 },
          { id: "three", name: "Aurora", count: 24 },
        ]}
      /><DataTable
        label="Empty records"
        columns={[{ key: "name", label: "Name" }]}
        rows={[]}
      /><DataTable
        label="Failed records"
        columns={[{ key: "name", label: "Name" }]}
        rows={[]}
        error="Could not load records"
        onretry={() => (decision = "Retry requested")}
      /><Pagination total={123} bind:page bind:pageSize /><Chart
        label="Signed values"
        data={[
          { label: "Positive", value: 100 },
          { label: "Negative", value: -40 },
          { label: "Zero", value: 0 },
          { label: "Very small", value: 0.1 },
        ]}
      /><Chart label="Empty chart" data={[]} />
    {:else if family === "feedback"}
      <button type="button" onclick={() => (toast = true)}
        >Show notification</button
      ><Toast
        title="Draft ready"
        message="Your local draft is ready for review. This notification stays until dismissed."
        bind:open={toast}
        ><button type="button" onclick={() => (decision = "Review opened")}
          >Review draft</button
        ></Toast
      ><Progress
        id="complete-progress"
        label="Known progress"
        value={45}
      /><Progress id="unknown-progress" label="Unknown progress" /><Marker
        label="3 unread"><button type="button">Inbox</button></Marker
      >
    {:else if family === "messages"}
      <Attachment
        id="ready-file"
        filename="project-notes.txt"
        type="text/plain"
        size={2048}
      /><Attachment
        id="uploading-file"
        filename="long-project-document.pdf"
        type="application/pdf"
        size={12000}
        status="uploading"
        progress={38}
      /><Attachment
        id="failed-file"
        filename="failed.txt"
        type="text/plain"
        size={42}
        status="error"
        onretry={() => (decision = "Retry requested")}
      /><Bubble
        ><p>A standalone content bubble with a longer sentence.</p></Bubble
      ><button
        type="button"
        onclick={() =>
          (messages = [...messages, `New message ${messages.length + 1}`])}
        >Append message</button
      ><MessageScroller label="Project conversation" count={messages.length}
        >{#each messages as message, i}<Message
            id={`message-${i}`}
            sender={i % 2 ? "Alex" : "Sam"}
            timestamp="2026-11-08T12:00:00+01:00"
            timeLabel="12:00"
            outgoing={i % 2 === 0}
            status={i === messages.length - 1 ? "Delivered" : undefined}
            ><p>{message}</p>
            {#if i === 0}<pre><code>const theme = "fjord";</code
                ></pre>{/if}</Message
          >{/each}</MessageScroller
      >
    {:else if family === "workflows"}
      <button type="button" onclick={() => (commandOpen = true)}
        >Open command palette</button
      ><Command
        id="complete-command"
        label="Project commands"
        items={menuItems}
        bind:open={commandOpen}
        onselect={action}
      /><label
        ><input type="checkbox" bind:checked={failSubmission} />Fail next
        submission</label
      ><Questionnaire
        id="complete-questionnaire"
        label="Project questionnaire"
        questions={[
          {
            id: "name",
            title: "Project name",
            validate: (answer) =>
              answer.trim() ? undefined : "Enter a project name.",
          },
          { id: "notes", title: "Project notes" },
        ]}
        onsubmit={async () => {
          await new Promise((resolve) => setTimeout(resolve, 80));
          if (failSubmission) {
            failSubmission = false;
            throw new Error(
              "Could not save responses. Retry with your retained answers.",
            );
          }
          decision = "Local responses collected";
        }}
        >{#snippet children(question, answer, setAnswer, error)}<Field
            id={`question-${question.id}`}
            label={question.title}
            {error}
            >{#snippet children(control)}<Input
                {...control}
                value={answer}
                oninput={(e) => setAnswer(e.currentTarget.value)}
              />{/snippet}</Field
          >{/snippet}</Questionnaire
      ><Trace
        label="Build process"
        steps={[
          {
            id: "resolve",
            label: "Resolve tokens",
            status: "complete",
            durationMs: 12,
            detail: "400 tokens resolved.",
          },
          {
            id: "validate",
            label: "Validate contrast",
            status: "running",
            startedAt: "2026-11-08T12:00:00+01:00",
          },
          {
            id: "export",
            label: "Export",
            status: "error",
            error: "Destination unavailable",
            detail: "Choose another output directory.",
          },
        ]}
      />
      <p role="status">{decision}</p>
    {:else if family === "layout"}
      <Carousel
        label="Project examples"
        slides={[
          { id: "one", label: "Overview" },
          { id: "two", label: "Details" },
          { id: "three", label: "Next steps" },
        ]}
        bind:index={slide}
        >{#snippet children(index)}<p>
            Slide content {index + 1}, no automatic rotation.
          </p>{/snippet}</Carousel
      ><Resizable label="Resize navigation and content" bind:value={split}
        >{#snippet first()}<p>First panel</p>
          <label>Retained input <input value="Keep this value" /></label
          >{/snippet}{#snippet second()}<p>
            Second panel with a longer explanation that wraps.
          </p>{/snippet}</Resizable
      ><ScrollArea label="Scrollable notes" blockSize={180}
        >{#each Array.from({ length: 20 }, (_, i) => i) as i}<p>
            Note {i + 1}
          </p>{/each}</ScrollArea
      ><Direction direction="rtl"
        ><p>RTL content preserves reading order and uses logical spacing.</p>
        <Breadcrumb label="RTL breadcrumb" items={nav} /></Direction
      >
    {/if}
    <output aria-label="Last example action">{decision}</output>
  </main></ThemeScope
>

<style>
  .fjord-complete {
    padding: min(var(--fjord-space-5), 5vw);
    display: grid;
    gap: var(--fjord-space-4);
    max-inline-size: 64rem;
    margin-inline: auto;
  }
  .fjord-example-settings {
    display: flex;
    flex-wrap: wrap;
    gap: var(--fjord-space-3);
  }
  h1,
  h2,
  p {
    margin: 0;
  }
  form {
    display: grid;
    gap: var(--fjord-space-3);
  }
  .fjord-complete :global([role="option"]) {
    padding: var(--fjord-space-3);
    display: block;
    inline-size: 100%;
    text-align: start;
  }
</style>
