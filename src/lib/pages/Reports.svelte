<script lang="ts">
  import type { Report } from "../types";
  import { api, errMessage } from "../api";
  import {
    activeProjects,
    boot,
    dailyGoal,
    refreshTick,
  } from "../stores";
  import {
    daysAgo,
    daysBetween,
    fillDays,
    formatDuration,
    formatRange,
    monthStart,
    shiftDay,
    todayStr,
    weekBounds,
  } from "../format";
  import { exportRange } from "../export";
  import ActivityChart from "../components/ActivityChart.svelte";
  import BarList from "../components/BarList.svelte";
  import Icon from "../components/Icon.svelte";
  import Rule from "../components/Rule.svelte";
  import Segmented from "../components/Segmented.svelte";

  type RangeId = "week" | "lastweek" | "month" | "30" | "year" | "custom";

  const ranges = [
    { value: "week", label: "This week" },
    { value: "lastweek", label: "Last week" },
    { value: "month", label: "This month" },
    { value: "30", label: "30 days" },
    { value: "year", label: "This year" },
    { value: "custom", label: "Custom" },
  ];

  let today = $derived($boot?.today ?? todayStr());
  let rangeId = $state<RangeId>("week");
  let customStart = $state(daysAgo(29, todayStr()));
  let customEnd = $state(todayStr());

  let filter = $state("all");
  let filterOpen = $state(false);
  let exportOpen = $state(false);

  let report = $state<Report | null>(null);
  let loading = $state(true);
  let error = $state("");
  let requestSeq = 0;

  let bounds = $derived.by(() => {
    if (rangeId === "custom") {
      return {
        start: customStart <= customEnd ? customStart : customEnd,
        end: customStart <= customEnd ? customEnd : customStart,
      };
    }
    if (rangeId === "lastweek") {
      const previous = weekBounds(shiftDay(today, -7));
      return { start: previous.start, end: previous.end };
    }
    if (rangeId === "month") return { start: monthStart(today), end: today };
    if (rangeId === "30") return { start: daysAgo(29, today), end: today };
    if (rangeId === "year") return { start: `${today.slice(0, 4)}-01-01`, end: today };
    const current = weekBounds(today);
    return { start: current.start, end: current.end };
  });

  let filterProjectId = $derived(
    filter === "all" || filter === "unassigned" ? null : Number(filter),
  );
  let filterUnassigned = $derived(filter === "unassigned");
  let filterLabel = $derived(
    filter === "all"
      ? "All projects"
      : filter === "unassigned"
        ? "Unassigned"
        : $activeProjects.find((project) => String(project.id) === filter)?.name ?? "Project",
  );

  let chartData = $derived(
    report ? fillDays(bounds.start, bounds.end, report.perDay) : [],
  );
  let activeDays = $derived(chartData.filter((row) => row.seconds > 0).length);
  let totalSeconds = $derived(report?.totalSeconds ?? 0);
  let average = $derived(activeDays > 0 ? totalSeconds / activeDays : 0);
  let sessionCount = $derived(report?.sessionCount ?? 0);
  let dayCount = $derived(daysBetween(bounds.start, bounds.end).length);

  async function load() {
    const seq = ++requestSeq;
    loading = true;
    try {
      const next = await api.report(
        bounds.start,
        bounds.end,
        filterProjectId,
        filterUnassigned,
      );
      if (seq !== requestSeq) return;
      report = next;
      error = "";
    } catch (cause) {
      if (seq !== requestSeq) return;
      error = errMessage(cause);
    } finally {
      if (seq === requestSeq) loading = false;
    }
  }

  async function doExport(format: "csv" | "json" | "md") {
    exportOpen = false;
    await exportRange(
      format,
      bounds.start,
      bounds.end,
      filterProjectId,
      filterUnassigned,
      filter === "all" ? "all-projects" : filterLabel.toLowerCase().replace(/\s+/g, "-"),
    );
  }

  $effect(() => {
    $refreshTick;
    filter;
    rangeId;
    customStart;
    customEnd;
    void load();
  });

  $effect(() => {
    if (report) {
      // keep the latest goal line in sync after settings changes
      $dailyGoal;
    }
  });

  function onWindowClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest(".popover-wrap")) {
      filterOpen = false;
      exportOpen = false;
    }
  }
</script>

<svelte:window onclick={onWindowClick} />

<div class="page reports">
  <header class="page-head reveal">
    <div class="eyebrow">Reports</div>
  </header>

  <div class="controls">
    <Segmented
      options={ranges}
      value={rangeId}
      onchange={(value) => (rangeId = value as RangeId)}
    />

    <div class="popover-wrap">
      <button class="pill-button" onclick={() => (filterOpen = !filterOpen)}>
        {#if filterProjectId != null}
          <span
            class="dot"
            style:background={$activeProjects.find((p) => p.id === filterProjectId)?.color ?? "var(--muted-2)"}
          ></span>
        {:else}
          <span class="dot empty"></span>
        {/if}
        <span>{filterLabel}</span>
        <Icon name="chevron-down" size={12} />
      </button>
      {#if filterOpen}
        <div class="menu filter-menu">
          <button
            class="menu-item"
            class:selected={filter === "all"}
            onclick={() => {
              filter = "all";
              filterOpen = false;
            }}
          >
            <span class="dot empty"></span> All projects
            {#if filter === "all"}<span class="check"><Icon name="check" size={13} /></span>{/if}
          </button>
          <button
            class="menu-item"
            class:selected={filter === "unassigned"}
            onclick={() => {
              filter = "unassigned";
              filterOpen = false;
            }}
          >
            <span class="dot empty"></span> Unassigned
            {#if filter === "unassigned"}<span class="check"><Icon name="check" size={13} /></span>{/if}
          </button>
          <div class="menu-sep"></div>
          {#each $activeProjects as project (project.id)}
            <button
              class="menu-item"
              class:selected={filter === String(project.id)}
              onclick={() => {
                filter = String(project.id);
                filterOpen = false;
              }}
            >
              <span class="dot" style:background={project.color}></span>
              <span class="menu-name">{project.name}</span>
              {#if filter === String(project.id)}
                <span class="check"><Icon name="check" size={13} /></span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="popover-wrap export-wrap">
      <button class="pill-button" onclick={() => (exportOpen = !exportOpen)}>
        <Icon name="download" size={14} />
        <span>Export</span>
        <Icon name="chevron-down" size={12} />
      </button>
      {#if exportOpen}
        <div class="menu export-menu">
          <button class="menu-item" onclick={() => doExport("csv")}>CSV</button>
          <button class="menu-item" onclick={() => doExport("json")}>JSON</button>
          <button class="menu-item" onclick={() => doExport("md")}>Markdown</button>
        </div>
      {/if}
    </div>
  </div>

  {#if rangeId === "custom"}
    <div class="custom-range">
      <label class="field">
        <span class="label">From</span>
        <input class="input" type="date" bind:value={customStart} max={customEnd} />
      </label>
      <label class="field">
        <span class="label">To</span>
        <input class="input" type="date" bind:value={customEnd} min={customStart} />
      </label>
    </div>
  {/if}

  <div class="range-line">{formatRange(bounds.start, bounds.end)}</div>

  <div class="summary">
    <div class="summary-cell">
      <span class="summary-label">Total</span>
      <span class="summary-value">{formatDuration(totalSeconds)}</span>
    </div>
    <div class="summary-cell">
      <span class="summary-label">Avg / active day</span>
      <span class="summary-value">{formatDuration(average)}</span>
    </div>
    <div class="summary-cell">
      <span class="summary-label">Active days</span>
      <span class="summary-value">{activeDays}<span class="summary-of">/{dayCount}</span></span>
    </div>
    <div class="summary-cell">
      <span class="summary-label">Sessions</span>
      <span class="summary-value">{sessionCount}</span>
    </div>
  </div>

  {#if error}
    <div class="inline-error">
      <span>{error}</span>
      <button class="link" onclick={() => void load()}>Retry</button>
    </div>
  {/if}

  <section class="section-sm">
    <div class="section-head">
      <div class="eyebrow">Daily activity</div>
      {#if loading}<span class="spin"></span>{/if}
    </div>
    <ActivityChart
      data={chartData}
      height={196}
      goalSeconds={$dailyGoal?.targetSeconds ?? 0}
    />
  </section>

  <section class="section">
    <Rule />
    <div class="eyebrow">Applications</div>
    <BarList
      rows={(report?.perApp ?? []).map((row) => ({
        label: row.app.replace(/\.exe$/i, ""),
        seconds: row.seconds,
      }))}
      color="var(--accent)"
    />
  </section>

  <section class="section">
    <Rule />
    <div class="eyebrow">Projects</div>
    <BarList
      rows={(report?.perProject ?? []).map((row) => ({
        label: row.name,
        seconds: row.seconds,
        color:
          $boot?.projects.find((project) => project.id === row.projectId)?.color ??
          (row.projectId == null ? "var(--muted-2)" : "var(--accent)"),
      }))}
      empty="No project time in this range."
    />
  </section>
</div>

<style>
  .page-head {
    padding-top: 64px;
  }

  .controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-top: 14px;
  }

  .popover-wrap {
    position: relative;
  }

  .pill-button {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 36px;
    padding: 0 13px;
    border: 1px solid var(--border);
    border-radius: var(--r-pill);
    background: transparent;
    color: var(--text);
    font-size: 13px;
    white-space: nowrap;
    transition:
      background-color 150ms ease-out,
      border-color 150ms ease-out;
  }

  .pill-button:hover {
    background: var(--surface-3);
    border-color: var(--border-strong);
  }

  .filter-menu,
  .export-menu {
    top: calc(100% + 6px);
    left: 0;
    width: 230px;
  }

  .export-wrap {
    margin-left: auto;
  }

  .export-menu {
    left: auto;
    right: 0;
    width: 170px;
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

  .custom-range {
    display: flex;
    gap: 12px;
    margin-top: 14px;
    max-width: 380px;
  }

  .range-line {
    margin-top: 18px;
    color: var(--muted);
    font-size: 13px;
  }

  .summary {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 10px;
    margin-top: 16px;
    padding: 22px 24px;
    border-radius: var(--r-large);
    background: var(--surface);
  }

  .summary-cell {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .summary-label {
    color: var(--muted);
    font-size: 12px;
  }

  .summary-value {
    font-size: 26px;
    font-weight: 600;
    letter-spacing: -0.035em;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .summary-of {
    color: var(--muted-2);
    font-size: 15px;
    font-weight: 500;
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

  @media (max-width: 820px) {
    .summary {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 18px 10px;
    }
    .export-wrap {
      margin-left: 0;
    }
  }
</style>
