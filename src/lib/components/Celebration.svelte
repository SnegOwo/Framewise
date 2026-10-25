<script lang="ts">
  import { celebration, clearCelebration } from "../celebration";

  const particles = Array.from({ length: 30 }, (_, index) => {
    const angle = (index / 30) * Math.PI * 2 + (index % 3) * 0.08;
    const distance = 92 + (index % 6) * 26;
    return {
      x: Math.cos(angle) * distance,
      y: Math.sin(angle) * distance,
      delay: (index % 9) * 20,
      size: 3 + (index % 3),
      tone: index % 4,
    };
  });

  $effect(() => {
    const current = $celebration;
    if (!current) return;
    const id = current.id;
    const timer = setTimeout(() => clearCelebration(id), 3000);
    return () => clearTimeout(timer);
  });
</script>

{#if $celebration}
  <div class="celebration" role="status" aria-live="polite">
    {#key $celebration.id}
      <div class="reward">
        <div class="burst" aria-hidden="true">
          {#each particles as particle}
            <span
              class="particle tone-{particle.tone}"
              style:--x="{particle.x}px"
              style:--y="{particle.y}px"
              style:--delay="{particle.delay}ms"
              style:--size="{particle.size}px"
            ></span>
          {/each}
        </div>

        <div class="medal">
          <span class="halo"></span>
          <svg viewBox="0 0 120 120" width="78" height="78" fill="none" aria-hidden="true">
            <circle
              cx="60"
              cy="60"
              r="52"
              stroke="var(--surface-3)"
              stroke-width="3"
            />
            <circle
              class="medal-arc"
              cx="60"
              cy="60"
              r="52"
              stroke="var(--accent)"
              stroke-width="3"
              stroke-linecap="round"
            />
            <path
              class="medal-check"
              d="m38 61 15 15 30-32"
              stroke="var(--accent)"
              stroke-width="6"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </div>

        <div class="reward-copy">
          <div class="reward-title">{$celebration.title}</div>
          <div class="reward-detail">{$celebration.detail}</div>
        </div>
      </div>
    {/key}
  </div>
{/if}

<style>
  .celebration {
    position: fixed;
    inset: 0;
    z-index: 120;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
  }

  .reward {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    min-width: 270px;
    padding: 28px 32px 24px;
    border: 1px solid var(--border);
    border-radius: var(--r-large);
    background: var(--surface);
    box-shadow: var(--shadow);
    animation:
      reward-in 520ms cubic-bezier(0.22, 1, 0.36, 1) both,
      reward-out 380ms ease-in 2500ms forwards;
  }

  @keyframes reward-in {
    0% {
      opacity: 0;
      transform: translateY(12px) scale(0.9);
    }
    65% {
      opacity: 1;
      transform: translateY(-2px) scale(1.015);
    }
    100% {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @keyframes reward-out {
    to {
      opacity: 0;
      transform: translateY(-6px) scale(0.97);
    }
  }

  .medal {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 78px;
    height: 78px;
  }

  .halo {
    position: absolute;
    inset: 2px;
    border-radius: 999px;
    background: var(--accent-soft);
    opacity: 0;
    animation: halo 900ms ease-out 320ms;
  }

  @keyframes halo {
    0% {
      opacity: 0;
      transform: scale(0.72);
    }
    45% {
      opacity: 1;
    }
    100% {
      opacity: 0;
      transform: scale(1.28);
    }
  }

  .medal-arc {
    stroke-dasharray: 327;
    stroke-dashoffset: 327;
    transform: rotate(-90deg);
    transform-origin: 60px 60px;
    animation: arc-draw 780ms cubic-bezier(0.22, 1, 0.36, 1) 90ms forwards;
  }

  @keyframes arc-draw {
    to {
      stroke-dashoffset: 0;
    }
  }

  .medal-check {
    stroke-dasharray: 82;
    stroke-dashoffset: 82;
    animation: check-draw 460ms cubic-bezier(0.22, 1, 0.36, 1) 520ms forwards;
  }

  @keyframes check-draw {
    to {
      stroke-dashoffset: 0;
    }
  }

  .reward-copy {
    position: relative;
    z-index: 1;
    text-align: center;
  }

  .reward-title {
    font-size: 19px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  .reward-detail {
    margin-top: 3px;
    color: var(--muted);
    font-size: 12.5px;
  }

  .burst {
    position: absolute;
    inset: 0;
    overflow: visible;
  }

  .particle {
    position: absolute;
    left: 50%;
    top: 50%;
    width: var(--size);
    height: var(--size);
    border-radius: 999px;
    animation: burst 900ms cubic-bezier(0.16, 1, 0.3, 1) var(--delay) both;
  }

  @keyframes burst {
    0% {
      transform: translate(-50%, -50%) scale(0.3);
      opacity: 0;
    }
    14% {
      opacity: 1;
    }
    100% {
      transform: translate(calc(-50% + var(--x)), calc(-50% + var(--y))) scale(1);
      opacity: 0;
    }
  }

  .tone-0 {
    background: var(--accent);
  }

  .tone-1 {
    background: color-mix(in srgb, var(--accent) 62%, var(--text));
  }

  .tone-2 {
    background: var(--accent);
    opacity: 0.72;
  }

  .tone-3 {
    background: var(--text);
    opacity: 0.45;
  }

  @media (prefers-reduced-motion: reduce) {
    .reward,
    .particle,
    .halo,
    .medal-arc,
    .medal-check {
      animation: none;
    }

    .particle {
      display: none;
    }

    .medal-arc,
    .medal-check {
      stroke-dashoffset: 0;
    }
  }
</style>
