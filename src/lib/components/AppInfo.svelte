<script lang="ts">
  import { appInfo } from "../stores";
  import { appDescription, appLabel } from "../format";
  import { api } from "../api";
  import AppIcon from "./AppIcon.svelte";
  import Icon from "./Icon.svelte";

  let details = $state<{ path: string | null } | null>(null);
  let loading = $state(false);
  let token = 0;

  $effect(() => {
    const app = $appInfo;
    const current = ++token;
    details = null;
    if (!app) {
      loading = false;
      return;
    }
    loading = true;
    api
      .appDetails(app)
      .then((result) => {
        if (current === token) details = result;
      })
      .catch(() => {})
      .finally(() => {
        if (current === token) loading = false;
      });
  });

  function close() {
    appInfo.set(null);
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === "Escape" && $appInfo) close();
  }}
/>

{#if $appInfo}
  <div
    class="app-info-backdrop"
    role="presentation"
    onclick={(event) => {
      if (event.target === event.currentTarget) close();
    }}
  >
    <div class="app-info" role="dialog" aria-modal="true" aria-label="Application details">
      <div class="app-info-head">
        <AppIcon app={$appInfo} size={40} />
        <div class="app-info-title">
          <div class="app-info-name">{appLabel($appInfo)}</div>
          <div class="app-info-exe">{$appInfo}</div>
        </div>
        <button class="app-info-close" onclick={close} aria-label="Close">
          <Icon name="x" size={14} />
        </button>
      </div>

      <div class="app-info-desc">{appDescription($appInfo)}</div>

      <div class="app-info-path">
        {#if loading}
          <span class="dim">Looking up the executable…</span>
        {:else if details?.path}
          <span class="path">{details.path}</span>
        {:else}
          <span class="dim">Not running right now. This name comes from your history.</span>
        {/if}
      </div>

      <p class="app-info-note">
        Framewise records executable names, never window titles.
      </p>
    </div>
  </div>
{/if}

<style>
  .app-info-backdrop {
    position: fixed;
    inset: 0;
    z-index: 110;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 14vh 20px 40px;
    background: var(--overlay);
    backdrop-filter: blur(3px);
  }

  .app-info {
    width: min(470px, 100%);
    padding: 20px;
    border: 1px solid var(--border);
    border-radius: 22px;
    background: var(--bg);
    box-shadow: var(--shadow);
    animation: app-info-in 180ms cubic-bezier(0.22, 1, 0.36, 1) both;
  }

  @keyframes app-info-in {
    from {
      opacity: 0;
      transform: translateY(6px) scale(0.98);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }

  .app-info-head {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .app-info-title {
    flex: 1;
    min-width: 0;
  }

  .app-info-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 18px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  .app-info-exe {
    margin-top: 1px;
    color: var(--muted);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .app-info-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--muted);
    transition:
      background-color 140ms ease,
      color 140ms ease;
  }

  .app-info-close:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .app-info-desc {
    margin-top: 18px;
    font-size: 14px;
    color: var(--text);
  }

  .app-info-path {
    margin-top: 10px;
    padding: 10px 12px;
    border-radius: 12px;
    background: var(--surface);
    color: var(--muted);
    font-size: 12px;
    line-height: 1.5;
    word-break: break-all;
  }

  .app-info-path .path {
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .app-info-note {
    margin-top: 14px;
    color: var(--muted-2);
    font-size: 11.5px;
    line-height: 1.5;
  }

  .dim {
    color: var(--muted-2);
  }
</style>
