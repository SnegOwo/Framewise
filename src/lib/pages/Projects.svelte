<script lang="ts">
  import { onMount } from "svelte";
  import type { Project, ProjectDetail } from "../types";
  import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
  import { api, errMessage } from "../api";
  import AppIcon from "../components/AppIcon.svelte";
  import {
    activeProject,
    boot,
    bumpRefresh,
    loadBootstrap,
    openProjectSetup,
    refreshTick,
    status,
    switchProject,
    toast,
  } from "../stores";
  import { navigate, sessionsProject } from "../router";
  import { newProjectSignal } from "../router";
  import {
    clampPercent,
    daysAgo,
    fillDays,
    appLabel,
    formatDuration,
    formatRange,
    todayStr,
    weekBounds,
  } from "../format";
  import Icon from "../components/Icon.svelte";
  import Rule from "../components/Rule.svelte";
  import Heatmap from "../components/Heatmap.svelte";

  const palette = [
    "#7dd3fc",
    "#93c5fd",
    "#a5b4fc",
    "#c4b5fd",
    "#f0abfc",
    "#f9a8d4",
    "#fca5a5",
    "#fdba74",
    "#fde047",
    "#86efac",
    "#5eead4",
    "#94a3b8",
  ];

  let adding = $state(false);
  let newName = $state("");
  let newClient = $state("");
  let newColor = $state(palette[0]);
  let creating = $state(false);
  let menuId = $state<number | null>(null);

  let renameId = $state<number | null>(null);
  let renameValue = $state("");

  let detailId = $state<number | null>(null);
  let detail = $state<ProjectDetail | null>(null);
  let detailHeat = $state<{ day: string; seconds: number }[]>([]);
  let detailLoading = $state(false);
  let detailRequest = 0;

  let today = $derived($boot?.today ?? todayStr());
  let week = $derived(weekBounds(today));
  let activeList = $derived(($boot?.projects ?? []).filter((project) => !project.archived));
  let archivedList = $derived(($boot?.projects ?? []).filter((project) => project.archived));

  let rows = $derived.by(() =>
    activeList.map((project) => {
      const seconds =
        $status?.weekByProject.find((row) => row.projectId === project.id)?.seconds ?? 0;
      const goal = ($boot?.goals ?? []).find(
        (item) => item.scope === "project" && item.projectId === project.id,
      );
      const target = goal?.targetSeconds ?? 0;
      const percent = target > 0 ? (seconds / target) * 100 : 0;
      return {
        project,
        seconds,
        target,
        percent,
        active: $activeProject?.id === project.id,
      };
    }),
  );

  let totalWeek = $derived(rows.reduce((sum, row) => sum + row.seconds, 0));

  let detailDays = $derived(
    detail ? fillDays(daysAgo(29, today), today, detail.perDay) : [],
  );
  let detailMax = $derived(
    Math.max(...detailDays.map((row) => row.seconds), 1),
  );
  let detailAppMax = $derived(
    Math.max(...(detail?.perApp ?? []).map((row) => row.seconds), 1),
  );
  let focusTotal = $derived(
    (detail?.focusedSeconds ?? 0) + (detail?.unfocusedSeconds ?? 0),
  );
  let focusPercent = $derived(
    focusTotal > 0
      ? Math.round(((detail?.focusedSeconds ?? 0) / focusTotal) * 100)
      : 0,
  );
  let focusedApps = $derived(
    (detail?.perApp ?? []).filter((row) => row.focused === true),
  );
  let otherApps = $derived(
    (detail?.perApp ?? []).filter((row) => row.focused !== true),
  );
  let toolSuggestions = $derived.by(() => {
    if (!detail) return [];
    const tools = new Set(detail.tools.map((tool) => tool.toLowerCase()));
    const seen = new Set<string>();
    return detail.perApp
      .map((row) => row.app)
      .filter((app) => {
        const key = app.toLowerCase();
        if (tools.has(key) || seen.has(key)) return false;
        seen.add(key);
        return true;
      })
      .slice(0, 4);
  });
  let toolBusy = $state(false);

  async function refresh() {
    await loadBootstrap();
    bumpRefresh();
  }

  async function create() {
    if (!newName.trim()) {
      toast("Give the project a name.", "error");
      return;
    }
    creating = true;
    try {
      const project = await api.createProject(newName.trim(), newClient.trim() || null, newColor);
      await refresh();
      adding = false;
      newName = "";
      newClient = "";
      newColor = palette[0];
      await switchProject(project.id);
      openProjectSetup(project, true);
    } catch (cause) {
      toast(errMessage(cause), "error");
    } finally {
      creating = false;
    }
  }

  function startRename(project: Project) {
    menuId = null;
    renameId = project.id;
    renameValue = project.name;
  }

  async function commitRename(project: Project) {
    const name = renameValue.trim();
    renameId = null;
    if (!name || name === project.name) return;
    try {
      await api.updateProject(
        project.id,
        name,
        project.client,
        project.color,
        project.archived,
      );
      await refresh();
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  async function toggleArchive(project: Project) {
    menuId = null;
    try {
      await api.updateProject(
        project.id,
        project.name,
        project.client,
        project.color,
        !project.archived,
      );
      await refresh();
      if (detailId === project.id) await refreshDetail(project.id);
      toast(project.archived ? "Project reopened." : "Project finished.");
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  async function remove(project: Project) {
    menuId = null;
    if (!window.confirm(`Delete “${project.name}”? Its sessions stay, unassigned.`)) return;
    try {
      await api.deleteProject(project.id);
      await refresh();
      if (detailId === project.id) detailId = null;
      toast("Project deleted.");
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  async function openDetail(project: Project) {
    detailId = project.id;
    detailLoading = true;
    await refreshDetail(project.id);
    detailLoading = false;
  }

  async function refreshDetail(id: number) {
    const request = ++detailRequest;
    try {
      const [next, heat] = await Promise.all([
        api.projectDetail(id),
        api.heatmap(daysAgo(181, today), today, id, false),
      ]);
      if (request !== detailRequest || detailId !== id) return;
      detail = next;
      detailHeat = heat;
    } catch (cause) {
      if (request === detailRequest) toast(errMessage(cause), "error");
    }
  }

  async function saveTools(apps: string[]) {
    if (!detail || toolBusy) return;
    toolBusy = true;
    try {
      await api.setProjectTools(detail.project.id, apps);
      await refreshDetail(detail.project.id);
      await loadBootstrap();
    } catch (cause) {
      toast(errMessage(cause), "error");
    } finally {
      toolBusy = false;
    }
  }

  function addTool(app: string) {
    if (!detail) return;
    void saveTools([...detail.tools, app]);
  }

  function removeTool(app: string) {
    if (!detail) return;
    void saveTools(detail.tools.filter((item) => item !== app));
  }

  async function pickTools() {
    try {
      const selected = await openFileDialog({
        title: "Choose focus tools",
        multiple: true,
        filters: [{ name: "Applications", extensions: ["exe"] }],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      const names = paths
        .map((path) => path.split(/[\\/]/).pop() ?? "")
        .filter(Boolean);
      if (names.length && detail) {
        await saveTools([...new Set([...detail.tools, ...names])]);
      }
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  function filterSessions(project: Project) {
    sessionsProject.set(project.id);
    navigate("sessions");
  }

  $effect(() => {
    if ($newProjectSignal > 0) {
      detailId = null;
      adding = true;
    }
  });

  $effect(() => {
    $refreshTick;
    if (detailId != null) void refreshDetail(detailId);
  });

  onMount(() => {
    const refreshIfVisible = () => {
      if (document.visibilityState === "visible" && detailId != null) {
        void refreshDetail(detailId);
      }
    };
    const timer = window.setInterval(refreshIfVisible, 30_000);
    window.addEventListener("focus", refreshIfVisible);
    document.addEventListener("visibilitychange", refreshIfVisible);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("focus", refreshIfVisible);
      document.removeEventListener("visibilitychange", refreshIfVisible);
    };
  });

  function onWindowClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (menuId != null && !target.closest(".row-menu-wrap")) menuId = null;
  }
</script>

<svelte:window onclick={onWindowClick} />

{#if detailId != null && detail}
  <div class="page detail">
    <button class="back" onclick={() => (detailId = null)}>
      <Icon name="chevron-right" size={13} /> Projects
    </button>

    <header class="detail-hero reveal">
      <div class="detail-total">{formatDuration(detail.totalSeconds)}</div>
      <div class="detail-name">
        {detail.project.name}{#if detail.project.client}<span class="detail-client">{" — "}{detail.project.client}</span>{/if}
      </div>
      <button class="detail-finish" onclick={() => detail && toggleArchive(detail.project)}>
        <Icon name={detail.project.archived ? "arrow-right" : "check"} size={13} />
        {detail.project.archived ? "Reopen project" : "Mark as finished"}
      </button>
    </header>

    <div class="detail-stats">
      <div class="detail-stat">
        <span class="detail-label">This week</span>
        <span class="detail-value">{formatDuration(detail.weekSeconds)}</span>
      </div>
      <div class="detail-stat">
        <span class="detail-label">Today</span>
        <span class="detail-value">{formatDuration(detail.todaySeconds)}</span>
      </div>
      <div class="detail-stat">
        <span class="detail-label">Sessions</span>
        <span class="detail-value">{detail.sessionCount}</span>
      </div>
      <div class="detail-stat">
        <span class="detail-label">Last active</span>
        <span class="detail-value small">
          {detail.lastActivity
            ? new Date(detail.lastActivity * 1000).toLocaleDateString(undefined, {
                month: "short",
                day: "numeric",
              })
            : "—"}
        </span>
      </div>
    </div>

    {#if focusTotal > 0}
      <section class="section focus-section">
        <div class="focus-top">
          <div>
            <div class="eyebrow">Focus</div>
            <div class="focus-number">{focusPercent}<span>%</span></div>
          </div>
          <p class="focus-copy">
            {formatDuration(detail.focusedSeconds)} in selected tools,
            {formatDuration(detail.unfocusedSeconds)} in other apps.
          </p>
        </div>
        <div class="focus-bar" role="img" aria-label={`${focusPercent}% of project time in selected tools`}>
          <span class="focus-fill" style:width="{focusPercent}%"></span>
          <span class="other-fill"></span>
        </div>
        <div class="focus-legend">
          <span><i class="focus-key on"></i> Selected tools</span>
          <span><i class="focus-key off"></i> Other apps</span>
        </div>
      </section>
    {/if}

    <section class="section tools-section">
      <Rule />
      <div class="section-head">
        <div>
          <div class="eyebrow">Focus tools</div>
          <p class="section-note">Selected apps count as focused while this project is active. Anything else is still tracked, but marked other.</p>
        </div>
        <div class="tool-actions">
          <button class="chip-add-button" onclick={() => detail && openProjectSetup(detail.project)}>
            Templates
          </button>
          <button class="chip-add-button" onclick={() => void pickTools()} disabled={toolBusy}>
            <Icon name="plus" size={13} /> Add .exe
          </button>
        </div>
      </div>
      <div class="tool-chips">
        {#each detail.tools as tool (tool)}
          <span class="tool-chip">
            <AppIcon app={tool} size={16} />
            <span class="tool-name">{appLabel(tool)}</span>
            <button class="tool-x" onclick={() => removeTool(tool)} aria-label={`Remove ${appLabel(tool)}`}>
              <Icon name="x" size={10} />
            </button>
          </span>
        {/each}
        {#if detail.tools.length === 0}
          <span class="tool-empty">No tools yet. Add the apps that count as real work.</span>
        {/if}
      </div>
      {#if toolSuggestions.length > 0}
        <div class="tool-suggest">
          <span>Quick add</span>
          {#each toolSuggestions as app (app)}
            <button onclick={() => addTool(app)}>+ {appLabel(app)}</button>
          {/each}
        </div>
      {/if}
    </section>

    <section class="section">
      <Rule />
      <div class="eyebrow">Applications</div>
      {#if focusTotal > 0}
        <div class="app-group">
          <span><i class="focus-key on"></i> Selected tools</span>
          <span>{formatDuration(detail.focusedSeconds)}</span>
        </div>
        <div class="apps">
          {#each focusedApps.slice(0, 8) as app (app.app)}
            <div class="app-row">
              <span class="app-name">{appLabel(app.app)}</span>
              <span class="app-track">
                <span class="app-fill" style:transform="scaleX({app.seconds / detailAppMax})" style:background={detail.project.color}></span>
              </span>
              <span class="app-time">{formatDuration(app.seconds)}</span>
            </div>
          {/each}
          {#if focusedApps.length === 0}
            <div class="empty-sub">Nothing in a selected tool yet.</div>
          {/if}
        </div>
        <div class="app-group">
          <span><i class="focus-key off"></i> Other apps</span>
          <span>{formatDuration(detail.unfocusedSeconds)}</span>
        </div>
        <div class="apps">
          {#each otherApps.slice(0, 8) as app (app.app + (app.focused ?? "?"))}
            <div class="app-row">
              <span class="app-name"><i class="focus-key" class:neutral={app.focused == null}></i> {appLabel(app.app)}</span>
              <span class="app-track">
                <span class="app-fill other" style:transform="scaleX({app.seconds / detailAppMax})"></span>
              </span>
              <span class="app-time">{formatDuration(app.seconds)}</span>
            </div>
          {/each}
          {#if otherApps.length === 0}
            <div class="empty-sub">No other app time yet.</div>
          {/if}
        </div>
      {:else}
        <div class="apps">
          {#each detail.perApp.slice(0, 8) as app (app.app)}
            <div class="app-row">
              <span class="app-name">{appLabel(app.app)}</span>
              <span class="app-track">
                <span class="app-fill" style:transform="scaleX({app.seconds / detailAppMax})" style:background={detail.project.color}></span>
              </span>
              <span class="app-time">{formatDuration(app.seconds)}</span>
            </div>
          {/each}
          {#if detail.perApp.length === 0}
            <div class="empty-sub">Nothing tracked yet.</div>
          {/if}
        </div>
      {/if}
    </section>

    <section class="section">
      <Rule />
      <div class="section-head">
        <div>
          <div class="eyebrow">Last 30 days</div>
        </div>
        <span class="section-meta">{formatRange(daysAgo(29, today), today)}</span>
      </div>
      <div class="mini-chart">
        {#each detailDays as day (day.day)}
          <span class="mini-slot" title={`${day.day}: ${formatDuration(day.seconds)}`}>
            <span
              class="mini-bar"
              style:height="{Math.max(day.seconds > 0 ? 3 : 1, (day.seconds / detailMax) * 100)}%"
              style:background={detail.project.color}
            ></span>
          </span>
        {/each}
      </div>
    </section>

    <section class="section">
      <Rule />
      <div class="section-head">
        <div><div class="eyebrow">Six months</div></div>
        <button class="link" onclick={() => detail && filterSessions(detail.project)}>
          View sessions <Icon name="arrow-right" size={13} />
        </button>
      </div>
      <div class="mini-heat">
        <Heatmap cells={detailHeat} endDay={today} weeks={27} />
      </div>
    </section>
  </div>
{:else}
  <div class="page projects">
    <header class="page-head reveal">
      <div>
        <div class="eyebrow">Projects</div>
        <h1 class="section-heading">Work in progress</h1>
      </div>
      <span class="week-total">{formatDuration(totalWeek)} this week</span>
    </header>

    <div class="project-list">
      {#each rows as row (row.project.id)}        <div class="project-row" class:active={row.active}>
          <button class="project-open" onclick={() => openDetail(row.project)}>
            <span class="dot" class:active-dot={row.active} style:background={row.project.color}></span>
            <div class="row-main">
              {#if renameId === row.project.id}
                <input
                  class="inline-input"
                  bind:value={renameValue}
                  onkeydown={(event) => {
                    if (event.key === "Enter") commitRename(row.project);
                    if (event.key === "Escape") renameId = null;
                  }}
                  onblur={() => commitRename(row.project)}
                  onclick={(event) => event.stopPropagation()}
                />
              {:else}
                <div class="row-title">{row.project.name}</div>
                <div class="row-sub">{row.project.client ?? "No client"}</div>
              {/if}
            </div>

            <div class="project-time">
              <span class="row-value">{formatDuration(row.seconds)}</span>
              <span class="project-track">
                <span
                  class="fill"
                  class:done={row.target > 0 && row.seconds >= row.target}
                  style:transform="scaleX({row.target > 0 ? clampPercent(row.percent) / 100 : 0.02})"
                ></span>
              </span>
            </div>
          </button>

          <div class="row-menu-wrap">
            <button
              class="menu-button"
              onclick={(event) => {
                event.stopPropagation();
                menuId = menuId === row.project.id ? null : row.project.id;
              }}
              aria-label="Project menu"
            >
              <Icon name="more" size={15} />
            </button>
            {#if menuId === row.project.id}
              <div class="menu row-menu">
                <button class="menu-item" onclick={() => startRename(row.project)}>
                  <Icon name="pencil" size={14} /> Rename
                </button>
                <button class="menu-item" onclick={() => toggleArchive(row.project)}>
                  <Icon name="check" size={14} /> Finish project
                </button>
                <div class="menu-sep"></div>
                <button class="menu-item danger" onclick={() => remove(row.project)}>
                  <Icon name="trash" size={14} /> Delete
                </button>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>

    {#if activeList.length === 0 && archivedList.length === 0 && !adding}
      <div class="projects-empty">
        <div class="empty-title">No projects yet</div>
        <div class="empty-sub">Create one, choose the apps that count, and Framewise starts crediting work.</div>
      </div>
    {/if}

    {#if adding}
      <form
        class="new-project"
        onsubmit={(event) => {
          event.preventDefault();
          void create();
        }}
      >
        <div class="new-fields">
          <input
            class="input"
            placeholder="Project name"
            bind:value={newName}
            aria-label="Project name"
          />
          <input
            class="input"
            placeholder="Client (optional)"
            bind:value={newClient}
            aria-label="Client"
          />
        </div>
        <div class="swatches" role="radiogroup" aria-label="Project color">
          {#each palette as color}
            <button
              type="button"
              class="swatch"
              class:selected={newColor === color}
              style:background={color}
              aria-label={`Color ${color}`}
              aria-checked={newColor === color}
              role="radio"
              onclick={() => (newColor = color)}
            ></button>
          {/each}
        </div>
        <div class="new-hint">Next you can pick focus tools, or skip and add them later.</div>
        <div class="new-actions">
          <button class="btn btn-primary btn-sm" type="submit" disabled={creating}>
            {creating ? "Creating…" : "Create project"}
          </button>
          <button class="btn btn-ghost btn-sm" type="button" onclick={() => (adding = false)}>
            Cancel
          </button>
        </div>
      </form>
    {:else}
      <button class="add-row" onclick={() => (adding = true)}>
        <Icon name="plus" size={16} /> New project
      </button>
    {/if}

    {#if archivedList.length > 0}
      <section class="section-sm">
        <div class="eyebrow archived-label">Finished</div>
        <div class="project-list">
          {#each archivedList as project (project.id)}
            <div class="project-row archived">
              <button class="project-open" onclick={() => openDetail(project)}>
                <span class="dot" style:background={project.color}></span>
                <div class="row-main">
                  <div class="row-title">{project.name}</div>
                  <div class="row-sub">{project.client ?? "No client"}</div>
                </div>
                <span class="row-value">Finished</span>
              </button>
              <button class="menu-button" onclick={() => toggleArchive(project)} aria-label="Reopen project">
                <Icon name="arrow-right" size={14} />
              </button>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  </div>
{/if}

<style>
  .page-head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
    padding-top: 64px;
    margin-bottom: 28px;
  }

  .week-total {
    color: var(--muted);
    font-size: 13px;
    white-space: nowrap;
    padding-bottom: 4px;
  }

  .project-list {
    display: flex;
    flex-direction: column;
  }

  .project-row {
    display: flex;
    align-items: center;
    gap: 4px;
    border-bottom: 1px solid var(--border-subtle);
    transition: background-color 150ms ease-out;
  }

  .project-row:hover {
    background: var(--surface);
  }

  .project-open {
    display: flex;
    align-items: center;
    gap: 14px;
    flex: 1;
    min-width: 0;
    min-height: 68px;
    padding: 12px 8px;
    border: 0;
    background: transparent;
    color: var(--text);
    text-align: left;
  }

  .dot.active-dot {
    box-shadow: 0 0 0 3px var(--accent-soft);
    outline: 1px solid var(--accent);
    outline-offset: 1px;
  }

  .row-title {
    font-size: 15.5px;
  }

  .project-time {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 6px;
    width: 132px;
    flex: 0 0 132px;
  }

  .project-track {
    display: block;
    width: 100%;
    height: 3px;
    border-radius: 999px;
    background: var(--surface-3);
    overflow: hidden;
  }

  .project-track .fill {
    display: block;
    width: 100%;
    height: 100%;
  }

  .menu-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--muted-2);
    opacity: 0;
    transition:
      opacity 150ms ease-out,
      background-color 150ms ease-out;
  }

  .project-row:hover .menu-button,
  .menu-button:focus-visible {
    opacity: 1;
  }

  .menu-button:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .row-menu-wrap {
    position: relative;
  }

  .row-menu {
    top: 34px;
    right: 0;
    width: 170px;
  }

  .row-menu .danger {
    color: var(--danger);
  }

  .inline-input {
    width: 100%;
    height: 28px;
    padding: 0 8px;
    border: 1px solid var(--accent-line);
    border-radius: 7px;
    background: var(--surface);
    color: var(--text);
    font-size: 15px;
    outline: none;
  }

  .add-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    min-height: 56px;
    padding: 12px 8px;
    border: 0;
    border-bottom: 1px solid var(--border-subtle);
    background: transparent;
    color: var(--muted);
    font-size: 14px;
    text-align: left;
    transition: color 150ms ease-out, background-color 150ms ease-out;
  }

  .add-row:hover {
    background: var(--surface);
    color: var(--text);
  }

  .new-project {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px;
    margin-top: 12px;
    border-radius: var(--r-tile);
    background: var(--surface);
  }

  .new-fields {
    display: grid;
    grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
    gap: 10px;
  }

  .swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .swatch {
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    outline: 2px solid transparent;
    outline-offset: 2px;
    transition: outline-color 140ms ease-out;
  }

  .swatch.selected {
    outline-color: var(--text);
  }

  .new-actions {
    display: flex;
    gap: 8px;
  }

  .new-hint {
    margin-top: 10px;
    color: var(--muted-2);
    font-size: 11.5px;
  }

  .archived-label {
    margin: 8px 0 6px;
  }

  .projects-empty {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 22px 0 6px;
  }

  .project-row.archived {
    opacity: 0.55;
  }

  /* Detail */
  .back {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    margin-top: 40px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--muted);
    font-size: 13px;
  }

  .back :global(svg) {
    transform: rotate(180deg);
  }

  .back:hover {
    color: var(--text);
  }

  .detail-hero {
    margin-top: 24px;
  }

  .detail-total {
    font-size: clamp(44px, 7vw, 64px);
    font-weight: 600;
    letter-spacing: -0.04em;
    line-height: 1;
  }

  .detail-name {
    margin-top: 10px;
    color: var(--muted);
    font-size: 24px;
    font-weight: 500;
    letter-spacing: -0.02em;
  }

  .detail-finish {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-top: 14px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--muted);
    font-size: 12px;
  }

  .detail-finish:hover {
    color: var(--accent-text);
  }

  .detail-stats {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 20px;
    margin-top: 40px;
    padding-bottom: 8px;
  }

  .detail-stat {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .detail-label {
    color: var(--muted);
    font-size: 12px;
  }

  .detail-value {
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.03em;
    font-variant-numeric: tabular-nums;
  }

  .detail-value.small {
    font-size: 16px;
  }

  .focus-section {
    margin-top: 36px;
  }

  .focus-top {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 24px;
  }

  .focus-number {
    margin-top: 4px;
    color: var(--green);
    font-size: 48px;
    font-weight: 600;
    letter-spacing: -0.045em;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }

  .focus-number span {
    margin-left: 1px;
    color: var(--muted);
    font-size: 24px;
    letter-spacing: -0.02em;
  }

  .focus-copy {
    max-width: 300px;
    padding-bottom: 3px;
    color: var(--muted);
    font-size: 12.5px;
    line-height: 1.5;
    text-align: right;
  }

  .focus-bar {
    display: flex;
    height: 8px;
    margin-top: 16px;
    overflow: hidden;
    border-radius: 999px;
    background: var(--surface-3);
  }

  .focus-fill {
    height: 100%;
    background: var(--green);
    transition: width 600ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .other-fill {
    flex: 1;
    height: 100%;
    background: color-mix(in srgb, var(--danger) 65%, var(--surface-3));
  }

  .focus-legend {
    display: flex;
    gap: 18px;
    margin-top: 9px;
    color: var(--muted);
    font-size: 11px;
  }

  .focus-legend span,
  .app-group > span:first-child {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .focus-key {
    display: inline-block;
    width: 6px;
    height: 6px;
    flex: 0 0 auto;
    border-radius: 999px;
    background: var(--muted-2);
  }

  .focus-key.on {
    background: var(--green);
  }

  .focus-key.off {
    background: var(--danger);
  }

  .focus-key.neutral {
    background: var(--muted-2);
  }

  .section-note {
    max-width: 580px;
    margin-top: 7px;
    color: var(--muted);
    font-size: 13px;
    line-height: 1.5;
  }

  .chip-add-button {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 30px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: transparent;
    color: var(--muted);
    font-size: 11.5px;
    white-space: nowrap;
  }

  .tool-actions {
    display: flex;
    align-items: center;
    gap: 7px;
  }

  .chip-add-button:hover {
    border-color: var(--accent-line);
    background: var(--accent-soft);
    color: var(--accent-text);
  }

  .chip-add-button:disabled {
    opacity: 0.5;
  }

  .tool-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 7px;
    margin-top: 16px;
  }

  .tool-chip {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 32px;
    padding: 0 5px 0 8px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface);
    font-size: 12.5px;
  }

  .tool-name {
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tool-x {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--muted-2);
    opacity: 0.55;
  }

  .tool-chip:hover .tool-x,
  .tool-x:focus-visible {
    opacity: 1;
  }

  .tool-x:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .tool-empty {
    color: var(--muted-2);
    font-size: 12.5px;
  }

  .tool-suggest {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    margin-top: 12px;
    color: var(--muted-2);
    font-size: 11px;
  }

  .tool-suggest button {
    height: 24px;
    padding: 0 8px;
    border: 1px dashed var(--border);
    border-radius: 999px;
    background: transparent;
    color: var(--muted);
    font-size: 10.5px;
  }

  .tool-suggest button:hover {
    border-color: var(--accent-line);
    color: var(--accent-text);
  }

  .app-group {
    display: flex;
    justify-content: space-between;
    margin-top: 18px;
    color: var(--muted);
    font-size: 10.5px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .app-fill.other {
    background: color-mix(in srgb, var(--danger) 62%, var(--surface-3));
  }

  .app-name {
    display: flex;
    align-items: center;
    gap: 7px;
  }

  .apps {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-top: 16px;
  }

  .app-row {
    display: flex;
    align-items: center;
    gap: 14px;
    min-height: 38px;
  }

  .app-name {
    width: 160px;
    flex: 0 0 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 14px;
  }

  .app-track {
    flex: 1;
    height: 6px;
    border-radius: 999px;
    background: var(--surface-3);
    overflow: hidden;
  }

  .app-fill {
    display: block;
    width: 100%;
    height: 100%;
    border-radius: inherit;
    transform-origin: left center;
    transition: transform 700ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .app-time {
    width: 64px;
    flex: 0 0 64px;
    text-align: right;
    color: var(--muted);
    font-size: 13px;
    font-variant-numeric: tabular-nums;
  }

  .mini-chart {
    display: flex;
    align-items: flex-end;
    gap: 4px;
    height: 120px;
    margin-top: 6px;
  }

  .mini-slot {
    flex: 1;
    display: flex;
    align-items: flex-end;
    height: 100%;
  }

  .mini-bar {
    display: block;
    width: 100%;
    border-radius: 4px 4px 2px 2px;
    opacity: 0.85;
    transform-origin: bottom;
  }

  .mini-slot:hover .mini-bar {
    opacity: 1;
  }

  .mini-heat {
    margin-top: 6px;
  }

  @media (max-width: 760px) {
    .detail-stats {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .project-time {
      width: 96px;
      flex-basis: 96px;
    }
    .new-fields {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
