<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { check, type Update } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import Logo from "./lib/components/Logo.svelte";
  import TopBar from "./lib/components/TopBar.svelte";
  import Dock from "./lib/components/Dock.svelte";
  import Toasts from "./lib/components/Toasts.svelte";
  import Mini from "./lib/components/Mini.svelte";
  import AppInfo from "./lib/components/AppInfo.svelte";
  import Celebration from "./lib/components/Celebration.svelte";
  import ProjectToolsDialog from "./lib/components/ProjectToolsDialog.svelte";
  import Onboarding from "./lib/components/Onboarding.svelte";
  import UpdatePrompt from "./lib/components/UpdatePrompt.svelte";
  import Dashboard from "./lib/pages/Dashboard.svelte";
  import Projects from "./lib/pages/Projects.svelte";
  import Reports from "./lib/pages/Reports.svelte";
  import Sessions from "./lib/pages/Sessions.svelte";
  import Settings from "./lib/pages/Settings.svelte";
  import {
    PAGES,
    commandOpen,
    navigate,
    page,
    requestNewProject,
    type Page,
  } from "./lib/router";
  import {
    boot,
    bumpRefresh,
    fatalError,
    goalSeconds,
    loadBootstrap,
    loading,
    refreshStatus,
    status,
    toast,
  } from "./lib/stores";
  import { errMessage } from "./lib/api";
  import { applyTheme, watchSystemTheme, type ThemeSetting } from "./lib/theme";
  import { motionDuration } from "./lib/motion";
  import { celebrate } from "./lib/celebration";
  import { formatDuration } from "./lib/format";

  let bootTheme = $state<ThemeSetting | null>(null);
  let updateInfo = $state<{ version: string; notes: string } | null>(null);
  let updateInstalling = $state(false);
  let pendingUpdate: Update | null = null;
  let updateCheckStarted = false;
  const LAST_UPDATE_KEY = "framewise-last-update-check";

  const isMini = (() => {
    try {
      return getCurrentWindow().label === "mini";
    } catch {
      return new URLSearchParams(window.location.search).get("mini") === "1";
    }
  })();

  $effect(() => {
    const setting = $boot?.settings.theme as ThemeSetting | undefined;
    if (!setting || setting === bootTheme) return;
    bootTheme = setting;
    applyTheme(setting, false);
  });

  async function maybeCheckUpdates() {
    if (isMini || updateCheckStarted) return;
    const settings = $boot?.settings;
    if (
      !settings ||
      !settings.autoUpdateCheck ||
      settings.portable ||
      !settings.onboardingComplete
    ) {
      return;
    }
    updateCheckStarted = true;
    let last = 0;
    try {
      last = Number(localStorage.getItem(LAST_UPDATE_KEY) ?? "0");
    } catch {
      last = 0;
    }
    if (Date.now() - last < 30 * 24 * 60 * 60 * 1000) return;
    try {
      localStorage.setItem(LAST_UPDATE_KEY, String(Date.now()));
    } catch {
      // ignore
    }
    try {
      const update = await check();
      if (update) {
        pendingUpdate = update;
        updateInfo = {
          version: update.version,
          notes: update.body ?? "",
        };
      }
    } catch {
      // Offline or no signed release yet. Leave the app quiet.
    }
  }

  $effect(() => {
    if ($boot) void maybeCheckUpdates();
  });

  async function installUpdate() {
    if (!pendingUpdate || updateInstalling) return;
    updateInstalling = true;
    try {
      await pendingUpdate.downloadAndInstall();
      await relaunch();
    } catch (error) {
      updateInstalling = false;
      toast(errMessage(error), "error");
    }
  }

  function closeUpdate() {
    if (updateInstalling) return;
    pendingUpdate = null;
    updateInfo = null;
  }

  const reachedGoals = new Set<number>();
  let goalsReady = false;

  $effect(() => {
    if (!$boot || !$status) return;
    for (const goal of $boot.goals) {
      if (goal.targetSeconds <= 0) continue;
      const seconds = goalSeconds(
        goal,
        $status,
        $boot.settings.goalsFocusOnly,
      );
      const done = seconds >= goal.targetSeconds;
      if (done && !reachedGoals.has(goal.id)) {
        reachedGoals.add(goal.id);
        if (goalsReady) {
          const tint =
            goal.projectId != null
              ? $boot.projects.find((project) => project.id === goal.projectId)?.color
              : undefined;
          celebrate(
            goal.period === "daily" ? "Daily goal reached" : "Weekly goal reached",
            `${formatDuration(seconds)} of ${formatDuration(goal.targetSeconds)}${
              goal.projectName ? ` · ${goal.projectName}` : ""
            }`,
            tint,
          );
        }
      } else if (!done) {
        reachedGoals.delete(goal.id);
      }
    }
    goalsReady = true;
  });

  async function syncFromBackend() {
    await loadBootstrap();
    bumpRefresh();
  }

  onMount(() => {
    void loadBootstrap();
    const stopWatchingTheme = watchSystemTheme();
    let alive = true;
    const unlisteners: Array<() => void> = [];

    void (async () => {
      try {
        const [unState, unNavigate] = await Promise.all([
          listen("framewise:state-changed", () => {
            void syncFromBackend();
          }),
          listen<string>("framewise:navigate", (event) => {
            const target = event.payload;
            if (target === "projects:new") {
              requestNewProject();
            } else if (PAGES.some((item) => item.id === target)) {
              navigate(target as Page);
            }
          }),
        ]);
        if (alive) {
          unlisteners.push(unState, unNavigate);
        } else {
          unState();
          unNavigate();
        }
      } catch {
        // Running outside Tauri (plain Vite) has no event bridge.
      }
    })();

    const refresh = () => {
      if (document.visibilityState === "visible") {
        void refreshStatus();
      }
    };

    const poll = window.setInterval(refresh, 1000);
    window.addEventListener("focus", refresh);
    document.addEventListener("visibilitychange", refresh);

    function onKeydown(event: KeyboardEvent) {
      const target = event.target;
      const element = target instanceof HTMLElement ? target : null;
      const typing = !!element?.closest(
        "input, textarea, select, [contenteditable='true']",
      );
      const mod = event.ctrlKey || event.metaKey;

      if (mod && !event.altKey && event.key.toLowerCase() === "k") {
        event.preventDefault();
        commandOpen.update((open) => !open);
        return;
      }

      if (typing) return;

      if (!mod && !event.altKey && event.key === "/") {
        event.preventDefault();
        commandOpen.set(true);
        return;
      }

      if (!mod && !event.altKey && /^[1-5]$/.test(event.key)) {
        event.preventDefault();
        const next = PAGES[Number(event.key) - 1];
        if (next) navigate(next.id);
        return;
      }

      if (mod && !event.altKey && !event.shiftKey && /^[1-5]$/.test(event.key)) {
        event.preventDefault();
        const next = PAGES[Number(event.key) - 1];
        if (next) navigate(next.id);
        return;
      }

      if (event.key === "Escape" && $commandOpen) {
        commandOpen.set(false);
      }
    }

    window.addEventListener("keydown", onKeydown);

    return () => {
      alive = false;
      for (const off of unlisteners) off();
      window.clearInterval(poll);
      window.removeEventListener("focus", refresh);
      document.removeEventListener("visibilitychange", refresh);
      window.removeEventListener("keydown", onKeydown);
      stopWatchingTheme();
    };
  });
</script>

{#if $loading && !$boot}
  <div class="boot">
    <div class="boot-mark"><Logo size={34} /></div>
    <div class="boot-name">Framewise</div>
    <div class="boot-bar"><span></span></div>
  </div>
{:else if $fatalError && !$boot}
  <div class="boot">
    <div class="boot-mark error"><Logo size={34} /></div>
    <div class="boot-name">Framewise could not open</div>
    <div class="boot-sub">{$fatalError}</div>
    <button class="btn" onclick={() => void loadBootstrap()}>Try again</button>
  </div>
{:else if isMini}
  <Mini />
{:else}
  <div class="app">
    <TopBar />

    <div class="scroll">
      <main class="column">
        {#key $page}
          <div in:fade={{ duration: motionDuration(200) }}>
            {#if $page === "dashboard"}
              <Dashboard />
            {:else if $page === "projects"}
              <Projects />
            {:else if $page === "reports"}
              <Reports />
            {:else if $page === "sessions"}
              <Sessions />
            {:else}
              <Settings />
            {/if}
          </div>
        {/key}
      </main>
    </div>

    <Dock />
  </div>
{/if}

{#if !isMini}
  <AppInfo />
  <Celebration />
  <Onboarding />
  <ProjectToolsDialog />
  {#if updateInfo}
    <UpdatePrompt
      version={updateInfo.version}
      notes={updateInfo.notes}
      installing={updateInstalling}
      oninstall={() => void installUpdate()}
      onclose={closeUpdate}
    />
  {/if}
  <Toasts />
{/if}

<style>
  .app {
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background: var(--bg);
  }

  .app::before {
    content: "";
    position: fixed;
    inset: 0;
    z-index: 0;
    pointer-events: none;
    background:
      radial-gradient(
        1100px 520px at 50% -18%,
        color-mix(in srgb, var(--accent) 5%, transparent),
        transparent 70%
      ),
      repeating-linear-gradient(
        0deg,
        transparent 0 3px,
        color-mix(in srgb, var(--text) 2%, transparent) 3px 4px
      );
    opacity: 0.7;
  }

  .scroll {
    position: relative;
    z-index: 1;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .column {
    position: relative;
    z-index: 1;
  }

  .boot {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    height: 100vh;
    padding: 40px;
    background: var(--bg);
    text-align: center;
  }

  .boot-mark {
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 4px;
  }

  .boot-name {
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.03em;
  }

  .boot-sub {
    max-width: 440px;
    color: var(--muted);
    font-size: 13px;
    line-height: 1.6;
    word-break: break-word;
  }

  .boot-bar {
    width: 110px;
    height: 2px;
    margin-top: 8px;
    border-radius: 999px;
    background: var(--surface-3);
    overflow: hidden;
  }

  .boot-bar span {
    display: block;
    width: 40%;
    height: 100%;
    border-radius: inherit;
    background: var(--accent);
    animation: boot-slide 1.1s ease-in-out infinite;
  }

  @keyframes boot-slide {
    0% {
      transform: translateX(-110%);
    }
    100% {
      transform: translateX(360%);
    }
  }
</style>
