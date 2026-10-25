<script lang="ts">
  import Icon from "./Icon.svelte";
  import CountUp from "./CountUp.svelte";

  let {
    icon,
    label,
    value,
    seconds = null,
    formatter,
    tone = "neutral",
  }: {
    icon: string;
    label: string;
    value?: string;
    seconds?: number | null;
    formatter?: (value: number) => string;
    tone?: "neutral" | "accent";
  } = $props();
</script>

<div class="tile">
  <span class="icon-chip" class:accent={tone === "accent"}>
    <Icon name={icon} size={15} />
  </span>
  <div class="tile-body">
    <div class="tile-value">
      {#if seconds != null && formatter}
        <CountUp value={seconds} {formatter} duration={380} />
      {:else}
        {value ?? ""}
      {/if}
    </div>
    <div class="tile-label">{label}</div>
  </div>
</div>

<style>
  .tile {
    justify-content: space-between;
    min-height: 124px;
  }

  .tile-body {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .tile-value {
    font-size: 30px;
    font-weight: 600;
    letter-spacing: -0.035em;
    line-height: 1.1;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .tile-label {
    color: var(--muted);
    font-size: 13px;
  }

  @media (max-width: 420px) {
    .tile-value {
      font-size: 26px;
    }
  }
</style>
