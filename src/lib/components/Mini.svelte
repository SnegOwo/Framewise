<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
  import Icon from "./Icon.svelte";
  import { activeProject, status, togglePaused } from "../stores";
  import { api } from "../api";
  import { appLabel, formatDuration, formatHMS } from "../format";

  const MIN_W = 150;
  const MIN_H = 64;
  const MAX_W = 640;
  const MAX_H = 420;
  const COMPACT_W = 280;
  const COMPACT_H = 108;
  const SIZE_KEY = "framewise-mini-size";

  let clock = $state(Math.floor(Date.now() / 1000));
  let closing = $state(false);
  let pinned = $state(false);
  let compact = $state(false);
  let resizing = false;
  let startX = 0;
  let startY = 0;
  let startW = 0;
  let startH = 0;
  let lastW = 0;
  let lastH = 0;

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

  onMount(() => {
    try {
      pinned = localStorage.getItem("framewise-mini-pinned") === "1";
    } catch {
      pinned = false;
    }
    void api.setMiniPinned(pinned);

    compact = isCompact(window.innerWidth, window.innerHeight);
    void restoreSize();

    const onResize = () => {
      if (window.innerWidth <= 1 || window.innerHeight <= 1) return;
      compact = isCompact(window.innerWidth, window.innerHeight);
    };
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  });

  function isCompact(width: number, height: number): boolean {
    return width < COMPACT_W || height < COMPACT_H;
  }

  function clampSize(width: number, height: number): { width: number; height: number } {
    return {
      width: Math.min(MAX_W, Math.max(MIN_W, Math.round(width))),
      height: Math.min(MAX_H, Math.max(MIN_H, Math.round(height))),
    };
  }

  function readSavedSize(): { width: number; height: number } | null {
    try {
      const raw = localStorage.getItem(SIZE_KEY);
      if (!raw) return null;
      const parsed = JSON.parse(raw) as { width?: unknown; height?: unknown };
      if (typeof parsed.width !== "number" || typeof parsed.height !== "number") return null;
      if (!Number.isFinite(parsed.width) || !Number.isFinite(parsed.height)) return null;
      return clampSize(parsed.width, parsed.height);
    } catch {
      return null;
    }
  }

  function saveSize(width: number, height: number) {
    try {
      localStorage.setItem(SIZE_KEY, JSON.stringify(clampSize(width, height)));
    } catch {
      // ignore
    }
  }

  // Restore the last size and keep the window anchored to its bottom-right
  // corner, so a tiny timer stays next to the tray instead of drifting.
  async function restoreSize() {
    const saved = readSavedSize();
    if (!saved) return;
    compact = isCompact(saved.width, saved.height);
    try {
      const win = getCurrentWindow();
      const position = await win.outerPosition();
      const outer = await win.outerSize();
      await win.setSize(new LogicalSize(saved.width, saved.height));
      // Outer size includes the invisible resize border, so re-measure it
      // instead of assuming inner size == outer size.
      await new Promise((resolve) => setTimeout(resolve, 30));
      const resizedOuter = await win.outerSize();
      const scale = await win.scaleFactor();
      const right = position.x + outer.width;
      const bottom = position.y + outer.height;
      await win.setPosition(
        new LogicalPosition(
          Math.round((right - resizedOuter.width) / scale),
          Math.round((bottom - resizedOuter.height) / scale),
        ),
      );
    } catch {
      // Older builds may not grant the position permission; size still applies.
    }
  }

  function togglePin() {
    pinned = !pinned;
    try {
      localStorage.setItem("framewise-mini-pinned", pinned ? "1" : "0");
    } catch {
      // ignore
    }
    void api.setMiniPinned(pinned);
  }

  function close() {
    if (closing) return;
    closing = true;
    window.setTimeout(() => void api.closeMiniWindow(), 190);
  }

  function expand() {
    void api.openFullApp();
  }

  function startResize(event: PointerEvent) {
    event.preventDefault();
    event.stopPropagation();
    resizing = true;
    startX = event.screenX;
    startY = event.screenY;
    startW = window.innerWidth;
    startH = window.innerHeight;
    lastW = startW;
    lastH = startH;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function moveResize(event: PointerEvent) {
    if (!resizing) return;
    const size = clampSize(startW + event.screenX - startX, startH + event.screenY - startY);
    lastW = size.width;
    lastH = size.height;
    compact = isCompact(size.width, size.height);
    void getCurrentWindow().setSize(new LogicalSize(size.width, size.height));
  }

  function endResize(event: PointerEvent) {
    resizing = false;
    if (lastW > 0 && lastH > 0) saveSize(lastW, lastH);
    try {
      (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
    } catch {
      // ignore
    }
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === "Escape") close();
  }}
/>

<div class="mini" class:closing class:compact>
  <div class="drag" data-tauri-drag-region={pinned ? "" : undefined}></div>

  <div class="top">
    {#if $activeProject}
      <span class="dot" style:background={$activeProject.color}></span>
    {:else}
      <span class="dot empty"></span>
    {/if}
    <span class="project" data-tauri-drag-region={pinned ? "" : undefined}>
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
    <button
      class="mini-pin"
      class:pinned
      onclick={togglePin}
      title={pinned ? "Pinned — drag anywhere" : "Pin to keep open and drag"}
      aria-pressed={pinned}
      aria-label={pinned ? "Unpin mini timer" : "Pin mini timer to keep open and drag"}
    >
      <Icon name="pin" size={12} />
      <span>{pinned ? "Pinned" : "Pin"}</span>
    </button>
    <button class="mini-btn" onclick={expand} title="Open Framewise" aria-label="Open Framewise">
      <Icon name="arrow-up-right" size={14} />
    </button>
    <button class="mini-btn" onclick={close} title="Close" aria-label="Close">
      <Icon name="x" size={14} />
    </button>
  </div>

  <div class="body" data-tauri-drag-region={pinned ? "" : undefined}>
    <div class="time" data-tauri-drag-region={pinned ? "" : undefined}>{isLive ? formatHMS(live) : formatDuration(today)}</div>
    <div class="sub" data-tauri-drag-region={pinned ? "" : undefined}>
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
    <div class="pin-note" class:pinned data-tauri-drag-region={pinned ? "" : undefined}>
      {pinned ? "Pinned · drag anywhere" : "Pin to drag · corner to resize"}
    </div>
  </div>

  <button
    class="resize-grip"
    class:pinned
    type="button"
    onpointerdown={startResize}
    onpointermove={moveResize}
    onpointerup={endResize}
    onpointercancel={endResize}
    title="Resize"
    aria-label="Resize mini timer"
  >
    <svg width="16" height="16" viewBox="0 0 14 14" fill="none" aria-hidden="true">
      <path
        d="M12 2 2 12M12 7 7 12M12 10 10 12"
        stroke="currentColor"
        stroke-width="1.2"
        stroke-linecap="round"
      />
    </svg>
  </button>
</div>

<style>
  :global(body) {
    background: var(--surface);
    overflow: hidden;
  }

  .mini {
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 0 13px 11px;
    background: var(--surface);
    border: 1px solid var(--border);
    box-sizing: border-box;
    transform-origin: 100% 100%;
    animation: mini-pop 230ms cubic-bezier(0.16, 1, 0.3, 1) both;
  }

  .mini.closing {
    animation: mini-out 180ms ease-in both;
  }

  /* Time-only mode: below 280×108 the timer sheds everything except the
     clock, and the controls become a hover overlay. */
  .mini.compact {
    padding: 0 10px;
  }

  .mini.compact .drag {
    display: none;
  }

  .mini.compact .top {
    position: absolute;
    top: 4px;
    right: 4px;
    left: 4px;
    z-index: 3;
    height: 24px;
    gap: 2px;
    padding: 0 3px;
    border-radius: 8px;
    background: color-mix(in srgb, var(--surface) 84%, transparent);
    opacity: 0;
    pointer-events: none;
    transition: opacity 140ms ease;
  }

  .mini.compact:hover .top,
  .mini.compact:focus-within .top {
    opacity: 1;
    pointer-events: auto;
  }

  .mini.compact .top .dot,
  .mini.compact .top .project,
  .mini.compact .top .spacer {
    display: none;
  }

  .mini.compact .mini-pin {
    width: 24px;
    padding: 0;
    justify-content: center;
  }

  .mini.compact .mini-pin span {
    display: none;
  }

  .mini.compact .body {
    align-items: center;
    justify-content: center;
    padding-left: 0;
    text-align: center;
  }

  .mini.compact .sub,
  .mini.compact .pin-note {
    display: none;
  }

  .mini.compact .time {
    font-size: clamp(16px, min(34vh, 19vw), 40px);
    letter-spacing: -0.04em;
  }

  .mini.compact .resize-grip {
    right: 2px;
    bottom: 2px;
    width: 18px;
    height: 18px;
    opacity: 0;
  }

  .mini.compact:hover .resize-grip,
  .mini.compact:focus-within .resize-grip {
    opacity: 0.75;
  }

  @keyframes mini-pop {
    from {
      opacity: 0;
      transform: scale(0.92);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  @keyframes mini-out {
    to {
      opacity: 0;
      transform: scale(0.96);
    }
  }

  [data-tauri-drag-region] {
    cursor: grab;
    user-select: none;
  }

  [data-tauri-drag-region]:active {
    cursor: grabbing;
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
    max-width: min(180px, 42vw);
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

  .mini-pin {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 8px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: transparent;
    color: var(--muted);
    font-size: 10.5px;
    font-weight: 600;
    white-space: nowrap;
    transition:
      background-color 140ms ease,
      border-color 140ms ease,
      color 140ms ease;
  }

  .mini-pin:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .mini-pin.pinned {
    border-color: var(--accent-line);
    background: var(--accent-soft);
    color: var(--accent-text);
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
    font-size: clamp(32px, 22vh, 54px);
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

  .pin-note {
    margin-top: 5px;
    color: var(--muted-2);
    font-size: 10px;
    letter-spacing: -0.005em;
  }

  .pin-note.pinned {
    color: var(--accent-text);
  }

  .resize-grip {
    position: absolute;
    right: 4px;
    bottom: 4px;
    z-index: 2;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 0;
    border-radius: 7px;
    background: color-mix(in srgb, var(--surface-3) 55%, transparent);
    color: var(--muted-2);
    opacity: 0.55;
    cursor: nwse-resize;
    touch-action: none;
    transition:
      opacity 140ms ease,
      color 140ms ease,
      background-color 140ms ease;
  }

  .resize-grip.pinned {
    opacity: 0.8;
  }

  .resize-grip.pinned:hover {
    background: var(--surface-3);
    color: var(--text);
    opacity: 1;
  }

  .resize-grip:disabled {
    opacity: 0.2;
    cursor: default;
  }
</style>
