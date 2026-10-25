<script lang="ts">
  import Modal from "./Modal.svelte";
  import Icon from "./Icon.svelte";

  let {
    version,
    notes,
    installing,
    oninstall,
    onclose,
  }: {
    version: string;
    notes: string;
    installing: boolean;
    oninstall: () => void;
    onclose: () => void;
  } = $props();
</script>

<Modal open title="Update available" width={480} {onclose}>
  <div class="update">
    <div class="update-mark"><Icon name="download" size={20} /></div>
    <div>
      <div class="update-title">Framewise {version} is ready.</div>
      <p class="update-copy">
        {#if installing}
          Installing the update. Framewise will restart when it finishes.
        {:else}
          Download and install the new version, then Framewise will restart.
        {/if}
      </p>
    </div>
  </div>
  {#if notes}
    <div class="update-notes">{notes}</div>
  {/if}
  {#snippet footer()}
    <button class="btn btn-ghost" onclick={onclose} disabled={installing}>Later</button>
    <button class="btn btn-primary" onclick={oninstall} disabled={installing}>
      {installing ? "Installing…" : "Install update"}
    </button>
  {/snippet}
</Modal>

<style>
  .update {
    display: flex;
    gap: 13px;
    align-items: flex-start;
  }

  .update-mark {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    flex: 0 0 auto;
    border-radius: 11px;
    background: var(--accent-soft);
    color: var(--accent-text);
  }

  .update-title {
    font-size: 15px;
    font-weight: 600;
    letter-spacing: -0.015em;
  }

  .update-copy {
    margin-top: 4px;
    color: var(--muted);
    font-size: 12.5px;
    line-height: 1.55;
  }

  .update-notes {
    max-height: 180px;
    margin-top: 16px;
    overflow-y: auto;
    padding: 12px 14px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--surface);
    color: var(--muted);
    font-size: 12px;
    line-height: 1.55;
    white-space: pre-wrap;
  }
</style>
