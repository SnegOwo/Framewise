<script lang="ts">
  import { tick } from "svelte";
  import { fly } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import { reducedMotion } from "../motion";
  import SessionModal from "./SessionModal.svelte";
  import {
    activeProject,
    activeProjects,
    boot,
    bumpRefresh,
    loadBootstrap,
    status,
    switchProject,
    togglePaused,
    toast,
  } from "../stores";
  import { PAGES, commandOpen, navigate, requestNewProject } from "../router";
  import { api, errMessage } from "../api";
  import { applyTheme, type ThemeSetting } from "../theme";
  import { exportRange } from "../export";
  import { todayStr, weekBounds } from "../format";

  type Action = {
    id: string;
    label: string;
    group: "Projects" | "Create" | "Actions" | "Go to" | "Export";
    icon: string;
    hint?: string;
    keywords?: string;
    color?: string;
    run: () => unknown;
  };

  let open = $state(false);
  let query = $state("");
  let selectedId = $state<string | null>(null);
  let inputEl: HTMLInputElement | undefined = $state();
  let dockEl: HTMLDivElement | undefined = $state();
  let resultsEl: HTMLDivElement | undefined = $state();
  let sessionOpen = $state(false);
  let thumb = $state({ top: 0, height: 0, ready: false });

  let range = $derived(weekBounds(todayStr()));
  let isLight = $derived(($boot?.settings.theme ?? "dark") === "light");
  let paused = $derived(!!$status?.paused);

  async function toggleTheme() {
    const next: ThemeSetting = isLight ? "dark" : "light";
    applyTheme(next);
    try {
      await api.updateSettings({ theme: next });
      await loadBootstrap();
    } catch (error) {
      toast(errMessage(error), "error");
    }
  }

  let projectActions = $derived<Action[]>(
    $activeProjects.map((project) => ({
      id: `project-${project.id}`,
      label: project.name,
      group: "Projects",
      icon: "projects",
      color: project.color,
      hint:
        $status?.activeProjectId === project.id
          ? "Active"
          : project.client ?? undefined,
      keywords: `project switch ${project.client ?? ""}`,
      run: () => switchProject(project.id),
    })),
  );

  let actions = $derived<Action[]>([
    {
      id: "pause",
      label: paused ? "Resume tracking" : "Pause tracking",
      group: "Actions",
      icon: paused ? "play" : "pause",
      keywords: "pause resume stop start timer",
      run: () => togglePaused(),
    },
    {
      id: "add-session",
      label: "Add session",
      group: "Actions",
      icon: "plus",
      keywords: "manual time entry add edit",
      run: () => {
        sessionOpen = true;
      },
    },
    {
      id: "mini",
      label: "Mini timer",
      group: "Actions",
      icon: "monitor",
      keywords: "mini tray compact timer",
      run: () => api.openMiniWindow(),
    },
    {
      id: "new-project",
      label: "New project",
      group: "Actions",
      icon: "plus",
      keywords: "create project add",
      run: () => requestNewProject(),
    },
    {
      id: "theme",
      label: isLight ? "Switch to dark" : "Switch to light",
      group: "Actions",
      icon: isLight ? "moon" : "sun",
      keywords: "theme appearance color",
      run: () => toggleTheme(),
    },
    ...PAGES.map(
      (item): Action => ({
        id: `page-${item.id}`,
        label: item.label,
        group: "Go to",
        icon:
          item.id === "dashboard"
            ? "dashboard"
            : item.id === "projects"
              ? "projects"
              : item.id === "reports"
                ? "reports"
                : item.id === "sessions"
                  ? "sessions"
                  : "settings",
        hint: String(item.key),
        keywords: `go page ${item.id}`,
        run: () => navigate(item.id),
      }),
    ),
    ...(["csv", "json", "md"] as const).map(
      (format): Action => ({
        id: `export-${format}`,
        label: `Export this week as ${format.toUpperCase()}`,
        group: "Export",
        icon: "download",
        keywords: `export download report ${format}`,
        run: () => exportRange(format, range.start, range.end),
      }),
    ),
  ]);

  let all = $derived<Action[]>([...projectActions, ...actions]);

  let createAction = $derived.by<Action | null>(() => {
    const name = query.trim();
    if (!name) return null;
    const q = name.toLowerCase();
    if ($activeProjects.some((project) => project.name.toLowerCase() === q)) return null;
    if (
      $activeProjects.some(
        (project) =>
          project.name.toLowerCase().includes(q) ||
          (project.client ?? "").toLowerCase().includes(q),
      )
    ) {
      return null;
    }
    return {
      id: "create-project",
      label: `Create project “${name}”`,
      group: "Create",
      icon: "plus",
      hint: "Enter",
      keywords: name,
      run: async () => {
        try {
          const project = await api.createProject(name, null, "#7dd3fc");
          await loadBootstrap();
          await switchProject(project.id);
          toast("Project created.", "success");
        } catch (error) {
          toast(errMessage(error), "error");
        }
      },
    };
  });

  let filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const matches = q
      ? all.filter((action) => fuzzy(`${action.label} ${action.keywords ?? ""}`, q))
      : all;
    return createAction ? [...matches, createAction] : matches;
  });

  let selected = $derived.by(() => {
    if (selectedId == null) return 0;
    const index = filtered.findIndex((action) => action.id === selectedId);
    return index >= 0 ? index : 0;
  });

  function fuzzy(text: string, query: string): boolean {
    const haystack = text.toLowerCase();
    if (haystack.includes(query)) return true;
    let index = 0;
    for (const char of haystack) {
      if (char === query[index]) index += 1;
      if (index >= query.length) return true;
    }
    return false;
  }

  $effect(() => {
    query;
    selectedId = null;
  });

  $effect(() => {
    open;
    query;
    filtered.length;
    selected;
    tick().then(measureThumb);
  });

  function measureThumb() {
    if (!resultsEl) return;
    if (filtered.length === 0) {
      thumb = { top: 0, height: 0, ready: false };
      return;
    }
    const buttons = resultsEl.querySelectorAll<HTMLButtonElement>(".result");
    const button = buttons[selected];
    if (!button) return;
    thumb = {
      top: button.offsetTop,
      height: button.offsetHeight,
      ready: true,
    };
    button.scrollIntoView({
      block: "nearest",
      behavior: reducedMotion() ? "auto" : "smooth",
    });
  }

  $effect(() => {
    if ($commandOpen) {
      open = true;
      tick().then(() => inputEl?.focus());
    } else {
      open = false;
      query = "";
    }
  });

  function openDock() {
    commandOpen.set(true);
  }

  function close() {
    open = false;
    query = "";
    selectedId = null;
    commandOpen.set(false);
    inputEl?.blur();
  }

  function run(action: Action | undefined) {
    if (!action) return;
    close();
    void action.run();
  }

  function onKey(event: KeyboardEvent) {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      if (!filtered.length) return;
      const current = filtered.findIndex((action) => action.id === selectedId);
      let next: number;
      if (current < 0) {
        next = event.key === "ArrowDown" ? 0 : filtered.length - 1;
      } else {
        const delta = event.key === "ArrowDown" ? 1 : -1;
        next = (current + delta + filtered.length) % filtered.length;
      }
      selectedId = filtered[next].id;
    } else if (event.key === "Enter") {
      event.preventDefault();
      if (filtered.length) run(filtered[selected]);
    } else if (event.key === "Escape") {
      event.preventDefault();
      close();
    }
  }

  function changed(items: Action[], index: number): boolean {
    if (index === 0) return true;
    return items[index - 1]?.group !== items[index]?.group;
  }

  function onWindowClick(event: MouseEvent) {
    if (open && dockEl && !dockEl.contains(event.target as Node)) close();
  }
</script>

<svelte:window onclick={onWindowClick} />

<div class="dock" class:expanded={open} bind:this={dockEl}>
  <div class="dock-row">
    <button
      class="dock-btn project"
      onclick={() => {
        query = "";
        openDock();
      }}
      aria-haspopup="listbox"
      aria-expanded={open}
      title="Switch project"
    >
      {#if $activeProject}
        <span class="dot" style:background={$activeProject.color}></span>
        <span class="project-name">{$activeProject.name}</span>
      {:else}
        <span class="dot empty"></span>
        <span class="project-name muted">No project</span>
      {/if}
      <Icon name="chevron-down" size={12} />
    </button>

    <span class="dock-divider"></span>

    <button
      class="dock-btn solo"
      onclick={() => void togglePaused()}
      title={paused ? "Resume tracking" : "Pause tracking"}
      aria-label={paused ? "Resume tracking" : "Pause tracking"}
    >
      <Icon name={paused ? "play" : "pause"} size={15} />
    </button>

    <span class="dock-divider"></span>

    <div class="dock-field">
      <Icon name="search" size={15} />
      <input
        bind:this={inputEl}
        bind:value={query}
        onfocus={openDock}
        onkeydown={onKey}
        placeholder="Switch project or type a command…"
        aria-label="Switch project or type a command"
        autocomplete="off"
        spellcheck="false"
      />
      {#if query}
        <button class="dock-clear" onclick={() => (query = "")} aria-label="Clear search">
          <Icon name="x" size={12} />
        </button>
      {:else}
        <span class="kbd">{open ? "Esc" : "/"}</span>
      {/if}
    </div>
  </div>

  {#if open}
    <div
      class="results"
      role="listbox"
      aria-label="Commands"
      bind:this={resultsEl}
      transition:fly={{ y: 8, duration: 220, opacity: 0.25 }}
    >
      {#if thumb.ready}
        <span
          class="result-thumb"
          style:top="{thumb.top}px"
          style:height="{thumb.height}px"
          aria-hidden="true"
        ></span>
      {/if}
      {#if filtered.length === 0}
        <div class="no-results">Nothing matches “{query}”</div>
      {:else}
        {#each filtered as action, index (action.id)}
          {#if changed(filtered, index)}
            <div class="group">{action.group}</div>
          {/if}
          <button
            class="result"
            class:selected={index === selected}
            onmouseenter={() => (selectedId = action.id)}
            onclick={() => run(action)}
            role="option"
            aria-selected={index === selected}
            style:animation-delay="{Math.min(index * 16, 160)}ms"
          >
            {#if action.color}
              <span class="dot" style:background={action.color}></span>
            {:else}
              <Icon name={action.icon} size={15} />
            {/if}
            <span class="result-label">{action.label}</span>
            {#if action.hint}<span class="result-hint">{action.hint}</span>{/if}
            {#if index === selected}<span class="kbd">↵</span>{/if}
          </button>
        {/each}
      {/if}
    </div>
    <div class="foot">
      <span><span class="kbd">↑</span><span class="kbd">↓</span> move</span>
      <span><span class="kbd">↵</span> run</span>
      <span><span class="kbd">Esc</span> close</span>
    </div>
  {/if}
</div>

<SessionModal
  open={sessionOpen}
  session={null}
  defaultDay={todayStr()}
  onclose={() => {
    sessionOpen = false;
    bumpRefresh();
  }}
/>

<style>
  .dock.expanded {
    flex-direction: column;
    align-items: stretch;
    gap: 0;
    padding: 6px;
    animation: dock-pop 340ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  @keyframes dock-pop {
    0% {
      transform: translateX(-50%) scale(0.982);
    }
    100% {
      transform: translateX(-50%) scale(1);
    }
  }

  .dock-row {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: 1 1 auto;
    width: 100%;
    min-height: 36px;
  }

  .dock-field {
    transition: background-color 180ms ease;
  }

  .dock-field:focus-within {
    background: var(--surface-3);
  }

  .dock-clear {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--muted-2);
    transition:
      background-color 140ms ease,
      color 140ms ease;
  }

  .dock-clear:hover {
    background: var(--surface-4);
    color: var(--text);
  }

  .project {
    max-width: 210px;
    padding: 0 10px;
  }

  .project-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .solo {
    width: 36px;
    padding: 0;
    color: var(--muted);
  }

  .solo:hover {
    color: var(--text);
  }

  .results {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    margin-top: 6px;
    padding: 4px;
    border-top: 1px solid var(--border-subtle);
  }

  .result-thumb {
    position: absolute;
    right: 4px;
    left: 4px;
    z-index: 0;
    border-radius: 9px;
    background: var(--surface-3);
    pointer-events: none;
    transition:
      top 240ms cubic-bezier(0.32, 0.72, 0, 1),
      height 240ms cubic-bezier(0.32, 0.72, 0, 1);
  }

  .group {
    position: relative;
    z-index: 1;
    padding: 8px 9px 4px;
    color: var(--muted-2);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .result {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    min-height: 36px;
    padding: 6px 9px;
    border: 0;
    border-radius: 9px;
    background: transparent;
    color: var(--text);
    font-size: 14px;
    text-align: left;
    animation: result-in 220ms ease-out both;
  }

  @keyframes result-in {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .result.selected {
    color: var(--text);
  }

  .result :global(svg) {
    color: var(--muted);
  }

  .result.selected :global(svg) {
    color: var(--text);
  }

  .result-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .result-hint {
    margin-left: auto;
    color: var(--muted-2);
    font-size: 12px;
    white-space: nowrap;
  }

  .result .kbd {
    margin-left: auto;
  }

  .result-hint + .kbd {
    margin-left: 8px;
  }

  .no-results {
    padding: 26px 10px;
    color: var(--muted-2);
    font-size: 13.5px;
    text-align: center;
  }

  .foot {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 8px 10px 2px;
    border-top: 1px solid var(--border-subtle);
    color: var(--muted-2);
    font-size: 11.5px;
  }

  .foot .kbd {
    margin-right: 2px;
    color: var(--muted);
  }

  @media (max-width: 700px) {
    .project {
      max-width: 130px;
    }
    .project-name {
      max-width: 82px;
    }
  }

  @media (max-width: 520px) {
    .project-name {
      display: none;
    }
  }
</style>
