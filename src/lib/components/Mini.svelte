<script lang="ts">
  import Icon from "./Icon.svelte";
  import { activeProject, status, togglePaused } from "../stores";
  import { api } from "../api";
  import { appLabel, formatDuration, formatHMS } from "../format";

  let clock = $state(Math.floor(Date.now() / 1000));
  let closing = $state(false);

  let live = $derived(
    $status?.trackingSince ? Math.max(0, clock - $status.trackingSince) : 0,
  );
  let isLive = $derived(
    !!$status &&
      !$status.paused &&
      !$status.locked &&
      !$status.idle &&
      !!$status.trackingSince &&
      !!$status.currentApp,
  );
  let today = $derived($status?.todaySeconds ?? 0);

  $effect(() => {
    const id = window.setInterval(() => {
      if (document.visibilityState === "visible") {
        clock = Math.floor(Date.now() / 1000);
      }
    }, 1000);
    return () => window.clearInterval(id);
  });

  function close() {
    if (closing) return;
    closing = true;
    window.setTimeout(() => void api.closeMiniWindow(), 190);
  }

  function expand() {
    void api.openFullApp();
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === "Escape") close();
  }}
/>

<div class="mini" class:closing>
  <div class="drag" data-tauri-drag-region></div>

  <div class="top">
    {#if $activeProject}
      <span class="dot" style:background={$activeProject.color}></span>
    {:else}
      <span class="dot empty"></span>
    {/if}
    <span class="project" data-tauri-drag-region>
      {$activeProject?.name ?? "No project"}
    </span>
    <span class="spacer"></span>
    <button
      class="mini-btn"
      onclick={() => void togglePaused()}
      title={$status?.paused ? "Resume" : "Pause"}
      aria-label={$status?.paused ? "Resume" : "Pause"}
    >
      <Icon name={$status?.paused ? "play" : "pause"} size={14} />
    </button>
    <button class="mini-btn" onclick={expand} title="Open Framewise" aria-label="Open Framewise">
      <Icon name="arrow-up-right" size={14} />
    </button>
    <button class="mini-btn" onclick={close} title="Close" aria-label="Close">
      <Icon name="x" size={14} />
    </button>
  </div>

  <div class="body" data-tauri-drag-region>
    <div class="time">{isLive ? formatHMS(live) : formatDuration(today)}</div>
    <div class="sub">
      {#if isLive}
        {appLabel($status?.currentApp ?? "")} · {formatDuration(today)} today
      {:else if $status?.paused}
        Paused
      {:else if $status?.locked}
        Locked
      {:else if $status?.idle}
        Idle
      {:else}
        Waiting
      {/if}
    </div>
  </div>
</div>

<style>
  :global(body) {
    background: var(--surface);
    overflow: hidden;
  }

  .mini {
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 0 13px 11px;
    background: var(--surface);
    border: 1px solid var(--border);
    box-sizing: border-box;
    transform-origin: 100% 100%;
    animation: mini-in 360ms cubic-bezier(0.22, 1, 0.36, 1) both;
    will-change: transform, opacity;
  }

  .mini.closing {
    animation: mini-out 190ms ease-in both;
  }

  @keyframes mini-in {
    from {
      opacity: 0;
      transform: translateY(12px) scale(0.92);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @keyframes mini-out {
    to {
      opacity: 0;
      transform: translateY(8px) scale(0.96);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .mini,
    .mini.closing {
      animation: none;
    }
  }

  .drag {
    height: 11px;
    flex: 0 0 auto;
  }

  .top {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 28px;
    flex: 0 0 auto;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    flex: 0 0 auto;
  }

  .dot.empty {
    border: 1px solid var(--muted-2);
  }

  .project {
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12.5px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .spacer {
    flex: 1;
  }

  .mini-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--muted);
    transition:
      background-color 140ms ease-out,
      color 140ms ease-out;
  }

  .mini-btn:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .body {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding-left: 1px;
    min-height: 0;
  }

  .time {
    font-size: 34px;
    font-weight: 600;
    letter-spacing: -0.045em;
    line-height: 1;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .sub {
    margin-top: 4px;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--muted);
    font-size: 11.5px;
  }
</style>
