<script lang="ts">
  import { appInitial, appLabel } from "../format";

  let {
    app,
    size = 30,
    tint = null,
    live = false,
  }: {
    app: string;
    size?: number;
    tint?: string | null;
    live?: boolean;
  } = $props();

  let label = $derived(appLabel(app));
  let initials = $derived(appInitial(app));
  let radius = $derived(Math.max(7, Math.round(size * 0.28)));
</script>

<span
  class="app-icon"
  class:live
  style:width="{size}px"
  style:height="{size}px"
  style:border-radius="{radius}px"
  style:font-size="{Math.max(9, Math.round(size * 0.34))}px"
  style:background={tint ? `color-mix(in srgb, ${tint} 16%, var(--surface-3))` : undefined}
  style:color={tint ?? undefined}
  title={label}
  aria-hidden="true"
>
  {initials}
</span>

<style>
  .app-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    background: var(--surface-3);
    color: var(--muted);
    font-weight: 600;
    letter-spacing: -0.02em;
    line-height: 1;
    user-select: none;
  }

  .app-icon.live {
    box-shadow: 0 0 0 1px var(--accent-line);
  }
</style>
