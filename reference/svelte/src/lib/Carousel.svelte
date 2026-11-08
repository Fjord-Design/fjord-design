<script lang="ts">
  import type { Snippet } from "svelte";
  import type { CarouselSlide } from "./complete-types.js";
  let {
    label,
    slides,
    index = $bindable(0),
    children,
  }: {
    label: string;
    slides: CarouselSlide[];
    index?: number;
    children: Snippet<[number]>;
  } = $props();
  const current = $derived.by(() => {
    if (
      !Number.isFinite(index) ||
      new Set(slides.map((slide) => slide.id)).size !== slides.length
    )
      throw new Error("Carousel needs a finite index and unique slides.");
    return Math.max(0, Math.min(slides.length - 1, Math.trunc(index)));
  });
</script>

<section
  class="fjord-widget fjord-carousel"
  aria-roledescription="carousel"
  aria-label={label}
>
  {#if slides.length}{#each slides as slide, i}<div
        role="group"
        aria-roledescription="slide"
        aria-label={`${i + 1} of ${slides.length}: ${slide.label}`}
        hidden={i !== current}
      >
        {@render children(i)}
      </div>{/each}
    <div class="fjord-button-group">
      <button
        type="button"
        aria-label="Previous slide"
        disabled={current === 0}
        onclick={() => (index = current - 1)}>Previous</button
      ><span role="status"
        >Slide {current + 1} of {slides.length}: {slides[current].label}</span
      ><button
        type="button"
        aria-label="Next slide"
        disabled={current === slides.length - 1}
        onclick={() => (index = current + 1)}>Next</button
      >
    </div>{:else}<p>No slides</p>{/if}
</section>
