<script lang="ts">
  import { fly } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import { activeProject, activeProjects, switchProject } from "../stores";
  import { requestNewProject } from "../router";
  import { motionDuration } from "../motion";

  let {
    opens = "down",
    size = "md",
    variant = "default",
  }: { opens?: "down" | "up"; size?: "sm" | "md"; variant?: "default" | "hero" } = $props();

  let open = $state(false);
  let query = $state("");
  let container: HTMLDivElement | undefined = $state();
  let searchInput: HTMLInputElement | undefined = $state();

  let filtered = $derived(
    $activeProjects.filter((project) => {
      const q = query.trim().toLowerCase();
      if (!q) return true;
      return (
        project.name.toLowerCase().includes(q) ||
        (project.client ?? "").toLowerCase().includes(q)
      );
    }),
  );

  $effect(() => {
    if (!open) {
      query = "";
      return;
    }
    if (searchInput) queueMicrotask(() => searchInput?.focus());
  });

  function onWindowClick(event: MouseEvent) {
    if (open && container && !container.contains(event.target as Node)) open = false;
  }

  async function choose(projectId: number | null) {
    open = false;
    await switchProject(projectId);
  }
</script>

<svelte:window onclick={onWindowClick} onkeydown={(event) => {
  if (event.key === "Escape") open = false;
}} />

<div class="picker" bind:this={container}>
  <button
    class="chip"
    class:sm={size === "sm"}
    class:hero={variant === "hero"}
    class:open
    onclick={() => (open = !open)}
    aria-haspopup="listbox"
    aria-expanded={open}
  >
    {#if $activeProject}
      <span class="dot" style:background={$activeProject.color}></span>
      <span class="name">{$activeProject.name}</span>
    {:else}
      <span class="dot empty"></span>
      <span class="name muted">no project</span>
    {/if}
    <Icon name="chevron-down" size={12} />
  </button>

  {#if open}
    <div
      class="menu picker-menu"
      class:up={opens === "up"}
      transition:fly={{ y: opens === "up" ? 6 : -6, duration: motionDuration(140) }}
      role="listbox"
    >
      <div class="search">
        <Icon name="search" size={14} />
        <input bind:this={searchInput} bind:value={query} placeholder="Find a project" />
      </div>
      <div class="list">
        {#each filtered as project (project.id)}
          <button
            class="menu-item"
            class:selected={$activeProject?.id === project.id}
            onclick={() => choose(project.id)}
            role="option"
            aria-selected={$activeProject?.id === project.id}
          >
            <span class="dot" style:background={project.color}></span>
            <span class="pname">{project.name}</span>
            {#if $activeProject?.id === project.id}
              <span class="check"><Icon name="check" size={13} /></span>
            {:else if project.client}
              <span class="client">{project.client}</span>
            {/if}
          </button>
        {/each}
        {#if filtered.length === 0}
          <div class="no-match">No matching project</div>
        {/if}
        <div class="menu-sep"></div>
        <button
          class="menu-item"
          class:selected={$activeProject == null}
          onclick={() => choose(null)}
          role="option"
          aria-selected={$activeProject == null}
        >
          <span class="dot empty"></span>
          <span class="pname">No project</span>
          {#if $activeProject == null}<span class="check"><Icon name="check" size={13} /></span>{/if}
        </button>
      </div>
      <button
        class="new"
        onclick={() => {
          open = false;
          requestNewProject();
        }}
      >
        <Icon name="plus" size={13} /> New project
      </button>
    </div>
  {/if}
</div>

<style>
  .picker {
    position: relative;
    display: inline-flex;
    max-width: 100%;
  }

  .chip {
    height: 40px;
    padding: 0 13px;
    font-size: 15px;
    font-weight: 600;
    vertical-align: middle;
  }

  .chip.sm {
    height: 32px;
    padding: 0 10px;
    font-size: 13px;
  }

  .chip.hero {
    height: 46px;
    padding: 0 15px;
    font-size: 23px;
    font-weight: 600;
    letter-spacing: -0.02em;
    vertical-align: 2px;
  }

  .chip.hero .dot {
    width: 12px;
    height: 12px;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .picker-menu {
    top: calc(100% + 8px);
    left: 0;
    width: 300px;
  }

  .picker-menu.up {
    top: auto;
    bottom: calc(100% + 8px);
  }

  .search {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 11px;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--muted);
  }

  .search input {
    width: 100%;
    height: 24px;
    padding: 0 2px;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    outline: none;
  }

  .list {
    max-height: 260px;
    overflow-y: auto;
  }

  .pname {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .client {
    margin-left: auto;
    color: var(--muted-2);
    font-size: 11px;
    white-space: nowrap;
  }

  .check {
    display: flex;
    margin-left: auto;
    color: var(--accent-text);
  }

  .no-match {
    padding: 14px 10px;
    color: var(--muted-2);
    font-size: 13px;
  }

  .new {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 10px;
    border: 0;
    border-top: 1px solid var(--border-subtle);
    background: transparent;
    color: var(--muted);
    font-size: 13px;
    text-align: left;
  }

  .new:hover {
    background: var(--surface-3);
    color: var(--text);
  }
</style>
