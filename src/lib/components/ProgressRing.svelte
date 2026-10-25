<script lang="ts">
  import { onMount } from "svelte";

  let {
    percent,
    size = 28,
    stroke = 3,
    done = false,
    showValue = false,
  }: {
    percent: number;
    size?: number;
    stroke?: number;
    done?: boolean;
    showValue?: boolean;
  } = $props();

  let mounted = $state(false);
  // svelte-ignore state_referenced_locally
  let wasDone = $state(done);
  let pulse = $state(false);
  let pulseTimer: ReturnType<typeof setTimeout> | undefined;

  let clamped = $derived(Math.max(0, Math.min(100, Number.isFinite(percent) ? percent : 0)));
  let radius = $derived((size - stroke) / 2);
  let circumference = $derived(2 * Math.PI * radius);
  let offset = $derived(mounted ? circumference * (1 - clamped / 100) : circumference);

  onMount(() => {
    const raf = requestAnimationFrame(() => (mounted = true));
    return () => cancelAnimationFrame(raf);
  });

  $effect(() => {
    if (done && !wasDone) {
      pulse = true;
      clearTimeout(pulseTimer);
      pulseTimer = setTimeout(() => (pulse = false), 620);
    }
    wasDone = done;
    return () => clearTimeout(pulseTimer);
  });
</script>

<div
  class="progress-ring"
  class:pulse
  style:width="{size}px"
  style:height="{size}px"
  aria-hidden="true"
>
  <svg width={size} height={size} class="svg">
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke="var(--surface-3)"
      stroke-width={stroke}
    />
    <circle
      cx={size / 2}
      cy={size / 2}
      r={radius}
      fill="none"
      stroke="var(--accent)"
      stroke-width={stroke}
      stroke-linecap="round"
      stroke-dasharray={circumference}
      stroke-dashoffset={offset}
      class="arc"
    />
  </svg>
  {#if done}
    <svg class="check" width={size * 0.42} height={size * 0.42} viewBox="0 0 24 24" fill="none">
      <path
        d="m5 12.5 4.5 4.5L19 7"
        stroke="var(--accent)"
        stroke-width="2.6"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  {:else if showValue}
    <span class="value">{Math.round(clamped)}%</span>
  {/if}
</div>

<style>
  .progress-ring {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
  }

  .svg {
    transform: rotate(-90deg);
  }

  .arc {
    transition: stroke-dashoffset 900ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .progress-ring.pulse {
    animation: ring-pulse 560ms ease-out;
  }

  @keyframes ring-pulse {
    0% {
      transform: scale(1);
    }
    40% {
      transform: scale(1.08);
    }
    100% {
      transform: scale(1);
    }
  }

  .check {
    position: absolute;
    animation: check-in 240ms ease-out both;
  }

  @keyframes check-in {
    from {
      opacity: 0;
      transform: scale(0.6);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .value {
    position: absolute;
    font-size: calc(var(--ring-size, 28px) * 0.3);
    font-variant-numeric: tabular-nums;
    color: var(--text);
  }
</style>
