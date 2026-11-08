<script lang="ts">
  import {
    ThemeScope,
    Tabs,
    Card,
    Field,
    Input,
    Switch,
    Button,
    Select,
    DataTable,
    Pagination,
    Chart,
    Message,
    MessageScroller,
    Attachment,
    Textarea,
    Toast,
    type ModePreference,
  } from "../lib/index.js";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  import "../lib/form-controls.css";
  import "../lib/choices.css";
  import "../lib/layout-components.css";
  import "../lib/complete-components.css";
  let {
    mode = "night",
    identity = "fjord",
  }: { mode?: ModePreference; identity?: string } = $props();
  let tab = $state("settings"),
    name = $state("Northern project"),
    notifications = $state(true),
    destination = $state(""),
    saved = $state(false);
  let page = $state(1),
    pageSize = $state(10),
    selected = $state<string[]>([]),
    draft = $state("");
  let messages = $state([
    "The project is ready for review.",
    "The complete data is available in the Data tab.",
  ]);
  const rows = Array.from({ length: 23 }, (_, i) => ({
    id: String(i + 1),
    name: `Project ${i + 1}`,
    count: (i + 1) * 3,
  }));
</script>

<ThemeScope {mode} {identity}>
  <main class="fjord-foundation fjord-semantic fjord-widget compositions">
    <h1>Project workspace</h1>
    <p>
      Settings, resident data and a conversation share component contracts. This
      example keeps all changes in memory.
    </p>
    <div class="settings">
      <label
        >Mode <select bind:value={mode}
          ><option value="night">Night</option><option value="day">Day</option
          ><option value="aurora">Aurora</option></select
        ></label
      >
      <label
        >Identity <select bind:value={identity}
          ><option value="fjord">Fjord</option><option value="fern">Fern</option
          ><option value="work">Work</option><option value="gaming"
            >Gaming</option
          ></select
        ></label
      >
    </div>
    <Tabs
      id="workspace"
      label="Project workspace"
      items={[
        { value: "settings", label: "Settings" },
        { value: "data", label: "Data" },
        { value: "messages", label: "Messages" },
      ]}
      bind:value={tab}
      panelTabbable={false}
    >
      {#snippet children(section)}
        {#if section === "settings"}
          <Card as="article"
            ><h2>Project settings</h2>
            <form
              onsubmit={(event) => {
                event.preventDefault();
                saved = true;
              }}
            >
              <Field id="workspace-name" label="Project name" required
                >{#snippet children(attributes)}<Input
                    {...attributes}
                    name="project"
                    bind:value={name}
                  />{/snippet}</Field
              >
              <Select
                id="workspace-destination"
                label="Default destination"
                options={[
                  { value: "local", label: "Local workspace" },
                  { value: "shared", label: "Shared workspace" },
                ]}
                bind:value={destination}
                name="destination"
                required
              />
              <Switch
                label="Notify me about project changes"
                bind:checked={notifications}
              />
              <Button type="submit" label="Save settings" />
            </form>
            <Toast
              title="Settings saved"
              message={`Saved ${name}; destination ${destination}; notifications ${notifications ? "on" : "off"}.`}
              bind:open={saved}
            />
          </Card>
        {:else if section === "data"}
          <h2>Project data</h2>
          <DataTable
            label="Workspace projects"
            columns={[
              { key: "name", label: "Project", sortable: true },
              { key: "count", label: "Items", sortable: true },
            ]}
            rows={rows.slice((page - 1) * pageSize, page * pageSize)}
            bind:selected
          />
          <Pagination
            label="Workspace project pages"
            total={rows.length}
            bind:page
            bind:pageSize
          />
          <p role="status">
            Selected project IDs: {selected.join(", ") || "none"}
          </p>
          <Chart
            label="Project item counts"
            data={rows
              .slice(0, 5)
              .map((row) => ({ label: row.name, value: row.count }))}
          />
        {:else}
          <h2>Project conversation</h2>
          <MessageScroller
            label="Workspace conversation"
            count={messages.length}
          >
            {#each messages as message, index}<Message
                id={`workspace-message-${index}`}
                sender={index % 2 ? "Alex" : "Sam"}
                timestamp="2026-11-08T12:00:00+01:00"
                timeLabel="12:00"
                outgoing={index % 2 === 1}>{message}</Message
              >{/each}
          </MessageScroller>
          <Attachment
            id="workspace-report"
            filename="project-report.csv"
            type="text/csv"
            size={2048}
          />
          <form
            onsubmit={(event) => {
              event.preventDefault();
              if (draft.trim()) {
                messages = [...messages, draft.trim()];
                draft = "";
              }
            }}
          >
            <Field id="workspace-reply" label="Reply" required
              >{#snippet children(attributes)}<Textarea
                  {...attributes}
                  bind:value={draft}
                />{/snippet}</Field
            >
            <Button type="submit" label="Send reply" />
          </form>
        {/if}
      {/snippet}
    </Tabs>
  </main>
</ThemeScope>

<style>
  .compositions {
    padding: min(var(--fjord-space-5), 5vw);
    display: grid;
    gap: var(--fjord-space-4);
    max-inline-size: 64rem;
    margin-inline: auto;
  }
  form {
    display: grid;
    gap: var(--fjord-space-4);
  }
  .settings {
    display: flex;
    flex-wrap: wrap;
    gap: var(--fjord-space-3);
  }
  h1,
  h2,
  p {
    overflow-wrap: anywhere;
  }
</style>
