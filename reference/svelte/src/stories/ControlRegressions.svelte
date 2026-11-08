<script lang="ts">
  import ThemeScope from "../lib/ThemeScope.svelte";
  import Slider from "../lib/Slider.svelte";
  import Combobox from "../lib/Combobox.svelte";
  import Select from "../lib/Select.svelte";
  import "../../../../dist/foundations.css";
  import "../../../../dist/semantics.css";
  import "../lib/complete-components.css";

  let value = $state(15);
  let min = $state(0),
    max = $state(100),
    step = $state(10);
  let choice = $state("item-29");
  let unavailable = $state(true);
  let removed = $state(false);
  let submissions = $state(0);
  let sliderSubmitted = $state("");
  const options = $derived(
    Array.from({ length: 30 }, (_, i) => ({
      value: `item-${i}`,
      label:
        i === 29
          ? "Zebra destination"
          : `Destination ${String(i).padStart(2, "0")}`,
      disabled: i === 4 || (i === 29 && unavailable),
    })).filter((option) => !removed || option.value !== "item-29"),
  );
  function configure(
    nextMin: number,
    nextMax: number,
    nextStep: number,
    nextValue: number,
  ) {
    min = nextMin;
    max = nextMax;
    step = nextStep;
    value = nextValue;
  }
</script>

<ThemeScope mode="day">
  <main
    class="fjord-foundation fjord-semantic fjord-widget control-regressions"
  >
    <h1>Stepped values and long option lists</h1>
    <form
      id="slider-form"
      onsubmit={(event) => {
        event.preventDefault();
        sliderSubmitted = String(
          new FormData(event.currentTarget).get("volume"),
        );
      }}
    >
      <Slider
        id="regression-slider"
        label="Volume"
        name="volume"
        bind:value
        {min}
        {max}
        {step}
      />
      <p>Bound value: <span id="slider-bound">{value}</span></p>
      <p>
        Submitted volume: <span id="slider-submitted">{sliderSubmitted}</span>
      </p>
      <button
        id="slider-same-value"
        type="button"
        onclick={() => configure(0.1, 20, 0.1, 20)}
        >Change bounds while keeping volume at 20</button
      >
      <button id="slider-external" type="button" onclick={() => (value = 35)}
        >Set volume to 35</button
      >
      <button
        id="slider-offset"
        type="button"
        onclick={() => configure(10, 20, 3, 20)}
        >Use an offset and partial final step</button
      >
      <button
        id="slider-decimal"
        type="button"
        onclick={() => configure(0.1, 0.6, 0.1, 0.35)}
        >Use fractional steps</button
      >
      <button id="slider-step" type="button" onclick={() => (step = 0.2)}
        >Change step to 0.2</button
      >
      <button id="slider-min" type="button" onclick={() => (min = 0.2)}
        >Change minimum to 0.2</button
      >
      <button
        id="slider-reset"
        type="button"
        onclick={() => configure(0, 100, 10, 15)}>Reset volume</button
      >
    </form>
    <section aria-label="Destination choices">
      <Combobox
        id="regression-combo"
        label="Search destination"
        options={options.map((option) => ({
          ...option,
          disabled: option.value === "item-4",
        }))}
      />
      <form
        id="select-form"
        onsubmit={(event) => {
          event.preventDefault();
          submissions += 1;
        }}
      >
        <Select
          id="regression-select"
          label="Destination"
          name="destination"
          bind:value={choice}
          {options}
          required
        />
        <button id="select-submit" type="submit">Save destination</button>
        <p>
          Successful submissions: <span id="submissions">{submissions}</span>
        </p>
      </form>
      <button
        id="select-enable"
        type="button"
        onclick={() => (unavailable = false)}>Enable Zebra destination</button
      >
      <button
        id="select-disable"
        type="button"
        onclick={() => (unavailable = true)}>Disable Zebra destination</button
      >
      <button id="select-remove" type="button" onclick={() => (removed = true)}
        >Remove Zebra destination</button
      >
      <button
        id="select-restore"
        type="button"
        onclick={() => {
          removed = false;
          unavailable = false;
          choice = "item-29";
        }}>Restore Zebra destination</button
      >
    </section>
  </main>
</ThemeScope>

<style>
  .control-regressions {
    max-width: 44rem;
    margin: 1rem auto;
    padding: 1rem;
  }
  section {
    margin-top: 2rem;
  }
  form {
    display: grid;
    gap: 0.5rem;
  }
</style>
