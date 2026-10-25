<script lang="ts">
  import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
  import Modal from "./Modal.svelte";
  import AppIcon from "./AppIcon.svelte";
  import Icon from "./Icon.svelte";
  import { api, errMessage } from "../api";
  import { appLabel, formatDuration } from "../format";
  import {
    bumpRefresh,
    closeProjectSetup,
    loadBootstrap,
    projectSetup,
    toast,
  } from "../stores";
  import type { AppTotal } from "../types";

  type Template = {
    id: string;
    name: string;
    hint: string;
    mark: string;
    apps: string[];
  };

  const TEMPLATES: Template[] = [
    {
      id: "montage",
      name: "Montage",
      hint: "Cut, grade, finish",
      mark: "MO",
      apps: [
        "adobe premiere pro.exe",
        "adobe after effects.exe",
        "photoshop.exe",
        "adobe media encoder.exe",
      ],
    },
    {
      id: "youtube",
      name: "YouTube video",
      hint: "Edit, thumbnail, upload",
      mark: "YT",
      apps: [
        "adobe premiere pro.exe",
        "photoshop.exe",
        "adobe media encoder.exe",
        "zen.exe",
      ],
    },
    {
      id: "motion",
      name: "Motion graphics",
      hint: "Animate and composite",
      mark: "MG",
      apps: [
        "adobe after effects.exe",
        "adobe premiere pro.exe",
        "illustrator.exe",
      ],
    },
    {
      id: "photo",
      name: "Photo edit",
      hint: "Retouch and deliver",
      mark: "PH",
      apps: ["photoshop.exe", "lightroom.exe", "lightroom classic.exe"],
    },
  ];

  let known = $state<AppTotal[]>([]);
  let selected = $state<string[]>([]);
  let search = $state("");
  let loading = $state(false);
  let saving = $state(false);

  let setup = $derived($projectSetup);
  let open = $derived(!!setup);
  let project = $derived(setup?.project ?? null);

  $effect(() => {
    const current = $projectSetup;
    if (!current) return;
    const id = current.project.id;
    search = "";
    selected = [];
    loading = true;
    Promise.all([api.knownApps(), api.projectDetail(id)])
      .then(([apps, detail]) => {
        if ($projectSetup?.project.id !== id) return;
        known = apps;
        selected = detail.tools.map((tool) => tool.toLowerCase());
      })
      .catch((cause) => toast(errMessage(cause), "error"))
      .finally(() => {
        if ($projectSetup?.project.id === id) loading = false;
      });
  });

  let appRows = $derived.by(() => {
    const rows = new Map<string, AppTotal>();
    for (const app of known) {
      rows.set(app.app.toLowerCase(), app);
    }
    for (const app of selected) {
      if (!rows.has(app)) rows.set(app, { app, seconds: 0, adobe: false });
    }
    const q = search.trim().toLowerCase();
    return [...rows.values()]
      .filter(
        (app) =>
          !q ||
          appLabel(app.app).toLowerCase().includes(q) ||
          app.app.toLowerCase().includes(q),
      )
      .sort((a, b) => {
        const selectedDelta =
          Number(selected.includes(b.app.toLowerCase())) -
          Number(selected.includes(a.app.toLowerCase()));
        return selectedDelta || b.seconds - a.seconds;
      });
  });

  let activeTemplate = $derived(
    TEMPLATES.find((template) => {
      if (template.apps.length !== selected.length) return false;
      const set = new Set(selected);
      return template.apps.every((app) => set.has(app));
    })?.id ?? null,
  );

  function toggle(app: string) {
    const key = app.toLowerCase();
    selected = selected.includes(key)
      ? selected.filter((item) => item !== key)
      : [...selected, key];
  }

  function applyTemplate(template: Template) {
    selected = [...template.apps];
  }

  function clearTemplate() {
    selected = [];
  }

  async function browseExe() {
    try {
      const picked = await openFileDialog({
        title: "Choose focus tools",
        multiple: true,
        filters: [{ name: "Applications", extensions: ["exe"] }],
      });
      if (!picked) return;
      const paths = Array.isArray(picked) ? picked : [picked];
      const names = paths
        .map((path) => path.split(/[\\/]/).pop() ?? "")
        .map((name) => name.trim().toLowerCase())
        .filter(Boolean);
      selected = [...new Set([...selected, ...names])];
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  async function save() {
    if (!project || saving) return;
    saving = true;
    try {
      await api.setProjectTools(project.id, selected);
      await loadBootstrap();
      bumpRefresh();
      toast(
        selected.length
          ? `${selected.length} focus tool${selected.length === 1 ? "" : "s"} saved.`
          : "No tools selected. You can add them later.",
        "success",
      );
      closeProjectSetup();
    } catch (cause) {
      toast(errMessage(cause), "error");
    } finally {
      saving = false;
    }
  }

  function skip() {
    if (setup?.isNew) {
      toast("You can add focus tools later from the project page.");
    }
    closeProjectSetup();
  }
</script>

<Modal {open} title="Choose focus tools" width={640} onclose={skip}>
  <div class="setup">
    <div class="setup-head">
      <div>
        <span class="eyebrow">Project</span>
        <div class="setup-project">{project?.name ?? ""}</div>
      </div>
      <div class="setup-count">{selected.length} selected</div>
    </div>
    <p class="setup-copy">
      Pick the apps that count as focused work while this project is active. A browser or
      a screenshot tool can be a tool too. Everything else stays tracked as other time.
    </p>

    <div class="templates-head">
      <span>Start from a template</span>
      {#if activeTemplate}
        <button class="clear" onclick={clearTemplate}>Clear</button>
      {/if}
    </div>
    <div class="templates">
      {#each TEMPLATES as template (template.id)}
        <button
          class="template"
          class:active={activeTemplate === template.id}
          onclick={() => applyTemplate(template)}
        >
          <span class="template-mark">{template.mark}</span>
          <span class="template-text">
            <span class="template-name">{template.name}</span>
            <span class="template-hint">{template.hint}</span>
          </span>
          {#if activeTemplate === template.id}
            <span class="template-check"><Icon name="check" size={12} /></span>
          {/if}
        </button>
      {/each}
    </div>

    <div class="apps-head">
      <span>Apps saved in Framewise</span>
      <button class="browse" onclick={() => void browseExe()}>
        <Icon name="plus" size={12} /> Add .exe
      </button>
    </div>

    <label class="search">
      <Icon name="search" size={14} />
      <input bind:value={search} placeholder="Search apps" autocomplete="off" spellcheck="false" />
    </label>

    <div class="app-list">
      {#if loading}
        <div class="app-empty">Loading apps…</div>
      {:else if appRows.length === 0}
        <div class="app-empty">No app matches “{search}”.</div>
      {:else}
        {#each appRows as app (app.app.toLowerCase())}
          {@const key = app.app.toLowerCase()}
          <button
            class="app-row"
            class:selected={selected.includes(key)}
            onclick={() => toggle(app.app)}
            aria-pressed={selected.includes(key)}
          >
            <span class="check" class:on={selected.includes(key)}>
              {#if selected.includes(key)}<Icon name="check" size={11} />{/if}
            </span>
            <AppIcon app={app.app} size={20} />
            <span class="app-name">{appLabel(app.app)}</span>
            <span class="app-meta">
              {#if app.seconds > 0}{formatDuration(app.seconds)}{:else if app.adobe}Adobe{/if}
            </span>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  {#snippet footer()}
    <button class="btn btn-ghost" onclick={skip}>{setup?.isNew ? "Skip for now" : "Cancel"}</button>
    <button
      class="btn btn-primary"
      onclick={() => void save()}
      disabled={saving || loading || (setup?.isNew && selected.length === 0)}
    >
      {saving ? "Saving…" : selected.length ? `Save ${selected.length} tools` : "Save tools"}
    </button>
  {/snippet}
</Modal>

<style>
  .setup-head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
  }

  .setup-project {
    margin-top: 5px;
    font-size: 21px;
    font-weight: 600;
    letter-spacing: -0.025em;
  }

  .setup-count {
    color: var(--muted);
    font-size: 11.5px;
    white-space: nowrap;
  }

  .setup-copy {
    max-width: 540px;
    margin-top: 10px;
    color: var(--muted);
    font-size: 13px;
    line-height: 1.6;
  }

  .templates-head,
  .apps-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 22px;
    color: var(--muted-2);
    font-size: 10.5px;
    font-weight: 500;
    letter-spacing: 0.07em;
    text-transform: uppercase;
  }

  .clear,
  .browse {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--accent-text);
    font-size: 10.5px;
    letter-spacing: 0;
    text-transform: none;
  }

  .clear:hover,
  .browse:hover {
    color: var(--text);
  }

  .templates {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
    margin-top: 9px;
  }

  .template {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    padding: 10px;
    border: 1px solid var(--border);
    border-radius: 13px;
    background: transparent;
    text-align: left;
    transition:
      background-color 140ms ease,
      border-color 140ms ease;
  }

  .template:hover {
    border-color: var(--accent-line);
    background: var(--surface);
  }

  .template.active {
    border-color: var(--accent-line);
    background: var(--accent-soft);
  }

  .template-mark {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    flex: 0 0 auto;
    border-radius: 8px;
    background: var(--surface-3);
    color: var(--muted);
    font-size: 9.5px;
    font-weight: 700;
    letter-spacing: 0.02em;
  }

  .template.active .template-mark {
    background: var(--accent-soft);
    color: var(--accent-text);
  }

  .template-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .template-name {
    color: var(--text);
    font-size: 12.5px;
    font-weight: 600;
  }

  .template-hint {
    overflow: hidden;
    color: var(--muted);
    font-size: 10.5px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .template-check {
    display: flex;
    margin-left: auto;
    color: var(--accent-text);
  }

  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    margin-top: 9px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: 9px;
    color: var(--muted-2);
  }

  .search:focus-within {
    border-color: var(--accent-line);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .search input {
    width: 100%;
    height: 100%;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: 12.5px;
    outline: none;
  }

  .app-list {
    display: flex;
    flex-direction: column;
    max-height: 222px;
    margin-top: 8px;
    overflow-y: auto;
  }

  .app-row {
    display: flex;
    align-items: center;
    gap: 9px;
    min-height: 38px;
    padding: 0 5px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--muted);
    text-align: left;
  }

  .app-row:hover {
    background: var(--surface);
    color: var(--text);
  }

  .app-row.selected {
    color: var(--text);
  }

  .check {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    flex: 0 0 auto;
    border: 1px solid var(--border-strong);
    border-radius: 5px;
    color: var(--primary-fg);
  }

  .check.on {
    border-color: transparent;
    background: var(--primary-bg);
  }

  .app-name {
    overflow: hidden;
    font-size: 12.5px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .app-meta {
    margin-left: auto;
    color: var(--muted-2);
    font-size: 10.5px;
    font-variant-numeric: tabular-nums;
  }

  .app-empty {
    padding: 22px 8px;
    color: var(--muted-2);
    font-size: 12.5px;
    text-align: center;
  }

  @media (max-width: 560px) {
    .templates {
      grid-template-columns: minmax(0, 1fr);
    }

    .setup-head {
      align-items: flex-start;
    }

    .app-list {
      max-height: 190px;
    }
  }
</style>
