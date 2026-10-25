<script lang="ts">
  import { tick } from "svelte";

  type Option = { value: string; label: string };

  let {
    options,
    value,
    onchange,
  }: {
    options: Option[];
    value: string;
    onchange: (value: string) => void;
  } = $props();

  let container: HTMLDivElement | undefined = $state();
  let thumb = $state({ left: 0, width: 0, ready: false });

  function measure() {
    if (!container) return;
    const index = options.findIndex((option) => option.value === value);
    const buttons = container.querySelectorAll<HTMLButtonElement>("button");
    const button = buttons[index];
    if (!button) return;
    thumb = {
      left: button.offsetLeft,
      width: button.offsetWidth,
      ready: true,
    };
  }

  $effect(() => {
    value;
    options;
    tick().then(measure);
  });

  $effect(() => {
    if (!container) return;
    const observer = new ResizeObserver(measure);
    observer.observe(container);
    return () => observer.disconnect();
  });
</script>

<div class="seg" bind:this={container} role="tablist">
  {#if thumb.ready}
    <span
      class="seg-thumb"
      style:left="{thumb.left}px"
      style:width="{thumb.width}px"
      aria-hidden="true"
    ></span>
  {/if}
  {#each options as option (option.value)}
    <button
      class:active={option.value === value}
      role="tab"
      aria-selected={option.value === value}
      onclick={() => onchange(option.value)}
    >
      {option.label}
    </button>
  {/each}
</div>
