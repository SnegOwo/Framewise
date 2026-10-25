<script lang="ts">
  import { get } from "svelte/store";
  import { onMount } from "svelte";
  import type { Session } from "../types";
  import { api, errMessage } from "../api";
  import {
    activeProjects,
    boot,
    bumpRefresh,
    loadBootstrap,
    refreshTick,
    showAppInfo,
    toast,
  } from "../stores";
  import { sessionsDay, sessionsProject } from "../router";
  import {
    appLabel,
    daysAgo,
    formatDate,
    formatDateLong,
    formatDuration,
    formatTimestamp,
    shiftDay,
    todayStr,
  } from "../format";
  import AppIcon from "../components/AppIcon.svelte";
  import Icon from "../components/Icon.svelte";
  import SessionModal from "../components/SessionModal.svelte";

  let today = $derived($boot?.today ?? todayStr());
  let filterDay = $state<string | null>(get(sessionsDay));
  let filterProjectId = $state<number | null>(get(sessionsProject));
  let sessions = $state<Session[]>([]);
  let loading = $state(true);
  let error = $state("");
  let reassignId = $state<number | null>(null);
  let editorOpen = $state(false);
  let editing = $state<Session | null>(null);

  onMount(() => {
    sessionsDay.set(null);
    sessionsProject.set(null);
  });

  let rangeStart = $derived(filterDay ?? daysAgo(89, today));
  let rangeEnd = $derived(filterDay ?? today);

  let groups = $derived.by(() => {
    const map = new Map<string, Session[]>();
    for (const session of sessions) {
      const list = map.get(session.day) ?? [];
      list.push(session);
      map.set(session.day, list);
    }
    return [...map.entries()]
      .sort((a, b) => b[0].localeCompare(a[0]))
      .map(([day, list]) => ({
        day,
        sessions: [...list].sort((a, b) => b.startedAt - a.startedAt),
        total: list.reduce((sum, item) => sum + (item.endedAt - item.startedAt), 0),
      }));
  });

  let count = $derived(sessions.length);

  async function load() {
    loading = true;
    try {
      sessions = await api.sessions({
        startDay: rangeStart,
        endDay: rangeEnd,
        projectId: filterProjectId,
        limit: 2000,
      });
      error = "";
    } catch (cause) {
      error = errMessage(cause);
    } finally {
      loading = false;
    }
  }

  async function reassign(session: Session, projectId: number | null) {
    reassignId = null;
    try {
      await api.reassignSessions([session.id], projectId);
      await loadBootstrap();
      await load();
      bumpRefresh();
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  async function remove(session: Session) {
    if (!window.confirm("Delete this session?")) return;
    try {
      await api.deleteSession(session.id);
      await load();
      await loadBootstrap();
      bumpRefresh();
      toast("Session deleted.");
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  function edit(session: Session) {
    editing = session;
    editorOpen = true;
  }

  function addSession() {
    editing = null;
    editorOpen = true;
  }

  function projectFor(session: Session) {
    return $boot?.projects.find((project) => project.id === session.projectId) ?? null;
  }

  function dayLabel(day: string): string {
    if (day === today) return "Today";
    if (day === shiftDay(today, -1)) return "Yesterday";
    return formatDateLong(day);
  }

  $effect(() => {
    $refreshTick;
    filterDay;
    filterProjectId;
    void load();
  });

  function onWindowClick(event: MouseEvent) {
    if (reassignId == null) return;
    const target = event.target as HTMLElement;
    if (!target.closest(".chip-wrap")) reassignId = null;
  }
</script>

<svelte:window onclick={onWindowClick} />

<div class="page sessions">
  <header class="page-head reveal">
    <div>
      <div class="eyebrow">Sessions</div>
      <h1 class="section-heading">The log</h1>
    </div>
    <button class="btn btn-sm" onclick={addSession}>
      <Icon name="plus" size={14} /> Add session
    </button>
  </header>

  {#if filterDay || filterProjectId != null}
    <div class="filter-row">
      <span class="filter-label">Showing</span>
      {#if filterDay}
        <button class="filter-chip" onclick={() => (filterDay = null)}>
          {formatDate(filterDay)} <Icon name="x" size={12} />
        </button>
      {/if}
      {#if filterProjectId != null}
        <button class="filter-chip" onclick={() => (filterProjectId = null)}>
          {$activeProjects.find((project) => project.id === filterProjectId)?.name ?? "Project"}
          <Icon name="x" size={12} />
        </button>
      {/if}
    </div>
  {/if}

  {#if error}
    <div class="inline-error">
      <span>{error}</span>
      <button class="link" onclick={() => void load()}>Retry</button>
    </div>
  {/if}

  {#if loading && sessions.length === 0}
    <div class="loading-line"><span class="spin"></span> Loading sessions</div>
  {:else if count === 0}
    <div class="empty-block">
      <div class="empty-title">No sessions in this range</div>
      <div class="empty-sub">Time appears as soon as you switch into a tracked app.</div>
    </div>
  {:else}
    {#each groups as group (group.day)}
      <section class="day-group">
        <div class="day-head">
          <span class="day-title">{dayLabel(group.day)}</span>
          <span class="day-date">{formatDateLong(group.day)}</span>
          <span class="day-total">{formatDuration(group.total)}</span>
        </div>

        <div class="session-list">
          {#each group.sessions as session (session.id)}
            <div class="session-row">
              <span class="time">
                {formatTimestamp(session.startedAt)}–{formatTimestamp(session.endedAt)}
              </span>
              <button
                class="app-open"
                onclick={() => showAppInfo(session.app)}
                title={`Inspect ${appLabel(session.app)}`}
              >
                <AppIcon app={session.app} size={24} tint={projectFor(session)?.color ?? null} />
                <span class="app">{appLabel(session.app)}</span>
              </button>

              <span class="chip-wrap">
                <button
                  class="chip sm project-chip"
                  onclick={() => (reassignId = reassignId === session.id ? null : session.id)}
                  title="Reassign project"
                >
                  {#if projectFor(session)}
                    <span class="dot" style:background={projectFor(session)?.color}></span>
                    {projectFor(session)?.name}
                  {:else}
                    <span class="dot empty"></span>
                    <span class="muted">Unassigned</span>
                  {/if}
                </button>
                {#if reassignId === session.id}
                  <div class="menu reassign-menu">
                    <button class="menu-item" onclick={() => reassign(session, null)}>
                      <span class="dot empty"></span> Unassigned
                    </button>
                    <div class="menu-sep"></div>
                    {#each $activeProjects as project (project.id)}
                      <button class="menu-item" onclick={() => reassign(session, project.id)}>
                        <span class="dot" style:background={project.color}></span>
                        <span class="menu-name">{project.name}</span>
                        {#if session.projectId === project.id}
                          <span class="check"><Icon name="check" size={13} /></span>
                        {/if}
                      </button>
                    {/each}
                  </div>
                {/if}
              </span>

              <span class="duration">{formatDuration(session.endedAt - session.startedAt)}</span>

              <span class="row-actions">
                <button class="icon-button" onclick={() => edit(session)} title="Edit">
                  <Icon name="pencil" size={14} />
                </button>
                <button class="icon-button danger" onclick={() => remove(session)} title="Delete">
                  <Icon name="trash" size={14} />
                </button>
              </span>
            </div>
          {/each}
        </div>
      </section>
    {/each}
  {/if}
</div>

<SessionModal
  open={editorOpen}
  session={editing}
  defaultDay={filterDay ?? today}
  onclose={() => {
    editorOpen = false;
    editing = null;
    void load();
    bumpRefresh();
  }}
/>

<style>
  .page-head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
    padding-top: 64px;
    margin-bottom: 22px;
  }

  .filter-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 18px;
  }

  .filter-label {
    color: var(--muted);
    font-size: 12.5px;
  }

  .filter-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 28px;
    padding: 0 10px;
    border: 1px dashed var(--border-strong);
    border-radius: var(--r-chip);
    background: transparent;
    color: var(--text);
    font-size: 12.5px;
  }

  .filter-chip:hover {
    background: var(--surface-3);
  }

  .day-group {
    margin-top: 30px;
  }

  .day-head {
    position: sticky;
    top: 0;
    z-index: 5;
    display: flex;
    align-items: baseline;
    gap: 10px;
    padding: 12px 0 10px;
    background: var(--bg);
    border-bottom: 1px solid var(--border-subtle);
  }

  .day-title {
    font-size: 15px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .day-date {
    color: var(--muted);
    font-size: 12.5px;
  }

  .day-total {
    margin-left: auto;
    color: var(--muted);
    font-size: 13px;
    font-variant-numeric: tabular-nums;
  }

  .session-row {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 50px;
    padding: 6px 4px;
    border-bottom: 1px solid var(--border-subtle);
    transition: background-color 140ms ease-out;
  }

  .session-row:hover {
    background: var(--surface);
  }

  .time {
    width: 108px;
    flex: 0 0 108px;
    color: var(--muted);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .app-open {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 186px;
    flex: 0 0 186px;
    min-width: 0;
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
  }

  .app-open:hover .app {
    color: var(--accent-text);
  }

  .app {
    min-width: 0;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 14px;
  }

  .chip-wrap {
    position: relative;
    min-width: 0;
    flex: 1;
  }

  .project-chip {
    height: 26px;
    padding: 0 8px;
    font-size: 12px;
  }

  .reassign-menu {
    top: calc(100% + 4px);
    left: 0;
    width: 220px;
  }

  .menu-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .check {
    display: flex;
    margin-left: auto;
    color: var(--accent-text);
  }

  .duration {
    width: 66px;
    flex: 0 0 66px;
    text-align: right;
    color: var(--text);
    font-size: 13px;
    font-variant-numeric: tabular-nums;
  }

  .row-actions {
    display: flex;
    gap: 0;
    width: 54px;
    flex: 0 0 54px;
    justify-content: flex-end;
    opacity: 0;
    transition: opacity 150ms ease-out;
  }

  .session-row:hover .row-actions,
  .session-row:focus-within .row-actions {
    opacity: 1;
  }

  .icon-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--muted-2);
  }

  .icon-button:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .icon-button.danger:hover {
    color: var(--danger);
    background: var(--danger-soft);
  }

  .inline-error {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 18px;
    padding: 10px 12px;
    border-radius: var(--r-control);
    background: var(--danger-soft);
    color: var(--danger);
    font-size: 13px;
  }

  .inline-error span {
    flex: 1;
  }

  .loading-line {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 30px 0;
    color: var(--muted);
    font-size: 13px;
  }

  .empty-block {
    padding: 40px 0;
  }

  @media (max-width: 780px) {
    .time {
      width: 92px;
      flex-basis: 92px;
      font-size: 11.5px;
    }
    .app-open {
      width: 128px;
      flex-basis: 128px;
    }
  }
</style>
