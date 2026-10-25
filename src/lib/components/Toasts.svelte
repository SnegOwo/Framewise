<script lang="ts">
  import { fly } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import { dismissToast, toasts } from "../stores";
  import { motionDuration } from "../motion";
</script>

<div class="toast-stack" aria-live="polite" aria-atomic="false">
  {#each $toasts as toast (toast.id)}
    <button
      class="toast"
      class:success={toast.tone === "success"}
      class:error={toast.tone === "error"}
      transition:fly={{ x: 10, y: 4, duration: motionDuration(150) }}
      onclick={() => dismissToast(toast.id)}
      title="Dismiss"
    >
      <span class="toast-dot"></span>
      <span class="toast-message">{toast.message}</span>
      <Icon name="x" size={12} />
    </button>
  {/each}
</div>

<style>
  .toast-stack {
    position: fixed;
    right: 18px;
    bottom: 84px;
    z-index: 90;
    display: flex;
    flex-direction: column-reverse;
    align-items: flex-end;
    gap: 6px;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 8px;
    max-width: 380px;
    min-height: 38px;
    padding: 9px 12px;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--surface);
    box-shadow: var(--shadow);
    color: var(--text);
    font-size: 13px;
    text-align: left;
    pointer-events: auto;
    animation: toast-in 150ms ease-out;
  }

  .toast:hover {
    background: var(--surface-3);
  }
  .toast-dot {
    width: 6px;
    height: 6px;
    flex: 0 0 auto;
    border-radius: 999px;
    background: var(--accent);
  }

  .toast.success .toast-dot {
    background: var(--green);
  }

  .toast.error .toast-dot {
    background: var(--danger);
  }

  .toast-message {
    flex: 1;
  }

  .toast :global(svg) {
    color: var(--muted-2);
  }

  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
