<script lang="ts">
  import type { Goal, Project } from "../types";
  import ProgressRing from "./ProgressRing.svelte";
  import Icon from "./Icon.svelte";
  import { formatDuration } from "../format";
  import { clampPercent } from "../format";

  let {
    goal,
    seconds,
    project = null,
    onchange,
  }: {
    goal: Goal;
    seconds: number;
    project?: Project | null;
    onchange: (targetSeconds: number) => void;
  } = $props();

  const step = 15 * 60;

  let editing = $state(false);
  // svelte-ignore state_referenced_locally
  let draft = $state(goal.targetSeconds);
  let container: HTMLDivElement | undefined = $state();

  let percent = $derived(goal.targetSeconds > 0 ? (seconds / goal.targetSeconds) * 100 : 0);
  let done = $derived(goal.targetSeconds > 0 && seconds >= goal.targetSeconds);
  let periodLabel = $derived(goal.period === "daily" ? "Daily" : "Weekly");
  let scopeLabel = $derived(
    goal.scope === "project" ? project?.name ?? goal.projectName ?? "Project" : null,
  );

  function start() {
    draft = goal.targetSeconds;
    editing = true;
  }

  function adjust(delta: number) {
    draft = Math.max(step, draft + delta);
    onchange(draft);
  }

  function onWindowClick(event: MouseEvent) {
    if (editing && container && !container.contains(event.target as Node)) editing = false;
  }
</script>

<svelte:window onclick={onWindowClick} />

<div class="goal" bind:this={container}>
  <ProgressRing percent={percent} {done} size={26} stroke={3} />

  <div class="goal-main">
    <div class="goal-title">
      {#if scopeLabel && project}
        <span class="dot" style:background={project.color}></span>
      {/if}
      <span>{periodLabel}{scopeLabel ? ` · ${scopeLabel}` : ""}</span>
      <span class="goal-target">{formatDuration(goal.targetSeconds)}</span>
    </div>
    <div class="track" aria-hidden="true">
      <span
        class="fill"
        class:done
        style:transform="scaleX({clampPercent(percent) / 100})"
      ></span>
    </div>
  </div>

  {#if editing}
    <div class="stepper">
      <button onclick={() => adjust(-step)} aria-label="Decrease goal">
        <Icon name="minus" size={13} />
      </button>
      <span class="value">{formatDuration(draft)}</span>
      <button onclick={() => adjust(step)} aria-label="Increase goal">
        <Icon name="plus" size={13} />
      </button>
    </div>
  {:else}
    <button class="goal-value" onclick={start} title="Edit goal">
      <span class="now">{formatDuration(seconds)}</span>
      <span class="pct">{Math.round(clampPercent(percent))}%</span>
    </button>
  {/if}
</div>

<style>
  .goal {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 52px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border-subtle);
  }

  .goal:last-child {
    border-bottom: 0;
  }

  .goal-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .goal-title {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 14px;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .goal-target {
    color: var(--muted-2);
    font-size: 12.5px;
  }

  .goal-value {
    display: inline-flex;
    align-items: baseline;
    gap: 8px;
    padding: 4px 8px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--muted);
    font-size: 13.5px;
    font-variant-numeric: tabular-nums;
    transition: background-color 140ms ease-out;
  }

  .goal-value:hover {
    background: var(--surface-3);
  }

  .goal-value .now {
    color: var(--text);
  }

  .pct {
    color: var(--muted-2);
    font-size: 11.5px;
  }
</style>
