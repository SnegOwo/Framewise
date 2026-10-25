<script lang="ts">
  import { save, open as openFileDialog } from "@tauri-apps/plugin-dialog";
  import { api, errMessage } from "../api";
  import {
    boot,
    bumpRefresh,
    loadBootstrap,
    toast,
  } from "../stores";
  import {
    appLabel,
    getDateStyle,
    setDateStyle,
    type DateStyle,
  } from "../format";
  import { applyTheme, type ThemeSetting } from "../theme";
  import Icon from "../components/Icon.svelte";
  import Rule from "../components/Rule.svelte";
  import Segmented from "../components/Segmented.svelte";
  import AppIcon from "../components/AppIcon.svelte";

  let idleMinutes = $derived($boot?.settings.idleMinutes ?? 3);
  let theme = $derived(($boot?.settings.theme ?? "dark") as ThemeSetting);
  let portable = $derived($boot?.settings.portable ?? false);
  let dataPath = $derived($boot?.settings.dataPath ?? "");
  let version = $derived($boot?.settings.version ?? "1.0.0");

  let dateStyle = $state<DateStyle>(getDateStyle());

  let ignoreApps = $derived($boot?.settings.ignoreApps ?? []);
  let adobeApps = $derived($boot?.settings.adobeApps ?? []);

  async function saveSettings(update: Parameters<typeof api.updateSettings>[0]) {
    try {
      await api.updateSettings(update);
      await loadBootstrap();
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  function changeTheme(value: string) {
    const next = value as ThemeSetting;
    applyTheme(next);
    void saveSettings({ theme: next });
  }

  function changeDateStyle(value: string) {
    const next = value as DateStyle;
    dateStyle = next;
    setDateStyle(next);
  }

  function stepIdle(delta: number) {
    const next = Math.max(1, Math.min(120, idleMinutes + delta));
    if (next === idleMinutes) return;
    void saveSettings({ idleMinutes: next });
  }

  async function replaceApps(kind: "ignore" | "adobe", apps: string[]) {
    const unique = [...new Set(apps.map((app) => app.trim()).filter(Boolean))];
    try {
      await api.replaceAppList(kind, unique);
      await loadBootstrap();
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  async function pickApps(kind: "ignore" | "adobe") {
    try {
      const selected = await openFileDialog({
        title: kind === "ignore" ? "Choose apps to ignore" : "Choose Adobe apps",
        multiple: true,
        filters: [{ name: "Applications", extensions: ["exe"] }],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      const names = paths
        .map((path) => path.split(/[\\/]/).pop() ?? "")
        .filter((name) => /\.exe$/i.test(name));
      if (names.length === 0) return;
      const list = kind === "ignore" ? ignoreApps : adobeApps;
      await replaceApps(kind, [...list, ...names]);
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  function removeApp(kind: "ignore" | "adobe", app: string) {
    const list = kind === "ignore" ? ignoreApps : adobeApps;
    void replaceApps(
      kind,
      list.filter((item) => item !== app),
    );
  }

  async function backup() {
    try {
      const path = await save({
        title: "Back up Framewise",
        defaultPath: `Framewise backup ${new Date().toISOString().slice(0, 10)}.sqlite3`,
        filters: [{ name: "SQLite database", extensions: ["sqlite3", "db"] }],
      });
      if (!path) return;
      await api.backupDatabase(path);
      toast("Backup saved.", "success");
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  async function restore() {
    try {
      const path = await openFileDialog({
        title: "Import a Framewise database",
        multiple: false,
        filters: [{ name: "SQLite database", extensions: ["sqlite3", "db"] }],
      });
      if (!path || Array.isArray(path)) return;
      if (!window.confirm("Replace the current database with this file?")) return;
      await api.importDatabase(path);
      await loadBootstrap();
      bumpRefresh();
      toast("Database imported.", "success");
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  async function openFolder() {
    try {
      await api.openDataFolder();
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }
</script>

<div class="page settings">
  <header class="page-head reveal">
    <div class="eyebrow">Settings</div>
  </header>

  <section class="group">
    <Rule />
    <div class="eyebrow group-label">Tracking</div>

    <div class="setting-row">
      <div class="setting-text">
        <div class="setting-name">Idle timeout</div>
        <div class="setting-help">Time without input before the clock pauses.</div>
      </div>
      <div class="stepper" role="group" aria-label="Idle timeout in minutes">
        <button onclick={() => stepIdle(-1)} aria-label="Decrease idle timeout" disabled={idleMinutes <= 1}>
          <Icon name="minus" size={13} />
        </button>
        <span class="value">{idleMinutes} min</span>
        <button onclick={() => stepIdle(1)} aria-label="Increase idle timeout" disabled={idleMinutes >= 120}>
          <Icon name="plus" size={13} />
        </button>
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-text">
        <div class="setting-name">Count background renders</div>
        <div class="setting-help">Media Encoder time while no Adobe app is focused.</div>
      </div>
      <button
        class="toggle"
        class:on={$boot?.settings.countMediaEncoderBackground}
        onclick={() =>
          void saveSettings({
            countMediaEncoderBackground: !$boot?.settings.countMediaEncoderBackground,
          })}
        aria-pressed={$boot?.settings.countMediaEncoderBackground}
        aria-label="Count background renders"
      >
        <span class="knob"></span>
      </button>
    </div>

    <div class="setting-row">
      <div class="setting-text">
        <div class="setting-name">Goal notifications</div>
        <div class="setting-help">One local notification when a goal is reached.</div>
      </div>
      <button
        class="toggle"
        class:on={$boot?.settings.notifyGoals}
        onclick={() => void saveSettings({ notifyGoals: !$boot?.settings.notifyGoals })}
        aria-pressed={$boot?.settings.notifyGoals}
        aria-label="Goal notifications"
      >
        <span class="knob"></span>
      </button>
    </div>

    <div class="setting-row">
      <div class="setting-text">
        <div class="setting-name">Celebration sound</div>
        <div class="setting-help">One soft chime when a goal is reached.</div>
      </div>
      <button
        class="toggle"
        class:on={$boot?.settings.celebrateSound}
        onclick={() => void saveSettings({ celebrateSound: !$boot?.settings.celebrateSound })}
        aria-pressed={$boot?.settings.celebrateSound}
        aria-label="Celebration sound"
      >
        <span class="knob"></span>
      </button>
    </div>

    <div class="setting-row">
      <div class="setting-text">
        <div class="setting-name">Launch at login</div>
        <div class="setting-help">
          {#if portable}
            Unavailable in portable mode.
          {:else}
            Start Framewise when you sign in.
          {/if}
        </div>
      </div>
      <button
        class="toggle"
        class:on={$boot?.settings.launchAtLogin}
        disabled={portable}
        onclick={() => void saveSettings({ launchAtLogin: !$boot?.settings.launchAtLogin })}
        aria-pressed={$boot?.settings.launchAtLogin}
        aria-label="Launch at login"
      >
        <span class="knob"></span>
      </button>
    </div>
  </section>

  <section class="group">
    <Rule />
    <div class="eyebrow group-label">Goals</div>

    <div class="setting-row">
      <div class="setting-text">
        <div class="setting-name">Count only focus time</div>
        <div class="setting-help">
          Goals count selected focus tools only. Apps without tools still count in full.
        </div>
      </div>
      <button
        class="toggle"
        class:on={$boot?.settings.goalsFocusOnly}
        onclick={() => void saveSettings({ goalsFocusOnly: !$boot?.settings.goalsFocusOnly })}
        aria-pressed={$boot?.settings.goalsFocusOnly}
        aria-label="Count only focus time toward goals"
      >
        <span class="knob"></span>
      </button>
    </div>

    <div class="eyebrow group-label">Appearance</div>

    <div class="setting-row">
      <div class="setting-text">
        <div class="setting-name">Theme</div>
      </div>
      <Segmented
        options={[
          { value: "dark", label: "Dark" },
          { value: "light", label: "Light" },
          { value: "system", label: "System" },
        ]}
        value={theme}
        onchange={changeTheme}
      />
    </div>

    <div class="setting-row">
      <div class="setting-text">
        <div class="setting-name">Date format</div>
        <div class="setting-help">English dates, or match this computer.</div>
      </div>
      <Segmented
        options={[
          { value: "english", label: "English" },
          { value: "system", label: "System" },
        ]}
        value={dateStyle}
        onchange={changeDateStyle}
      />
    </div>
  </section>

  <section class="group">
    <Rule />
    <div class="eyebrow group-label">Adobe apps</div>
    <div class="setting-note">Adobe executables that always credit the active project.</div>
    <div class="chips">
      {#each adobeApps as app (app)}
        <span class="app-chip">
          <AppIcon app={app} size={18} />
          <span class="chip-name">{appLabel(app)}</span>
          <button class="chip-x" onclick={() => removeApp("adobe", app)} aria-label={`Remove ${appLabel(app)}`}>
            <Icon name="x" size={11} />
          </button>
        </span>
      {/each}
      <button class="chip-add-button" onclick={() => void pickApps("adobe")}>
        <Icon name="plus" size={13} /> Add .exe
      </button>
    </div>
  </section>

  <section class="group">
    <Rule />
    <div class="eyebrow group-label">Ignored apps</div>
    <div class="setting-note">Never tracked, even if they look like Adobe apps.</div>
    <div class="chips">
      {#each ignoreApps as app (app)}
        <span class="app-chip">
          <AppIcon app={app} size={18} />
          <span class="chip-name">{appLabel(app)}</span>
          <button class="chip-x" onclick={() => removeApp("ignore", app)} aria-label={`Remove ${appLabel(app)}`}>
            <Icon name="x" size={11} />
          </button>
        </span>
      {/each}
      <button class="chip-add-button" onclick={() => void pickApps("ignore")}>
        <Icon name="plus" size={13} /> Add .exe
      </button>
    </div>
  </section>

  <section class="group">
    <Rule />
    <div class="eyebrow group-label">Data</div>

    <div class="setting-row">
      <div class="setting-text">
        <div class="setting-name">Location</div>
        <div class="setting-help path">{dataPath}</div>
      </div>
      <button class="btn btn-sm" onclick={openFolder}>
        <Icon name="folder" size={14} /> Open folder
      </button>
    </div>

    <div class="setting-row">
      <div class="setting-text">
        <div class="setting-name">Backup</div>
        <div class="setting-help">Save a copy of the SQLite database.</div>
      </div>
      <div class="button-pair">
        <button class="btn btn-sm" onclick={backup}>
          <Icon name="download" size={14} /> Backup
        </button>
        <button class="btn btn-sm" onclick={restore}>
          <Icon name="upload" size={14} /> Import
        </button>
      </div>
    </div>
  </section>

  <section class="group">
    <Rule />
    <div class="eyebrow group-label">About</div>

    <div class="about">
      <div class="about-line"><span>Version</span><span>{version}</span></div>
      <div class="about-line">
        <span>Mode</span><span>{portable ? "Portable" : "Installed"}</span>
      </div>
      <div class="about-line"><span>Made by</span><span>Sneg HQ</span></div>
      <div class="about-line"><span>License</span><span>MIT</span></div>
    </div>

    <p class="privacy">
      Framewise works entirely offline. There are no accounts, no telemetry and no remote
      requests. Everything stays in the database shown above.
    </p>
  </section>
</div>

<style>
  .page-head {
    padding-top: 64px;
  }

  .group {
    margin-top: 44px;
  }

  .group-label {
    margin: 18px 0 6px;
  }

  .setting-note {
    margin: 0 0 14px;
    color: var(--muted);
    font-size: 13px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    min-height: 62px;
    padding: 10px 0;
    border-bottom: 1px solid var(--border-subtle);
  }

  .setting-row:last-child {
    border-bottom: 0;
  }

  .setting-text {
    min-width: 0;
  }

  .setting-name {
    font-size: 15px;
  }

  .setting-help {
    margin-top: 2px;
    color: var(--muted);
    font-size: 12.5px;
  }

  .setting-help.path {
    font-variant-numeric: tabular-nums;
    word-break: break-all;
  }

  .button-pair {
    display: flex;
    gap: 8px;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }

  .app-chip {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 32px;
    padding: 0 6px 0 8px;
    border: 1px dashed var(--border-strong);
    border-radius: var(--r-chip);
    font-size: 12.5px;
  }

  .chip-name {
    white-space: nowrap;
  }

  .chip-x {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--muted-2);
    opacity: 0;
    transition:
      opacity 140ms ease-out,
      background-color 140ms ease-out;
  }

  .app-chip:hover .chip-x,
  .chip-x:focus-visible {
    opacity: 1;
  }

  .chip-x:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .chip-add-button {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 32px;
    padding: 0 11px;
    border: 1px dashed var(--border);
    border-radius: var(--r-chip);
    background: transparent;
    color: var(--muted);
    font-size: 12.5px;
  }

  .chip-add-button:hover {
    color: var(--text);
    border-color: var(--border-strong);
  }

  .about {
    display: flex;
    flex-direction: column;
    gap: 0;
    margin-top: 4px;
  }

  .about-line {
    display: flex;
    justify-content: space-between;
    gap: 20px;
    padding: 9px 0;
    border-bottom: 1px solid var(--border-subtle);
    font-size: 14px;
  }

  .about-line span:first-child {
    color: var(--muted);
  }

  .privacy {
    max-width: 560px;
    margin-top: 18px;
    color: var(--muted);
    font-size: 13.5px;
    line-height: 1.6;
  }

  @media (max-width: 640px) {
    .setting-row {
      flex-direction: column;
      align-items: flex-start;
      gap: 10px;
    }
  }
</style>
