<script lang="ts">
  import type { HeatmapCell } from "../types";
  import {
    addDays,
    formatDateLong,
    formatDuration,
    formatMonth,
    mondayOf,
    parseDay,
    toDay,
    todayStr,
  } from "../format";

  let {
    cells,
    endDay = todayStr(),
    weeks = 53,
    onselect,
  }: {
    cells: HeatmapCell[];
    endDay?: string;
    weeks?: number;
    onselect?: (day: string) => void;
  } = $props();

  type Cell = {
    day: string;
    seconds: number;
    week: number;
    row: number;
    future: boolean;
  };

  let hovered = $state<{ cell: Cell; x: number; y: number } | null>(null);

  let model = $derived.by(() => {
    const map = new Map(cells.map((cell) => [cell.day, cell.seconds]));
    const endWeek = parseDay(mondayOf(endDay));
    const start = addDays(endWeek, -(weeks - 1) * 7);
    const rows: Cell[][] = [];
    const months: { label: string; week: number }[] = [];
    let lastMonth = -1;
    for (let w = 0; w < weeks; w++) {
      const row: Cell[] = [];
      for (let d = 0; d < 7; d++) {
        const date = addDays(start, w * 7 + d);
        const day = toDay(date);
        row.push({
          day,
          seconds: map.get(day) ?? 0,
          week: w,
          row: d,
          future: day > endDay,
        });
      }
      const anchor = addDays(start, w * 7);
      if (anchor.getMonth() !== lastMonth) {
        months.push({ label: formatMonth(toDay(anchor)), week: w });
        lastMonth = anchor.getMonth();
      }
      rows.push(row);
    }
    const flat = rows.flat();
    const total = flat.reduce((sum, cell) => sum + cell.seconds, 0);
    const activeDays = flat.filter((cell) => cell.seconds > 0).length;
    return { rows, months, total, activeDays };
  });

  function level(seconds: number): number {
    if (seconds <= 0) return 0;
    if (seconds < 1800) return 1;
    if (seconds < 7200) return 2;
    if (seconds < 14400) return 3;
    return 4;
  }

  function show(event: MouseEvent, cell: Cell) {
    if (cell.future) return;
    hovered = { cell, x: event.clientX, y: event.clientY - 10 };
  }

  function pick(cell: Cell) {
    if (cell.future || !onselect) return;
    onselect(cell.day);
  }
</script>

<div class="heat" style:--pitch="14px">
  <div class="heat-scroll">
    <div class="heat-grid" style:width="{weeks * 14 + 30}px">
      <div class="months">
        {#each model.months as month}
          <span class="month" style:left="{month.week * 14 + 30}px">{month.label}</span>
        {/each}
      </div>
      <div class="body">
        <div class="weekdays">
          <span style:top="0px">Mon</span>
          <span style:top="28px">Wed</span>
          <span style:top="56px">Fri</span>
        </div>
        <div class="cells">
          {#each model.rows as week}
            {#each week as cell}
              <button
                class="cell"
                class:future={cell.future}
                style:background={cell.seconds > 0 ? `var(--heat-${level(cell.seconds)})` : undefined}
                style:grid-column={cell.week + 1}
                style:grid-row={cell.row + 1}
                style:animation-delay="{Math.min(560, cell.week * 7)}ms"
                onmouseenter={(event) => show(event, cell)}
                onmouseleave={() => (hovered = null)}
                onclick={() => pick(cell)}
                tabindex={cell.future ? -1 : 0}
                aria-label={`${formatDateLong(cell.day)}: ${formatDuration(cell.seconds)}`}
              ></button>
            {/each}
          {/each}
        </div>
      </div>
    </div>
  </div>

  <div class="heat-foot">
    <span class="legend">
      <span>Less</span>
      {#each [0, 1, 2, 3, 4] as step}
        <span
          class="legend-cell"
          style:background={step === 0 ? "var(--heat-0)" : `var(--heat-${step})`}
        ></span>
      {/each}
      <span>More</span>
    </span>
    <span class="meta">{model.activeDays} active {model.activeDays === 1 ? "day" : "days"}</span>
  </div>

  {#if hovered}
    <div class="heat-tooltip fixed" style:left="{hovered.x}px" style:top="{hovered.y}px">
      <div class="tip-muted">{formatDateLong(hovered.cell.day)}</div>
      <div>
        {hovered.cell.seconds > 0 ? `${formatDuration(hovered.cell.seconds)} tracked` : "No tracked time"}
      </div>
    </div>
  {/if}
</div>

<style>
  .heat {
    width: 100%;
  }

  .heat-scroll {
    overflow-x: auto;
    overflow-y: hidden;
    padding-bottom: 4px;
  }

  .heat-grid {
    position: relative;
    user-select: none;
  }

  .months {
    position: relative;
    height: 18px;
  }

  .month {
    position: absolute;
    top: 0;
    color: var(--muted-2);
    font-size: 10px;
    white-space: nowrap;
  }

  .body {
    display: flex;
    align-items: flex-start;
    gap: 4px;
  }

  .weekdays {
    position: relative;
    width: 26px;
    height: 88px;
    flex: 0 0 26px;
  }

  .weekdays span {
    position: absolute;
    right: 0;
    color: var(--muted-2);
    font-size: 9.5px;
    line-height: 11px;
  }

  .cells {
    display: grid;
    grid-auto-flow: column;
    grid-template-rows: repeat(7, 11px);
    grid-auto-columns: 11px;
    gap: 3px;
  }

  .cell {
    width: 11px;
    height: 11px;
    padding: 0;
    border: 0;
    border-radius: 3px;
    background: var(--heat-0);
    animation: cell-in 220ms ease-out both;
  }

  .cell.future {
    opacity: 0.3;
  }

  .cell:hover {
    transform: scale(1.22);
    outline: 1px solid var(--accent);
    outline-offset: 1px;
  }

  @keyframes cell-in {
    from {
      opacity: 0;
      transform: scale(0.6);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .heat-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 12px;
    color: var(--muted-2);
    font-size: 11.5px;
  }

  .legend {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .legend-cell {
    width: 10px;
    height: 10px;
    border-radius: 3px;
  }

  .meta {
    white-space: nowrap;
  }
</style>
