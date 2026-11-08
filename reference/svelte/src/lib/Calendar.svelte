<script lang="ts">
  import { tick } from "svelte";
  import { parseDate, isoDate, addDays, addMonths, todayDate } from "./date.js";
  let {
    label,
    value = $bindable(""),
    endValue = $bindable(""),
    range = false,
    locale = "en-GB",
    weekStartsOn = 1,
    min,
    max,
    disabledDate,
    onchange,
  }: {
    label: string;
    value?: string;
    endValue?: string;
    range?: boolean;
    locale?: string;
    weekStartsOn?: number;
    min?: string;
    max?: string;
    disabledDate?: (date: string) => boolean;
    onchange?: (value: string, endValue: string) => void;
  } = $props();
  let active = $state(""),
    root: HTMLDivElement;
  const initial = $derived(
    value ||
      (min && todayDate() < min
        ? min
        : max && todayDate() > max
          ? max
          : todayDate()),
  );
  const current = $derived(active || initial);
  const first = $derived(current.slice(0, 8) + "01");
  const days = $derived.by(() => {
    if (!Number.isInteger(weekStartsOn) || weekStartsOn < 0 || weekStartsOn > 6)
      throw new Error("Calendar weekStartsOn must be 0–6.");
    parseDate(current);
    if (value) parseDate(value);
    if (endValue) parseDate(endValue);
    if (min) parseDate(min);
    if (max) parseDate(max);
    if (min && max && min > max)
      throw new Error("Calendar min must not exceed max.");
    const offset = (parseDate(first).getUTCDay() - weekStartsOn + 7) % 7;
    return Array.from({ length: 42 }, (_, i) => addDays(first, i - offset));
  });
  const weekdays = $derived(
    Array.from({ length: 7 }, (_, i) =>
      new Intl.DateTimeFormat(locale, {
        weekday: "short",
        timeZone: "UTC",
      }).format(parseDate(addDays("2024-01-07", i + weekStartsOn))),
    ),
  );
  const title = $derived(
    new Intl.DateTimeFormat(locale, {
      month: "long",
      year: "numeric",
      timeZone: "UTC",
    }).format(parseDate(first)),
  );
  function disabled(date: string) {
    return !!(
      (min && date < min) ||
      (max && date > max) ||
      disabledDate?.(date)
    );
  }
  const tabbable = $derived(
    days.find((date) => date === current && !disabled(date)) ??
      days.find((date) => !disabled(date)),
  );
  function select(date: string) {
    if (disabled(date)) return;
    if (!range) {
      value = date;
    } else if (!value || endValue) {
      value = date;
      endValue = "";
    } else {
      endValue = date >= value ? date : value;
      value = date >= value ? value : date;
    }
    onchange?.(value, endValue);
  }
  async function focus(date: string) {
    let next = date;
    if (min && next < min) next = min;
    if (max && next > max) next = max;
    for (let i = 0; i < 366 && disabled(next); i++)
      next = addDays(next, next < current ? -1 : 1);
    if (disabled(next)) return;
    active = next;
    await tick();
    root.querySelector<HTMLButtonElement>(`[data-date="${next}"]`)?.focus();
  }
  function key(e: KeyboardEvent, date: string) {
    let next = date;
    const rtl = getComputedStyle(root).direction === "rtl";
    if (e.key === "ArrowRight") next = addDays(date, rtl ? -1 : 1);
    else if (e.key === "ArrowLeft") next = addDays(date, rtl ? 1 : -1);
    else if (e.key === "ArrowDown") next = addDays(date, 7);
    else if (e.key === "ArrowUp") next = addDays(date, -7);
    else if (e.key === "Home")
      next = addDays(
        date,
        -((parseDate(date).getUTCDay() - weekStartsOn + 7) % 7),
      );
    else if (e.key === "End")
      next = addDays(
        date,
        6 - ((parseDate(date).getUTCDay() - weekStartsOn + 7) % 7),
      );
    else if (e.key === "PageDown") next = addMonths(date, e.shiftKey ? 12 : 1);
    else if (e.key === "PageUp") next = addMonths(date, e.shiftKey ? -12 : -1);
    else return;
    e.preventDefault();
    void focus(next);
  }
</script>

<div class="fjord-widget fjord-calendar" bind:this={root}>
  <div class="fjord-calendar-head">
    <button
      type="button"
      aria-label="Previous month"
      onclick={() => focus(addMonths(current, -1))}>‹</button
    ><strong aria-live="polite">{title}</strong><button
      type="button"
      aria-label="Next month"
      onclick={() => focus(addMonths(current, 1))}>›</button
    >
  </div>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex (The two-dimensional date grid needs named keyboard scrolling at enlarged text sizes.) -->
  <div
    class="fjord-calendar-grid"
    role="region"
    aria-label={`${label}, scrollable calendar grid`}
    tabindex="0"
  >
    <table role="grid" aria-label={label}>
      <thead
        ><tr
          >{#each weekdays as weekday}<th scope="col">{weekday}</th>{/each}</tr
        ></thead
      ><tbody
        >{#each Array.from({ length: 6 }, (_, i) => i) as row}<tr
            >{#each days.slice(row * 7, row * 7 + 7) as date}<td
                aria-selected={date === value ||
                  date === endValue ||
                  (range && !!endValue && date > value && date < endValue)}
                ><button
                  type="button"
                  data-date={date}
                  data-outside={date.slice(0, 7) !== first.slice(0, 7)}
                  disabled={disabled(date)}
                  tabindex={date === tabbable ? 0 : -1}
                  aria-label={new Intl.DateTimeFormat(locale, {
                    dateStyle: "full",
                    timeZone: "UTC",
                  }).format(parseDate(date))}
                  aria-current={date === todayDate() ? "date" : undefined}
                  onclick={() => {
                    active = date;
                    select(date);
                  }}
                  onkeydown={(e) => key(e, date)}
                  >{parseDate(date).getUTCDate()}</button
                ></td
              >{/each}</tr
          >{/each}</tbody
      >
    </table>
  </div>
  {#if range}<p role="status">
      {value
        ? endValue
          ? `${value} – ${endValue}`
          : `Start ${value}; choose an end date`
        : "Choose a start date"}
    </p>{/if}
</div>
