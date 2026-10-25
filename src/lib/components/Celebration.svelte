<script lang="ts">
  import { celebration, clearCelebration } from "../celebration";
  import { settings } from "../stores";

  const TICKS = Array.from({ length: 60 }, (_, index) => ({
    angle: index * 6,
    major: index % 5 === 0,
  }));

  const SPARKS = Array.from({ length: 24 }, (_, index) => ({
    angle: (index / 24) * 360 + (index % 2) * 3,
    distance: 116 + (index % 5) * 13,
    delay: (index % 8) * 16,
    length: 9 + (index % 3) * 5,
  }));

  let lastSoundId = 0;
  let soundTimer: ReturnType<typeof setTimeout> | undefined;

  function playSound() {
    try {
      const audio = new Audio("/sounds/goal.mp3");
      audio.volume = 0.42;
      void audio.play().catch(() => {});
    } catch {
      // Sound is a bonus; never let it break the reward.
    }
  }

  $effect(() => {
    const current = $celebration;
    if (!current) return;
    const id = current.id;

    if (id !== lastSoundId) {
      lastSoundId = id;
      if ($settings?.celebrateSound !== false) {
        clearTimeout(soundTimer);
        soundTimer = setTimeout(playSound, 220);
      }
    }

    const timer = setTimeout(() => clearCelebration(id), 3400);
    return () => {
      clearTimeout(timer);
      clearTimeout(soundTimer);
    };
  });
</script>

{#if $celebration}
  <div
    class="celebration"
    role="status"
    aria-live="polite"
    style:--reward={$celebration.tint}
  >
    {#key $celebration.id}
      <div class="glow" aria-hidden="true"></div>

      <div class="stage">
        <div class="pulse-rings" aria-hidden="true">
          <span></span>
          <span></span>
          <span></span>
        </div>

        <svg class="dial" viewBox="0 0 240 240" aria-hidden="true">
          <circle class="dial-line" cx="120" cy="120" r="100" />
          <circle class="dial-arc" cx="120" cy="120" r="100" />
          <circle class="dial-comet" cx="120" cy="120" r="100" />
          <g class="ticks">
            {#each TICKS as tick, index}
              <line
                class="tick"
                class:major={tick.major}
                x1="120"
                y1="20"
                x2="120"
                y2={tick.major ? 32 : 28}
                transform="rotate({tick.angle} 120 120)"
                style:--i={index}
              />
            {/each}
          </g>
        </svg>

        <div class="sparks" aria-hidden="true">
          {#each SPARKS as spark}
            <span
              class="spark"
              style:--a="{spark.angle}deg"
              style:--d="{spark.distance}px"
              style:--delay="{spark.delay}ms"
              style:--len="{spark.length}px"
            ></span>
          {/each}
        </div>

        <div class="medallion">
          <svg class="check" viewBox="0 0 100 100" fill="none" aria-hidden="true">
            <path
              d="M26 52l17 17 32-37"
              stroke="var(--reward)"
              stroke-width="7"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </div>
      </div>

      <div class="copy">
        <span class="eyebrow">Goal complete</span>
        <div class="title">{$celebration.title}</div>
        <div class="detail">{$celebration.detail}</div>
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
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 18px;
    pointer-events: none;
    background: color-mix(in srgb, var(--bg) 58%, transparent);
    backdrop-filter: blur(3px);
    animation:
      veil-in 320ms ease-out both,
      veil-out 360ms ease-in 2860ms forwards;
  }

  @keyframes veil-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes veil-out {
    to {
      opacity: 0;
    }
  }

  .glow {
    position: absolute;
    width: 560px;
    height: 560px;
    border-radius: 999px;
    background: radial-gradient(
      circle,
      color-mix(in srgb, var(--reward) 18%, transparent),
      transparent 62%
    );
    opacity: 0;
    animation: glow-in 1000ms ease-out forwards;
  }

  @keyframes glow-in {
    0% {
      opacity: 0;
      transform: scale(0.7);
    }
    45% {
      opacity: 1;
    }
    100% {
      opacity: 0.6;
      transform: scale(1);
    }
  }

  .stage {
    position: relative;
    width: 240px;
    height: 240px;
    animation: stage-in 560ms cubic-bezier(0.22, 1, 0.36, 1) both;
  }

  @keyframes stage-in {
    0% {
      opacity: 0;
      transform: scale(0.84);
    }
    70% {
      opacity: 1;
      transform: scale(1.02);
    }
    100% {
      opacity: 1;
      transform: scale(1);
    }
  }

  .dial {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }

  .dial-line {
    fill: none;
    stroke: var(--border);
    stroke-width: 1;
  }

  .dial-arc {
    fill: none;
    stroke: var(--reward);
    stroke-width: 2;
    stroke-linecap: round;
    stroke-dasharray: 628;
    stroke-dashoffset: 628;
    transform: rotate(-90deg);
    transform-origin: 120px 120px;
    animation: dial-draw 760ms cubic-bezier(0.22, 1, 0.36, 1) 90ms forwards;
  }

  @keyframes dial-draw {
    to {
      stroke-dashoffset: 0;
    }
  }

  .dial-comet {
    fill: none;
    stroke: var(--reward);
    stroke-width: 2.5;
    stroke-linecap: round;
    stroke-dasharray: 28 600;
    transform: rotate(-90deg);
    transform-origin: 120px 120px;
    opacity: 0;
    animation: comet 1200ms cubic-bezier(0.4, 0, 0.2, 1) 120ms;
  }

  @keyframes comet {
    0% {
      opacity: 0.9;
      stroke-dashoffset: 628;
    }
    70% {
      opacity: 0.55;
    }
    100% {
      opacity: 0;
      stroke-dashoffset: 0;
    }
  }

  .ticks {
    transform-origin: 120px 120px;
    animation: ticks-settle 880ms cubic-bezier(0.16, 1, 0.3, 1) both;
  }

  @keyframes ticks-settle {
    from {
      transform: rotate(-14deg);
    }
    to {
      transform: rotate(0deg);
    }
  }

  .tick {
    stroke: var(--muted-2);
    stroke-width: 1.1;
    opacity: 0;
    animation: tick-in 240ms ease-out calc(var(--i) * 7ms) forwards;
  }

  .tick.major {
    stroke: var(--muted);
  }

  @keyframes tick-in {
    to {
      opacity: 0.65;
    }
  }

  .pulse-rings span {
    position: absolute;
    inset: 40px;
    border: 1px solid var(--reward);
    border-radius: 999px;
    opacity: 0;
    animation: pulse-ring 1050ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .pulse-rings span:nth-child(2) {
    animation-delay: 130ms;
  }

  .pulse-rings span:nth-child(3) {
    animation-delay: 260ms;
  }

  @keyframes pulse-ring {
    0% {
      opacity: 0.38;
      transform: scale(0.9);
    }
    100% {
      opacity: 0;
      transform: scale(1.42);
    }
  }

  .spark {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 2.5px;
    height: var(--len);
    border-radius: 999px;
    background: var(--reward);
    opacity: 0;
    transform-origin: 50% 0;
    animation: spark 900ms cubic-bezier(0.16, 1, 0.3, 1) var(--delay) forwards;
  }

  @keyframes spark {
    0% {
      opacity: 0;
      transform: rotate(var(--a)) translateY(-52px) scaleY(0.4);
    }
    18% {
      opacity: 1;
    }
    100% {
      opacity: 0;
      transform: rotate(var(--a)) translateY(calc(-1 * var(--d))) scaleY(1);
    }
  }

  .medallion {
    position: absolute;
    inset: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface);
    box-shadow: 0 20px 70px rgba(0, 0, 0, 0.28);
  }

  .medallion::before {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: radial-gradient(
      circle at 50% 58%,
      color-mix(in srgb, var(--reward) 16%, transparent),
      transparent 68%
    );
    opacity: 0;
    animation: wash-in 720ms ease-out 260ms forwards;
  }

  @keyframes wash-in {
    to {
      opacity: 1;
    }
  }

  .check {
    position: relative;
    z-index: 1;
    width: 62px;
    height: 62px;
  }

  .check path {
    stroke-dasharray: 90;
    stroke-dashoffset: 90;
    animation: check-draw 440ms cubic-bezier(0.22, 1, 0.36, 1) 480ms forwards;
  }

  @keyframes check-draw {
    to {
      stroke-dashoffset: 0;
    }
  }

  .copy {
    position: relative;
    z-index: 2;
    text-align: center;
    animation: copy-in 540ms cubic-bezier(0.22, 1, 0.36, 1) 160ms both;
  }

  @keyframes copy-in {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }

  .eyebrow {
    display: block;
    color: var(--muted);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  .title {
    margin-top: 8px;
    font-size: 28px;
    font-weight: 600;
    line-height: 1.08;
    letter-spacing: -0.035em;
  }

  .detail {
    margin-top: 5px;
    color: var(--muted);
    font-size: 13.5px;
  }

  @media (prefers-reduced-motion: reduce) {
    .celebration {
      animation: none;
      background: color-mix(in srgb, var(--bg) 58%, transparent);
      backdrop-filter: none;
    }

    .glow,
    .pulse-rings,
    .sparks,
    .dial-comet {
      display: none;
    }

    .stage,
    .copy,
    .medallion,
    .medallion::before,
    .ticks,
    .dial-arc,
    .tick,
    .check path {
      animation: none;
    }

    .dial-arc,
    .check path {
      stroke-dashoffset: 0;
    }

    .tick {
      opacity: 0.55;
    }

    .medallion::before {
      opacity: 1;
    }
  }
</style>
