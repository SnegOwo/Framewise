<script lang="ts">
  import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
  import Modal from "./Modal.svelte";
  import Logo from "./Logo.svelte";
  import Segmented from "./Segmented.svelte";
  import { api, errMessage } from "../api";
  import { applyTheme, type ThemeSetting } from "../theme";
  import { boot, loadBootstrap, toast } from "../stores";
  import type { DetectedApp } from "../types";

  let theme = $state<ThemeSetting>("dark");
  let focusGoals = $state(true);
  let notifyGoals = $state(true);
  let celebrateSound = $state(true);
  let launchAtLogin = $state(false);
  let adobeApps = $state<string[]>([]);
  let adobeInitial = $state<string[]>([]);
  let scanPaths = $state<DetectedApp[]>([]);
  let scanNote = $state("");
  let scanning = $state(false);
  let saving = $state(false);
  let initialized = $state(false);

  let open = $derived(!!$boot && !$boot.settings.onboardingComplete);
  let portable = $derived($boot?.settings.portable ?? false);

  $effect(() => {
    const settings = $boot?.settings;
    if (!settings || initialized) return;
    theme = settings.theme as ThemeSetting;
    focusGoals = settings.goalsFocusOnly;
    notifyGoals = settings.notifyGoals;
    celebrateSound = settings.celebrateSound;
    launchAtLogin = settings.launchAtLogin;
    adobeApps = [...settings.adobeApps];
    adobeInitial = [...settings.adobeApps];
    initialized = true;
  });

  function addAdobeNames(names: string[]) {
    const byName = new Map(adobeApps.map((app) => [app.toLowerCase(), app]));
    for (const name of names) {
      const clean = name.trim();
      if (clean) byName.set(clean.toLowerCase(), clean);
    }
    adobeApps = [...byName.values()];
  }

  async function scanAdobe() {
    if (scanning) return;
    scanning = true;
    scanNote = "Scanning Program Files, ProgramData, the registry and fixed drives…";
    try {
      const found = await api.scanAdobeApps();
      scanPaths = found;
      const names = [...new Set(found.map((item) => item.app))];
      addAdobeNames(names);
      scanNote = found.length
        ? `Found ${names.length} app${names.length === 1 ? "" : "s"} in ${found.length} location${found.length === 1 ? "" : "s"}.`
        : "No Adobe apps found. Choose the .exe manually.";
    } catch (cause) {
      scanNote = "The scan could not finish. Choose the .exe manually.";
      toast(errMessage(cause), "error");
    } finally {
      scanning = false;
    }
  }

  async function pickAdobe() {
    try {
      const selected = await openFileDialog({
        title: "Choose Adobe apps",
        multiple: true,
        filters: [{ name: "Applications", extensions: ["exe"] }],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      const names = paths
        .map((path) => path.split(/[\\/]/).pop() ?? "")
        .filter((name) => /\.exe$/i.test(name));
      addAdobeNames(names);
      scanNote = names.length
        ? `Added ${names.length} app${names.length === 1 ? "" : "s"}.`
        : "";
    } catch (cause) {
      toast(errMessage(cause), "error");
    }
  }

  function removeAdobe(name: string) {
    adobeApps = adobeApps.filter((app) => app !== name);
  }

  async function finish(skip: boolean) {
    if (saving) return;
    saving = true;
    try {
      if (!skip) {
        const before = [...adobeInitial].sort().join("|").toLowerCase();
        const after = [...adobeApps].sort().join("|").toLowerCase();
        if (before !== after) {
          await api.replaceAppList("adobe", adobeApps);
        }
      }
      await api.updateSettings(
        skip
          ? { onboardingComplete: true }
          : {
              onboardingComplete: true,
              theme,
              goalsFocusOnly: focusGoals,
              notifyGoals,
              celebrateSound,
              launchAtLogin: portable ? false : launchAtLogin,
            },
      );
      if (!skip) applyTheme(theme, true);
      await loadBootstrap();
    } catch (cause) {
      toast(errMessage(cause), "error");
    } finally {
      saving = false;
    }
  }
</script>

<Modal {open} title="Welcome to Framewise" width={560} onclose={() => void finish(true)}>
  <div class="welcome">
    <div class="welcome-head">
      <span class="welcome-mark"><Logo size={34} /></span>
      <div>
        <div class="welcome-title">A quiet timer for the work that matters.</div>
        <p>
          Choose how Framewise should behave. These are the defaults — you can change
          everything later in Settings.
        </p>
      </div>
    </div>

    <div class="option">
      <div class="option-text">
        <div class="option-name">Theme</div>
        <div class="option-help">Dark is the default.</div>
      </div>
      <Segmented
        options={[
          { value: "dark", label: "Dark" },
          { value: "light", label: "Light" },
          { value: "system", label: "System" },
        ]}
        value={theme}
        onchange={(value) => (theme = value as ThemeSetting)}
      />
    </div>

    <div class="adobe-block">
      <div class="adobe-head">
        <div>
          <div class="option-name">Adobe apps</div>
          <div class="option-help">Framewise uses these to credit Adobe time to the active project.</div>
        </div>
        <div class="adobe-actions">
          <button class="btn btn-sm" onclick={() => void scanAdobe()} disabled={scanning || saving}>
            {scanning ? "Scanning…" : "Scan this PC"}
          </button>
          <button class="btn btn-sm btn-ghost" onclick={() => void pickAdobe()} disabled={scanning || saving}>
            Choose .exe
          </button>
        </div>
      </div>
      {#if scanNote}
        <div class="scan-note">{scanNote}</div>
      {/if}
      <div class="adobe-chips">
        {#each adobeApps as app (app)}
          <span class="adobe-chip">
            <span>{app}</span>
            <button onclick={() => removeAdobe(app)} aria-label={`Remove ${app}`}>×</button>
          </span>
        {/each}
        {#if adobeApps.length === 0}
          <span class="adobe-empty">No Adobe apps selected. You can add them later in Settings.</span>
        {/if}
      </div>
      {#if scanPaths.length > 0}
        <details class="scan-paths">
          <summary>Found paths</summary>
          <div class="scan-path-list">
            {#each scanPaths as item (item.path)}
              <div><span>{item.app}</span><code>{item.path}</code></div>
            {/each}
          </div>
        </details>
      {/if}
    </div>

    <div class="option">
      <div class="option-text">
        <div class="option-name">Goals count only focus time</div>
        <div class="option-help">Selected tools count; other app time still stays in reports.</div>
      </div>
      <button
        class="toggle"
        class:on={focusGoals}
        onclick={() => (focusGoals = !focusGoals)}
        aria-pressed={focusGoals}
        aria-label="Count only focus time toward goals"
      >
        <span class="knob"></span>
      </button>
    </div>

    <div class="option">
      <div class="option-text">
        <div class="option-name">Goal notifications</div>
        <div class="option-help">One local notification when a target is reached.</div>
      </div>
      <button
        class="toggle"
        class:on={notifyGoals}
        onclick={() => (notifyGoals = !notifyGoals)}
        aria-pressed={notifyGoals}
        aria-label="Goal notifications"
      >
        <span class="knob"></span>
      </button>
    </div>

    <div class="option">
      <div class="option-text">
        <div class="option-name">Celebration sound</div>
        <div class="option-help">A soft chime with the goal reward.</div>
      </div>
      <button
        class="toggle"
        class:on={celebrateSound}
        onclick={() => (celebrateSound = !celebrateSound)}
        aria-pressed={celebrateSound}
        aria-label="Celebration sound"
      >
        <span class="knob"></span>
      </button>
    </div>

    <div class="option" class:disabled={portable}>
      <div class="option-text">
        <div class="option-name">Launch at login</div>
        <div class="option-help">
          {portable ? "Unavailable in portable mode." : "Start Framewise when you sign in."}
        </div>
      </div>
      <button
        class="toggle"
        class:on={launchAtLogin && !portable}
        onclick={() => {
          if (!portable) launchAtLogin = !launchAtLogin;
        }}
        aria-pressed={launchAtLogin && !portable}
        aria-label="Launch at login"
        disabled={portable}
      >
        <span class="knob"></span>
      </button>
    </div>

    <div class="note">Framewise stays in the tray and never sends your data anywhere.</div>
  </div>

  {#snippet footer()}
    <button class="btn btn-ghost" onclick={() => void finish(true)} disabled={saving}>
      Skip — use defaults
    </button>
    <button class="btn btn-primary" onclick={() => void finish(false)} disabled={saving}>
      {saving ? "Saving…" : "Get started"}
    </button>
  {/snippet}
</Modal>

<style>
  .welcome-head {
    display: flex;
    gap: 14px;
    align-items: flex-start;
    padding-bottom: 6px;
  }

  .welcome-mark {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 46px;
    flex: 0 0 auto;
    border-radius: 13px;
    background: var(--surface);
  }

  .welcome-title {
    font-size: 17px;
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  .welcome-head p {
    margin-top: 4px;
    color: var(--muted);
    font-size: 12.5px;
    line-height: 1.55;
  }

  .option {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 18px;
    min-height: 58px;
    padding: 9px 0;
    border-top: 1px solid var(--border-subtle);
  }

  .adobe-block {
    margin-top: 14px;
    padding: 14px 0 12px;
    border-top: 1px solid var(--border-subtle);
    border-bottom: 1px solid var(--border-subtle);
  }

  .adobe-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .adobe-actions {
    display: flex;
    flex: 0 0 auto;
    gap: 6px;
  }

  .scan-note {
    margin-top: 7px;
    color: var(--accent-text);
    font-size: 11px;
  }

  .adobe-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    max-height: 84px;
    margin-top: 9px;
    overflow-y: auto;
  }

  .adobe-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 25px;
    padding: 0 4px 0 8px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface);
    color: var(--muted);
    font-size: 10.5px;
  }

  .adobe-chip button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 17px;
    height: 17px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: transparent;
    color: var(--muted-2);
    font-size: 13px;
    line-height: 1;
  }

  .adobe-chip button:hover {
    background: var(--surface-3);
    color: var(--text);
  }

  .adobe-empty {
    color: var(--muted-2);
    font-size: 11px;
  }

  .scan-paths {
    margin-top: 8px;
    color: var(--muted);
    font-size: 10.5px;
  }

  .scan-paths summary {
    cursor: pointer;
    color: var(--muted-2);
  }

  .scan-path-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 90px;
    margin-top: 6px;
    overflow-y: auto;
  }

  .scan-path-list div {
    display: flex;
    gap: 8px;
    min-width: 0;
  }

  .scan-path-list span {
    flex: 0 0 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .scan-path-list code {
    overflow: hidden;
    color: var(--muted-2);
    font-family: "JetBrains Mono Variable", monospace;
    font-size: 9.5px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .option:first-of-type {
    margin-top: 10px;
  }

  .option.disabled {
    opacity: 0.55;
  }

  .option-text {
    min-width: 0;
  }

  .option-name {
    font-size: 13.5px;
    font-weight: 500;
  }

  .option-help {
    margin-top: 2px;
    color: var(--muted);
    font-size: 11.5px;
    line-height: 1.45;
  }

  .note {
    margin-top: 16px;
    color: var(--muted-2);
    font-size: 11px;
  }
</style>
