<script lang="ts">
  import type { Snippet } from "svelte";
  import { fade, fly } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import { motionDuration } from "../motion";

  let {
    open = false,
    title,
    width = 460,
    onclose,
    children,
    footer,
  }: {
    open?: boolean;
    title: string;
    width?: number;
    onclose: () => void;
    children: Snippet;
    footer?: Snippet;
  } = $props();

  function onKey(event: KeyboardEvent) {
    if (open && event.key === "Escape") {
      event.stopPropagation();
      onclose();
    }
  }
</script>

<svelte:window onkeydown={onKey} />

{#if open}
  <div
    class="modal-backdrop"
    transition:fade={{ duration: motionDuration(120) }}
    role="presentation"
    onclick={(event) => {
      if (event.target === event.currentTarget) onclose();
    }}
  >
    <div
      class="modal"
      style:max-width="{width}px"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      transition:fly={{ y: 4, duration: motionDuration(140) }}
    >
      <div class="modal-head">
        <h2 class="modal-title">{title}</h2>
        <button class="btn btn-ghost btn-icon btn-sm" onclick={onclose} aria-label="Close">
          <Icon name="x" size={14} />
        </button>
      </div>
      <div class="modal-body">
        {@render children()}
      </div>
      {#if footer}
        <div class="modal-foot">
          {@render footer()}
        </div>
      {/if}
    </div>
  </div>
{/if}
