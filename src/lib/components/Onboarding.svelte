<script lang="ts">
  import Modal from "./Modal.svelte";
  import Logo from "./Logo.svelte";
  import Segmented from "./Segmented.svelte";
  import { api, errMessage } from "../api";
  import { applyTheme, type ThemeSetting } from "../theme";
  import { boot, loadBootstrap, toast } from "../stores";

  let theme = $state<ThemeSetting>("dark");
  let focusGoals = $state(true);
  let notifyGoals = $state(true);
  let celebrateSound = $state(true);
  let launchAtLogin = $state(false);
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
    initialized = true;
  });

  async function finish(skip: boolean) {
    if (saving) return;
    saving = true;
    try {
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
