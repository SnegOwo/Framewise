<script lang="ts">
  import type { HeatmapCell, Session } from "../types";
  import { api, errMessage } from "../api";
  import {
    activeProjects,
    boot,
    bumpRefresh,
    dailyGoal,
    goalSeconds,
    goals,
    loadBootstrap,
    refreshTick,
    showAppInfo,
    status,
    streakDays,
    toast,
  } from "../stores";
  import { navigate, requestNewProject, sessionsDay, sessionsProject } from "../router";
  import { appLabel, formatDuration, formatHMS, formatTimestamp, todayStr } from "../format";
  import Icon from "../components/Icon.svelte";
  import Rule from "../components/Rule.svelte";
  import StatTile from "../components/StatTile.svelte";
  import CountUp from "../components/CountUp.svelte";
  import AppIcon from "../components/AppIcon.svelte";
  import ProjectPicker from "../components/ProjectPicker.svelte";
  import Timeline from "../components/Timeline.svelte";
  import Heatmap from "../components/Heatmap.svelte";
  import GoalRow from "../components/GoalRow.svelte";

  let today = $derived($boot?.today ?? todayStr());
  let currentYear = new Date().getFullYear();
  let year = $state(currentYear);
  let heatFilter = $state("all");
  let heat = $state<HeatmapCell[]>([]);
  let heatLoading = $state(true);
  let sessions = $state<Session[]>([]);
  let error = $state("");
  let clock = $state(Math.floor(Date.now() / 1000));
  let firstRunDismissed = $state((() => {
    try {
      return localStorage.getItem("framewise-general-tracking") === "1";
    } catch {
      return false;
    }
  })());

  let adding = $state(false);
  let addPeriod = $state<"daily" | "weekly">("daily");
  let addProjectId = $state<number | null>(null);
  let addTarget = $state(4 * 3600);

  let todaySeconds = $derived($status?.todaySeconds ?? 0);
  let isLive = $derived(!!$status && !$status.paused && !$status.locked && !$status.idle);
  let trackState = $derived<"live" | "paused" | "idle" | "locked">(
    $status?.paused ? "paused" : $status?.locked ? "locked" : $status?.idle ? "idle" : "live",
  );
  let liveSeconds = $derived(
    $status?.trackingSince ? Math.max(0, clock - $status.trackingSince) : 0,
  );
  let target = $derived($dailyGoal?.targetSeconds ?? 0);
  let goalsFocusOnly = $derived($boot?.settings.goalsFocusOnly ?? true);
  let goalToday = $derived(
    $dailyGoal ? goalSeconds($dailyGoal, $status, goalsFocusOnly) : 0,
  );
  let percent = $derived(target > 0 ? Math.round((goalToday / target) * 100) : 0);
  let remaining = $derived(Math.max(0, target - goalToday));

  let heatProjectId = $derived(
    heatFilter === "all" || heatFilter === "unassigned" ? null : Number(heatFilter),
  );
  let heatUnassigned = $derived(heatFilter === "unassigned");
  let heatEnd = $derived(year === currentYear ? today : `${year}-12-31`);
  let heatHours = $derived(
    Math.round(heat.reduce((sum, cell) => sum + cell.seconds, 0) / 3600),
  );

  let sortedGoals = $derived.by(() => {
    const rank = (scope: string, period: string) =>
      (scope === "global" ? 0 : 10) + (period === "daily" ? 0 : 1);
    return [...$goals]
      .sort((a, b) => rank(a.scope, a.period) - rank(b.scope, b.period))
      .map((goal) => ({
        goal,
        seconds: goalSeconds(goal, $status, goalsFocusOnly),
      }));
  });

  async function loadHeat() {
    heatLoading = true;
    try {
      heat = await api.heatmap(
        `${year}-01-01`,
        `${year}-12-31`,
        heatProjectId,
        heatUnassigned,
      );
      error = "";
    } catch (cause) {
      error = errMessage(cause);
    } finally {
      heatLoading = false;
    }
  }

  async function loadSessions() {
    try {
      sessions = await api.sessions({ startDay: today, endDay: today, limit: 500 });
    } catch (cause) {
      error = errMessage(cause);
    }
  }

  async function saveGoal(goalId: number, projectId: number | null, targetSeconds: number) {
    try {
      await api.updateGoal(goalId, projectId, targetSeconds);
      await loadBootstrap();
      bumpRefresh();
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  async function addGoal() {
    try {
      const scope = addProjectId == null ? "global" : "project";
      await api.createGoal(scope, addProjectId, addPeriod, addTarget);
      await loadBootstrap();
      bumpRefresh();
      adding = false;
      toast("Goal added.", "success");
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  function openDay(day: string) {
    sessionsDay.set(day);
    sessionsProject.set(null);
    navigate("sessions");
  }

  function trackGenerally() {
    firstRunDismissed = true;
    try {
      localStorage.setItem("framewise-general-tracking", "1");
    } catch {
      // ignore
    }
  }

  $effect(() => {
    if ($activeProjects.length > 0 && firstRunDismissed) {
      firstRunDismissed = false;
      try {
        localStorage.removeItem("framewise-general-tracking");
      } catch {
        // ignore
      }
    }
  });

  $effect(() => {
    year;
    heatFilter;
    $refreshTick;
    void loadHeat();
  });

  $effect(() => {
    $refreshTick;
    today;
    void loadSessions();
  });

  $effect(() => {
    const id = window.setInterval(() => {
      if (document.visibilityState === "visible") {
        clock = Math.floor(Date.now() / 1000);
      }
    }, 1000);
    return () => window.clearInterval(id);
  });

  $effect(() => {
    const id = window.setInterval(() => {
      if (document.visibilityState === "visible") {
        void loadSessions();
        void loadHeat();
      }
    }, 60_000);
    return () => window.clearInterval(id);
  });
</script>

<div class="page dashboard">
  <section class="hero-block reveal">
    <h1 class="hero">
      <span class="hero-line">
        <CountUp value={todaySeconds} formatter={formatDuration} duration={400} /> today on
        <ProjectPicker variant="hero" />
      </span>
      <span class="hero-line hero-muted">
        {#if target > 0}
          {#if remaining > 0}
            {percent}% of your {formatDuration(target)} goal · {formatDuration(remaining)} to go
          {:else}
            {formatDuration(target)} goal reached · {formatDuration(todaySeconds)} tracked
          {/if}
        {:else}
          No daily goal yet
        {/if}
      </span>
    </h1>
  </section>

  {#if $boot && $activeProjects.length === 0 && !firstRunDismissed}
    <section class="start-card reveal">
      <span class="start-mark"><Icon name="target" size={20} /></span>
      <div class="start-copy">
        <span class="eyebrow">First run</span>
        <div class="start-title">Make your first project.</div>
        <p>Name it and choose the apps that count — or track generally without a project for now.</p>
      </div>
      <div class="start-actions">
        <button class="btn btn-primary" onclick={requestNewProject}>
          Create a project
        </button>
        <button class="btn btn-ghost" onclick={trackGenerally}>
          Track generally
        </button>
      </div>
    </section>
  {/if}

  {#if error}
    <div class="inline-error">
      <Icon name="info" size={14} />
      <span>{error}</span>
      <button class="link" onclick={() => void loadHeat()}>Retry</button>
    </div>
  {/if}

  <section class="section-sm status-section reveal">
    <div class="status-card">
      <div class="status-top">
        {#if $status?.currentApp}
          <button
            class="status-app-open"
            onclick={() => $status?.currentApp && showAppInfo($status.currentApp)}
            title="Inspect application"
          >
            <AppIcon app={$status.currentApp} size={38} live={isLive} />
            <span class="status-app">
              <span class="status-name">{appLabel($status.currentApp)}</span>
              <span class="status-sub">
                {#if $status.trackingSince}
                  Since {formatTimestamp($status.trackingSince)}
                {:else}
                  Waiting for input
                {/if}
                {#if $status.renderActive}· Media Encoder rendering{/if}
              </span>
            </span>
          </button>
          {#if isLive}
            <div class="status-timer">
              <span class="timer">{formatHMS(liveSeconds)}</span>
              <span class="status-state"><span class="dot dot-live"></span>Tracking</span>
              {#if $status?.currentAppFocused !== null && $status?.currentAppFocused !== undefined}
                <span
                  class="focus-pill"
                  class:on={$status.currentAppFocused}
                  class:off={!$status.currentAppFocused}
                >
                  <span class="focus-dot"></span>
                  {$status.currentAppFocused ? "On tool" : "Off tool"}
                </span>
              {/if}
            </div>
          {:else}
            <div class="status-timer">
              <span class="status-state"><span class="dot dot-paused"></span>{trackState === "paused" ? "Paused" : trackState === "idle" ? "Idle" : "Locked"}</span>
            </div>
          {/if}
        {:else}
          <span class="status-empty-icon"><Icon name="monitor" size={18} /></span>
          <div class="status-app">
            <div class="status-name">Nothing in focus</div>
            <div class="status-sub">Open an app to start the clock</div>
          </div>
          <div class="status-timer">
            <span class="status-state"><span class="dot dot-paused"></span>{trackState === "paused" ? "Paused" : "Waiting"}</span>
          </div>
        {/if}
      </div>

      <Timeline
        {sessions}
        projects={$boot?.projects ?? []}
        day={today}
        nowTs={clock}
        {trackState}
        currentApp={$status?.currentApp ?? null}
      />
    </div>
  </section>

  <section class="tiles reveal">
    <StatTile
      icon="clock"
      label="Today"
      seconds={todaySeconds}
      formatter={formatDuration}
      tone="accent"
    />
    <StatTile
      icon="calendar"
      label="This week"
      seconds={$status?.weekSeconds ?? 0}
      formatter={formatDuration}
    />
    <StatTile
      icon="target"
      label="Project today"
      seconds={$status?.projectTodaySeconds ?? 0}
      formatter={formatDuration}
    />
    <StatTile
      icon="flame"
      label={$streakDays === 1 ? "Day streak" : "Day streak"}
      value={`${$streakDays}`}
    />
  </section>

  <section class="section">
    <Rule />
    <div class="section-head heat-head">
      <div>
        <div class="eyebrow">Activity</div>
        <h2 class="section-heading">
          {#if heatLoading && heat.length === 0}
            Loading
          {:else}
            {heatHours}h tracked in {year}
          {/if}
        </h2>
      </div>
      <div class="heat-controls">
        <label class="filter-pill">
          <span class="sr-only">Filter activity by project</span>
          <select bind:value={heatFilter}>
            <option value="all">All projects</option>
            <option value="unassigned">Unassigned</option>
            {#each $activeProjects as project (project.id)}
              <option value={String(project.id)}>{project.name}</option>
            {/each}
          </select>
          <Icon name="chevron-down" size={12} />
        </label>
        <div class="year-switch" role="tablist" aria-label="Year">
          <button
            class:active={year === currentYear}
            role="tab"
            aria-selected={year === currentYear}
            onclick={() => (year = currentYear)}>{currentYear}</button
          >
          <button
            class:active={year === currentYear - 1}
            role="tab"
            aria-selected={year === currentYear - 1}
            onclick={() => (year = currentYear - 1)}>{currentYear - 1}</button
          >
        </div>
      </div>
    </div>
    <div class="heat-surface">
      <Heatmap cells={heat} endDay={heatEnd} weeks={53} onselect={openDay} />
    </div>
  </section>

  <section class="section">
    <Rule />
    <div class="section-head">
      <div>
        <div class="eyebrow">Goals</div>
      </div>
      <button class="btn btn-sm" onclick={() => (adding = !adding)}>
        <Icon name="plus" size={14} /> Goal
      </button>
    </div>

    {#if adding}
      <div class="add-goal">
        <div class="seg add-seg" role="tablist">
          <button class:active={addPeriod === "daily"} role="tab" onclick={() => (addPeriod = "daily")}>Daily</button>
          <button class:active={addPeriod === "weekly"} role="tab" onclick={() => (addPeriod = "weekly")}>Weekly</button>
        </div>
        <select class="select add-select" bind:value={addProjectId} aria-label="Goal scope">
          <option value={null}>All projects</option>
          {#each $activeProjects as project (project.id)}
            <option value={project.id}>{project.name}</option>
          {/each}
        </select>
        <div class="stepper">
          <button onclick={() => (addTarget = Math.max(900, addTarget - 1800))} aria-label="Less">
            <Icon name="minus" size={13} />
          </button>
          <span class="value">{formatDuration(addTarget)}</span>
          <button onclick={() => (addTarget += 1800)} aria-label="More">
            <Icon name="plus" size={13} />
          </button>
        </div>
        <button class="btn btn-primary btn-sm" onclick={addGoal}>Add</button>
      </div>
    {/if}

    {#if sortedGoals.length === 0}
      <div class="goals-empty">
        <div class="empty-title">No goals yet</div>
        <div class="empty-sub">Set a daily or weekly target and the rings fill as you work.</div>
      </div>
    {:else}
      <div class="goals">
        {#each sortedGoals as row (row.goal.id)}
          <GoalRow
            goal={row.goal}
            seconds={row.seconds}
            project={$boot?.projects.find((project) => project.id === row.goal.projectId) ?? null}
            onchange={(seconds) => saveGoal(row.goal.id, row.goal.projectId, seconds)}
          />
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .hero-block {
    padding-top: 56px;
  }

  .hero :global(.chip) {
    margin-left: 8px;
  }

  .status-section {
    margin-top: 40px;
  }

  .start-card {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-top: 34px;
    padding: 20px 22px;
    border: 1px solid var(--border);
    border-radius: var(--r-large);
    background:
      radial-gradient(420px 180px at 100% 0%, var(--accent-soft), transparent 70%),
      var(--surface);
  }

  .start-mark {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 42px;
    height: 42px;
    flex: 0 0 auto;
    border-radius: 13px;
    background: var(--accent-soft);
    color: var(--accent-text);
  }

  .start-copy {
    min-width: 0;
    flex: 1;
  }

  .start-title {
    margin-top: 6px;
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  .start-copy p {
    max-width: 540px;
    margin-top: 3px;
    color: var(--muted);
    font-size: 12.5px;
    line-height: 1.5;
  }

  .start-actions {
    display: flex;
    flex: 0 0 auto;
    gap: 7px;
  }

  .status-card {
    padding: 18px 22px 16px;
    border-radius: var(--r-large);
    background: var(--surface);
  }

  .status-top {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .status-app-open {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    min-width: 0;
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
  }

  .status-app-open:hover .status-name {
    color: var(--accent-text);
  }

  .status-app {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
  }

  .status-name {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 16px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  .status-sub {
    display: block;
    margin-top: 1px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--muted);
    font-size: 12.5px;
  }

  .status-timer {
    text-align: right;
  }

  .timer {
    display: block;
    font-size: 26px;
    font-weight: 600;
    letter-spacing: -0.04em;
    line-height: 1.05;
    font-variant-numeric: tabular-nums;
  }

  .status-state {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
    font-size: 11.5px;
  }

  .status-state .dot {
    width: 6px;
    height: 6px;
  }

  .focus-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 20px;
    margin-top: 5px;
    padding: 0 8px;
    border: 1px solid var(--border);
    border-radius: 999px;
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.01em;
  }

  .focus-pill.on {
    border-color: color-mix(in srgb, var(--green) 35%, transparent);
    background: color-mix(in srgb, var(--green) 10%, transparent);
    color: var(--green);
  }

  .focus-pill.off {
    border-color: color-mix(in srgb, var(--danger) 32%, transparent);
    background: var(--danger-soft);
    color: var(--danger);
  }

  .focus-dot {
    width: 5px;
    height: 5px;
    border-radius: 999px;
    background: currentColor;
  }

  .status-empty-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    border-radius: 11px;
    background: var(--surface-3);
    color: var(--muted);
  }

  .tiles {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 12px;
    margin-top: 16px;
  }

  .heat-head {
    align-items: flex-end;
  }

  .heat-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .filter-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 32px;
    padding: 0 10px;
    border-radius: var(--r-pill);
    background: var(--surface-3);
    color: var(--muted);
  }

  .filter-pill select {
    max-width: 150px;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: 12.5px;
    appearance: none;
    outline: none;
  }

  .year-switch {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 3px;
    border-radius: var(--r-pill);
    background: var(--surface-3);
  }

  .year-switch button {
    height: 26px;
    padding: 0 9px;
    border: 0;
    border-radius: var(--r-pill);
    background: transparent;
    color: var(--muted);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .year-switch button.active {
    background: var(--surface-2);
    color: var(--text);
  }

  .heat-surface {
    padding: 22px 24px 18px;
    border-radius: var(--r-large);
    background: var(--surface);
  }

  .add-goal {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    margin-bottom: 14px;
    padding: 12px;
    border-radius: var(--r-control);
    background: var(--surface);
  }

  .add-seg {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 3px;
    border-radius: var(--r-pill);
    background: var(--surface-3);
  }

  .add-seg button {
    height: 28px;
    padding: 0 11px;
    border: 0;
    border-radius: var(--r-pill);
    background: transparent;
    color: var(--muted);
    font-size: 12.5px;
  }

  .add-seg button.active {
    background: var(--surface-2);
    color: var(--text);
  }

  .add-select {
    width: auto;
    min-width: 150px;
    height: 34px;
  }

  .goals {
    display: flex;
    flex-direction: column;
  }

  .goals-empty {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 14px 0 8px;
  }

  .inline-error {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 22px;
    padding: 10px 12px;
    border-radius: var(--r-control);
    background: var(--danger-soft);
    color: var(--danger);
    font-size: 13px;
  }

  .inline-error span {
    flex: 1;
  }

  @media (max-width: 820px) {
    .tiles {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .hero-block {
      padding-top: 48px;
    }
    .heat-head {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }
    .start-card {
      flex-wrap: wrap;
    }
    .start-copy {
      flex-basis: calc(100% - 58px);
    }
    .start-actions {
      margin-left: 58px;
    }
  }
</style>
