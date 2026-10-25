<script lang="ts">
  import type { DayTotal } from "../types";
  import { formatDate, formatDuration, formatWeekday } from "../format";

  let {
    data,
    height = 180,
    goalSeconds = 0,
    emptyText = "Nothing tracked in this range",
  }: {
    data: DayTotal[];
    height?: number;
    goalSeconds?: number;
    emptyText?: string;
  } = $props();

  let hovered = $state<{ index: number; x: number; y: number } | null>(null);

  let max = $derived(Math.max(goalSeconds, ...data.map((row) => row.seconds), 1));
  let goalPercent = $derived(goalSeconds > 0 ? (goalSeconds / max) * 100 : 0);
  let allZero = $derived(data.length > 0 && data.every((row) => row.seconds === 0));
  let step = $derived(data.length > 24 ? 4 : data.length > 14 ? 2 : 1);

  function show(event: MouseEvent, index: number) {
    hovered = { index, x: event.clientX, y: event.clientY - 12 };
  }
</script>

<div class="chart" style:height="{height + 26}px">
  <div class="plot" style:height="{height}px">
    <span class="grid" style:top="0"></span>
    <span class="grid" style:top="50%"></span>
    <span class="grid" style:top="100%"></span>

    {#if goalSeconds > 0}
      <div
        class="goal"
        class:below={goalPercent > 78}
        style:bottom="{Math.min(100, goalPercent)}%"
      >
        <span class="goal-label">{formatDuration(goalSeconds)} goal</span>
      </div>
    {/if}

    <div class="bars">
      {#each data as day, index (day.day)}
        <button
          class="slot"
          onmouseenter={(event) => show(event, index)}
          onmouseleave={() => (hovered = null)}
          onfocus={(event) => {
            const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
            hovered = { index, x: rect.left + rect.width / 2, y: rect.top - 4 };
          }}
          onblur={() => (hovered = null)}
          aria-label={`${formatDate(day.day)}: ${formatDuration(day.seconds)}`}
        >
          <span
            class="bar"
            class:ghost={allZero}
            class:empty={!allZero && day.seconds === 0}
            style:height="{allZero
              ? 100
              : day.seconds > 0
                ? Math.max(2, (day.seconds / max) * 100)
                : 2}%"
          ></span>
        </button>
      {/each}
    </div>

    {#if allZero}
      <div class="chart-empty">{emptyText}</div>
    {/if}
  </div>

  <div class="labels">
    {#each data as day, index (day.day)}
      <span class="label">
        {#if index % step === 0 || index === data.length - 1}
          {data.length > 16 ? formatDate(day.day) : `${formatWeekday(day.day)} ${formatDate(day.day)}`}
        {/if}
      </span>
    {/each}
  </div>

  {#if hovered && data[hovered.index]}
    <div class="tooltip fixed" style:left="{hovered.x}px" style:top="{hovered.y}px">
      <div>
        {formatDate(data[hovered.index].day, {
          weekday: "short",
          month: "short",
          day: "numeric",
        })}
      </div>
      <div class="tip-muted">{formatDuration(data[hovered.index].seconds)}</div>
    </div>
  {/if}
</div>

<style>
  .chart {
    position: relative;
    width: 100%;
  }

  .plot {
    position: relative;
  }

  .grid {
    position: absolute;
    left: 0;
    right: 0;
    height: 0;
    border-top: 1px dashed var(--guide);
    pointer-events: none;
  }

  .goal {
    position: absolute;
    left: 0;
    right: 0;
    height: 0;
    border-top: 1px dashed var(--accent-line);
    pointer-events: none;
  }

  .goal-label {
    position: absolute;
    right: 0;
    bottom: 4px;
    padding: 0 4px;
    background: var(--bg);
    color: var(--accent-text);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .goal.below .goal-label {
    bottom: auto;
    top: 4px;
  }

  .bars {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: flex-end;
    gap: 3px;
  }

  .slot {
    display: flex;
    align-items: flex-end;
    flex: 1 1 0;
    min-width: 2px;
    height: 100%;
    padding: 0;
    border: 0;
    background: transparent;
  }

  .bar {
    width: 100%;
    max-width: 42px;
    margin: 0 auto;
    border-radius: 6px 6px 2px 2px;
    background: var(--accent);
    opacity: 0.9;
    transform-origin: bottom;
    transition:
      opacity 130ms ease-out,
      background-color 130ms ease-out;
  }

  .bar.empty {
    background: var(--surface-3);
    opacity: 1;
    border-radius: 2px;
  }

  .bar.ghost {
    opacity: 0.08;
  }

  .slot:hover .bar:not(.empty) {
    opacity: 1;
  }

  .labels {
    display: flex;
    gap: 3px;
    margin-top: 6px;
  }

  .label {
    flex: 1 1 0;
    min-width: 2px;
    overflow: visible;
    color: var(--muted-2);
    font-size: 10px;
    text-align: center;
    white-space: nowrap;
  }

  .chart-empty {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--muted-2);
    font-size: 13px;
    pointer-events: none;
  }
</style>
