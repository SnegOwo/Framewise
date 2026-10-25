<script lang="ts">
  import type { Session } from "../types";
  import Modal from "./Modal.svelte";
  import { api, errMessage } from "../api";
  import { activeProjects, loadBootstrap, settings, toast } from "../stores";
  import {
    fromDateTimeLocal,
    toDateTimeLocal,
  } from "../format";

  let {
    open = false,
    session = null,
    defaultDay = null,
    onclose,
  }: {
    open?: boolean;
    session?: Session | null;
    defaultDay?: string | null;
    onclose: () => void;
  } = $props();

  let appName = $state("");
  let projectId = $state<number | null>(null);
  let start = $state("");
  let end = $state("");
  let saving = $state(false);
  let error = $state("");
  let lastKey = $state("");

  $effect(() => {
    const key = `${open}:${session?.id ?? "new"}:${defaultDay ?? ""}`;
    if (key === lastKey) return;
    lastKey = key;
    if (!open) return;
    const now = Math.floor(Date.now() / 1000);
    if (session) {
      appName = session.app;
      projectId = session.projectId;
      start = toDateTimeLocal(session.startedAt);
      end = toDateTimeLocal(session.endedAt);
    } else {
      appName = "Adobe Premiere Pro.exe";
      projectId = null;
      let endTs = now;
      if (defaultDay) {
        const [y, m, d] = defaultDay.split("-").map(Number);
        const date = new Date(y, m - 1, d, 18, 0, 0, 0);
        endTs = Math.floor(date.getTime() / 1000);
      }
      start = toDateTimeLocal(endTs - 30 * 60);
      end = toDateTimeLocal(endTs);
    }
    error = "";
  });

  let appSuggestions = $derived(
    Array.from(
      new Set([
        ...($settings?.adobeApps ?? []),
        "Adobe Premiere Pro.exe",
        "Adobe After Effects.exe",
        "Photoshop.exe",
        "Illustrator.exe",
        "Adobe Media Encoder.exe",
      ]),
    ),
  );

  async function save() {
    const started = fromDateTimeLocal(start);
    const ended = fromDateTimeLocal(end);
    if (!appName.trim()) {
      error = "Enter an application name.";
      return;
    }
    if (!Number.isFinite(started) || !Number.isFinite(ended)) {
      error = "Enter a valid start and end time.";
      return;
    }
    if (ended <= started) {
      error = "End time must be after start time.";
      return;
    }
    saving = true;
    error = "";
    try {
      await api.saveSession(
        session?.id ?? null,
        appName.trim(),
        projectId,
        started,
        ended,
      );
      await loadBootstrap();
      toast(session ? "Session updated." : "Session added.", "success");
      onclose();
    } catch (cause) {
      error = errMessage(cause);
    } finally {
      saving = false;
    }
  }

  async function remove() {
    if (!session) return;
    if (!window.confirm("Delete this session? This cannot be undone.")) return;
    try {
      await api.deleteSession(session.id);
      await loadBootstrap();
      toast("Session deleted.");
      onclose();
    } catch (cause) {
      error = errMessage(cause);
    }
  }
</script>

<Modal {open} title={session ? "Edit session" : "Add session"} width={470} {onclose}>
  <div class="form">
    <div class="field">
      <label class="label" for="session-app">Application</label>
      <input
        id="session-app"
        class="input"
        bind:value={appName}
        list="session-app-suggestions"
        placeholder="Adobe Premiere Pro.exe"
      />
      <datalist id="session-app-suggestions">
        {#each appSuggestions as app}<option value={app}></option>{/each}
      </datalist>
    </div>

    <div class="field">
      <label class="label" for="session-project">Project</label>
      <select id="session-project" class="select" bind:value={projectId}>
        <option value={null}>Unassigned</option>
        {#each $activeProjects as project}
          <option value={project.id}>{project.name}</option>
        {/each}
      </select>
      <div class="hint">Assigning a project here overrides attribution for this session.</div>
    </div>

    <div class="row">
      <div class="field">
        <label class="label" for="session-start">Start</label>
        <input id="session-start" class="input" type="datetime-local" bind:value={start} />
      </div>
      <div class="field">
        <label class="label" for="session-end">End</label>
        <input id="session-end" class="input" type="datetime-local" bind:value={end} />
      </div>
    </div>

    {#if session?.source === "render"}
      <div class="note">This was background Media Encoder render time.</div>
    {/if}
    <div class="note">Sessions crossing midnight are split automatically for reporting.</div>
    {#if error}<div class="form-error">{error}</div>{/if}
  </div>

  {#snippet footer()}
    {#if session}
      <button class="btn btn-danger" onclick={remove} disabled={saving}>Delete</button>
    {/if}
    <span class="spacer"></span>
    <button class="btn" onclick={onclose} disabled={saving}>Cancel</button>
    <button class="btn btn-primary" onclick={save} disabled={saving}>
      {saving ? "Saving..." : session ? "Save session" : "Add session"}
    </button>
  {/snippet}
</Modal>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  .note {
    color: var(--muted-2);
    font-size: 10.5px;
    line-height: 1.5;
  }

  .spacer {
    flex: 1;
  }

  .form-error {
    padding: 7px 9px;
    border: 1px solid var(--danger);
    border-radius: 6px;
    background: var(--danger-soft);
    color: var(--danger);
    font-size: 11.5px;
  }
</style>
