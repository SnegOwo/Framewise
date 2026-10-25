<script lang="ts">
  import { formatDuration } from "../format";

  let {
    rows,
    empty = "Nothing tracked yet.",
    color = null,
  }: {
    rows: { label: string; seconds: number; color?: string }[];
    empty?: string;
    color?: string | null;
  } = $props();

  let max = $derived(Math.max(...rows.map((row) => row.seconds), 1));
</script>

<div class="bars">
  {#each rows as row (row.label)}
    <div class="bar-row">
      <span class="bar-name" title={row.label}>{row.label}</span>
      <span class="bar-track">
        <span
          class="bar-fill"
          style:width="{(row.seconds / max) * 100}%"
          style:background={row.color ?? color ?? "var(--accent)"}
        ></span>
      </span>
      <span class="bar-time">{formatDuration(row.seconds)}</span>
    </div>
  {/each}
  {#if rows.length === 0}
    <div class="bar-empty">{empty}</div>
  {/if}
</div>

<style>
  .bars {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .bar-row {
    display: flex;
    align-items: center;
    gap: 14px;
    min-height: 40px;
  }

  .bar-name {
    width: 170px;
    flex: 0 0 170px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 14px;
  }

  .bar-track {
    flex: 1;
    height: 7px;
    border-radius: 999px;
    background: var(--surface-3);
    overflow: hidden;
  }

  .bar-fill {
    display: block;
    height: 100%;
    border-radius: inherit;
    opacity: 0.9;
    transition: width 600ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .bar-row:hover .bar-fill {
    opacity: 1;
  }

  .bar-time {
    width: 64px;
    flex: 0 0 64px;
    text-align: right;
    color: var(--muted);
    font-size: 13px;
    font-variant-numeric: tabular-nums;
  }

  .bar-empty {
    padding: 14px 0;
    color: var(--muted-2);
    font-size: 13.5px;
  }

  @media (max-width: 640px) {
    .bar-name {
      width: 120px;
      flex-basis: 120px;
    }
  }
</style>
