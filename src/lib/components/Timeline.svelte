<script lang="ts">
  import type { Project, Session } from "../types";
  import { appLabel, formatTimestamp, parseDay } from "../format";

  type Track = {
    key: string;
    label: string;
    sessions: Session[];
  };

  let {
    sessions,
    projects,
    day,
    nowTs,
    trackState = "live",
    currentApp = null,
  }: {
    sessions: Session[];
    projects: Project[];
    day: string;
    nowTs: number;
    trackState?: "live" | "paused" | "idle" | "locked";
    currentApp?: string | null;
  } = $props();

  let hovered = $state<{ session: Session; x: number; y: number } | null>(null);

  let dayStart = $derived.by(() => {
    const date = parseDay(day);
    date.setHours(0, 0, 0, 0);
    return Math.floor(date.getTime() / 1000);
  });
  let dayEnd = $derived(dayStart + 86_400);

  let tracks = $derived.by((): Track[] => {
    const totals = new Map<string, number>();
    for (const session of sessions) {
      totals.set(session.app, (totals.get(session.app) ?? 0) + (session.endedAt - session.startedAt));
    }
    const ordered = [...totals.entries()].sort((a, b) => b[1] - a[1]);
    const top = ordered.slice(0, 3).map(([app]) => app);
    const groups = new Map<string, Session[]>();
    for (const app of top) groups.set(app, []);
    groups.set("__other", []);
    for (const session of sessions) {
      const key = top.includes(session.app) ? session.app : "__other";
      groups.get(key)?.push(session);
    }
    const result: Track[] = top.map((app) => ({
      key: app,
      label: appLabel(app),
      sessions: groups.get(app) ?? [],
    }));
    const other = groups.get("__other") ?? [];
    if (other.length > 0 || top.length === 0) {
      result.push({ key: "__other", label: top.length === 0 ? "No time" : "Other", sessions: other });
    }
    return result;
  });

  let nowFraction = $derived(
    Math.max(0, Math.min(1, (nowTs - dayStart) / (dayEnd - dayStart))),
  );

  let isToday = $derived(nowTs >= dayStart && nowTs < dayEnd);

  function fraction(ts: number): number {
    return Math.max(0, Math.min(1, (ts - dayStart) / (dayEnd - dayStart)));
  }

  function color(session: Session): string {
    if (session.projectId == null) return "var(--accent)";
    return projects.find((project) => project.id === session.projectId)?.color ?? "var(--accent)";
  }

  function label(session: Session): string {
    return appLabel(session.app);
  }

  function show(event: MouseEvent, session: Session) {
    hovered = { session, x: event.clientX, y: event.clientY - 12 };
  }
</script>

<div class="timeline" class:dimmed={trackState !== "live"}>
  <div class="body">
    <div class="labels">
      {#each tracks as track (track.key)}
        <div class="label" class:current={currentApp === track.key}>
          {#if currentApp === track.key && trackState === "live"}
            <span class="dot dot-live"></span>
          {/if}
          <span class="label-text">{track.label}</span>
        </div>
      {/each}
    </div>

    <div class="plot">
      {#each tracks as track (track.key)}
        <div class="track">
          {#each track.sessions as session (session.id)}
            {@const left = fraction(session.startedAt)}
            {@const right = fraction(session.endedAt)}
            <button
              class="pill"
              style:left="{left * 100}%"
              style:width="{Math.max(0.35, (right - left) * 100)}%"
              style:background={color(session)}
              onmouseenter={(event) => show(event, session)}
              onmouseleave={() => (hovered = null)}
              onfocus={() => {}}
              aria-label={`${label(session)} ${formatTimestamp(session.startedAt)} to ${formatTimestamp(session.endedAt)}`}
            ></button>
          {/each}
        </div>
      {/each}

      <div class="grid">
        {#each [0, 6, 12, 18] as hour}
          <span class="grid-line" style:left="{(hour / 24) * 100}%"></span>
        {/each}
      </div>

      {#if isToday && trackState === "live"}
        <div class="now" style:left="{nowFraction * 100}%">
          <span class="now-cap"></span>
        </div>
      {/if}
    </div>
  </div>

  <div class="axis">
    <div class="axis-spacer"></div>
    <div class="hours">
      {#each [0, 6, 12, 18] as hour}
        <span class="hour" style:left="{(hour / 24) * 100}%">
          {hour === 0 ? "12am" : hour === 12 ? "12pm" : `${hour}am`}
        </span>
      {/each}
    </div>
  </div>

  {#if hovered}
    <div class="tooltip fixed" style:left="{hovered.x}px" style:top="{hovered.y}px">
      <div>{label(hovered.session)}</div>
      <div class="tip-muted">
        {formatTimestamp(hovered.session.startedAt)}–{formatTimestamp(hovered.session.endedAt)}
        {#if hovered.session.projectName} · {hovered.session.projectName}{/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .timeline {
    width: 100%;
  }

  .body {
    display: flex;
    gap: 12px;
  }

  .labels {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 96px;
    flex: 0 0 96px;
  }

  .label {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 14px;
    color: var(--muted);
    font-size: 11.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .label.current {
    color: var(--text);
  }

  .label-text {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .plot {
    position: relative;
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .track {
    position: relative;
    height: 14px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--text) 5%, transparent);
  }

  .pill {
    position: absolute;
    top: 3px;
    height: 8px;
    min-width: 2px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    opacity: 0.9;
    transition:
      opacity 120ms ease-out,
      transform 120ms ease-out;
  }

  .pill:hover {
    opacity: 1;
    transform: scaleY(1.6);
  }

  .grid {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }

  .grid-line {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 0;
    border-left: 1px dashed var(--guide);
  }

  .now {
    position: absolute;
    top: -4px;
    bottom: -4px;
    width: 0;
    pointer-events: none;
  }

  .now::before {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    width: 1.5px;
    background: var(--accent);
    box-shadow: 0 0 8px color-mix(in srgb, var(--accent) 50%, transparent);
  }

  .now-cap {
    position: absolute;
    top: -2px;
    left: -2px;
    width: 5px;
    height: 5px;
    border-radius: 999px;
    background: var(--accent);
  }

  .dimmed .pill {
    opacity: 0.35;
    filter: grayscale(0.5);
  }

  .dimmed .now {
    opacity: 0.4;
  }

  .axis {
    display: flex;
    gap: 12px;
    margin-top: 8px;
  }

  .axis-spacer {
    width: 96px;
    flex: 0 0 96px;
  }

  .hours {
    position: relative;
    flex: 1;
    height: 14px;
  }

  .hour {
    position: absolute;
    top: 0;
    color: var(--muted-2);
    font-size: 10px;
    transform: translateX(-50%);
    white-space: nowrap;
  }

  .hour:first-child {
    transform: none;
  }

  @media (max-width: 700px) {
    .labels,
    .axis-spacer {
      width: 74px;
      flex-basis: 74px;
    }
    .label {
      font-size: 10.5px;
    }
  }
</style>
