<script lang="ts">
  import { tick } from "svelte";
  import Logo from "./Logo.svelte";
  import Icon from "./Icon.svelte";
  import { PAGES, navigate, page } from "../router";
  import { boot, loadBootstrap, toast } from "../stores";
  import { api, errMessage } from "../api";
  import { applyTheme, type ThemeSetting } from "../theme";

  let nav: HTMLElement | undefined = $state();
  let thumb = $state({ left: 0, width: 0, ready: false });
  let popping = $state(false);

  let theme = $derived(($boot?.settings.theme ?? "dark") as ThemeSetting);
  let isLight = $derived(theme === "light");

  function measure() {
    if (!nav) return;
    const index = PAGES.findIndex((item) => item.id === $page);
    const button = nav.querySelectorAll<HTMLButtonElement>("button")[index];
    if (!button) return;
    thumb = {
      left: button.offsetLeft + 6,
      width: Math.max(8, button.offsetWidth - 12),
      ready: true,
    };
  }

  $effect(() => {
    $page;
    $boot;
    tick().then(measure);
  });

  $effect(() => {
    if (!nav) return;
    const observer = new ResizeObserver(measure);
    observer.observe(nav);
    return () => observer.disconnect();
  });

  async function toggleTheme() {
    const next: ThemeSetting = isLight ? "dark" : "light";
    applyTheme(next);
    popping = true;
    try {
      await api.updateSettings({ theme: next });
      await loadBootstrap();
    } catch (error) {
      toast(errMessage(error), "error");
    }
    window.setTimeout(() => (popping = false), 460);
  }
</script>

<header class="topbar">
  <div class="inner">
    <button class="brand" onclick={() => navigate("dashboard")} aria-label="Framewise dashboard">
      <Logo size={26} />
      <span class="wordmark">Framewise</span>
    </button>

    <nav class="nav" bind:this={nav} aria-label="Pages">
      {#if thumb.ready}
        <span
          class="nav-thumb"
          style:left="{thumb.left}px"
          style:width="{thumb.width}px"
          aria-hidden="true"
        ></span>
      {/if}
      {#each PAGES as item (item.id)}
        <button
          class:active={$page === item.id}
          aria-current={$page === item.id ? "page" : undefined}
          onclick={() => navigate(item.id)}
        >
          {item.label}
        </button>
      {/each}
    </nav>

    <button
      class="theme"
      class:popping
      onclick={toggleTheme}
      title={isLight ? "Switch to dark" : "Switch to light"}
      aria-label={isLight ? "Switch to dark" : "Switch to light"}
    >
      <span class="theme-icon" class:light={isLight}>
        <span class="glyph sun">
          <span class="ambient sun-ambient"><Icon name="sun" size={17} /></span>
        </span>
        <span class="glyph moon">
          <span class="ambient moon-ambient"><Icon name="moon" size={17} /></span>
        </span>
      </span>
    </button>
  </div>
</header>

<style>
  .topbar {
    position: relative;
    z-index: 40;
    flex: 0 0 auto;
    border-bottom: 1px dashed var(--guide);
    background: color-mix(in srgb, var(--bg) 86%, transparent);
    backdrop-filter: blur(14px);
  }

  .inner {
    display: flex;
    align-items: center;
    gap: 22px;
    width: 100%;
    max-width: var(--content-width);
    height: 64px;
    margin: 0 auto;
    padding: 0 24px;
  }

  .brand {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--text);
  }

  .wordmark {
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.035em;
  }

  .nav {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0;
    margin-left: 6px;
    height: 100%;
  }

  .nav button {
    position: relative;
    z-index: 1;
    height: 32px;
    padding: 0 10px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--muted);
    font-size: 14px;
    transition: color 150ms ease-out;
  }

  .nav button:hover {
    color: var(--text);
  }

  .nav button.active {
    color: var(--text);
  }

  .nav-thumb {
    position: absolute;
    bottom: 5px;
    height: 1.5px;
    border-radius: 999px;
    background: var(--accent);
    transition:
      left 240ms cubic-bezier(0.32, 0.72, 0, 1),
      width 240ms cubic-bezier(0.32, 0.72, 0, 1);
  }

  .theme {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    margin-left: auto;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--muted);
    transition:
      background-color 150ms ease-out,
      color 150ms ease-out;
  }

  .theme:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .theme-icon {
    position: relative;
    width: 18px;
    height: 18px;
  }

  .glyph {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      opacity 260ms ease,
      transform 420ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  .glyph.sun {
    opacity: 1;
    transform: rotate(0deg) scale(1);
  }

  .glyph.moon {
    opacity: 0;
    transform: rotate(-70deg) scale(0.5);
  }

  .theme-icon.light .glyph.sun {
    opacity: 0;
    transform: rotate(70deg) scale(0.5);
  }

  .theme-icon.light .glyph.moon {
    opacity: 1;
    transform: rotate(0deg) scale(1);
  }

  .ambient {
    display: flex;
  }

  .sun-ambient {
    animation: sun-turn 34s linear infinite;
  }

  .moon-ambient {
    animation: moon-bob 5.5s ease-in-out infinite;
  }

  @keyframes sun-turn {
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes moon-bob {
    0%,
    100% {
      transform: rotate(-5deg) translateY(0);
    }
    50% {
      transform: rotate(5deg) translateY(-1px);
    }
  }

  .theme.popping .theme-icon {
    animation: theme-pop 460ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  @keyframes theme-pop {
    0% {
      transform: scale(1) rotate(0);
    }
    35% {
      transform: scale(0.82) rotate(-10deg);
    }
    70% {
      transform: scale(1.08) rotate(5deg);
    }
    100% {
      transform: scale(1) rotate(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .sun-ambient,
    .moon-ambient,
    .theme.popping .theme-icon {
      animation: none;
    }
  }

  @media (max-width: 640px) {
    .inner {
      gap: 10px;
      padding: 0 16px;
    }
    .nav button {
      padding: 0 7px;
      font-size: 13px;
    }
    .wordmark {
      display: none;
    }
  }
</style>
