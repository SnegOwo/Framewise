<script lang="ts">
  import { formatDuration } from "../format";

  let {
    value,
    formatter = formatDuration,
    duration = 480,
    class: className = "",
  }: {
    value: number;
    formatter?: (value: number) => string;
    duration?: number;
    class?: string;
  } = $props();

  // svelte-ignore state_referenced_locally
  let displayed = $state(value);
  // svelte-ignore state_referenced_locally
  let current = value;

  $effect(() => {
    const target = value;
    const from = current;
    if (typeof window === "undefined") return;
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduced || Math.abs(target - from) < 1 || document.visibilityState !== "visible") {
      current = target;
      displayed = target;
      return;
    }
    let raf = 0;
    const started = performance.now();
    const step = (now: number) => {
      const t = Math.min(1, (now - started) / duration);
      const eased = 1 - Math.pow(1 - t, 3);
      current = from + (target - from) * eased;
      displayed = current;
      if (t < 1) raf = requestAnimationFrame(step);
      else {
        current = target;
        displayed = target;
      }
    };
    raf = requestAnimationFrame(step);
    return () => cancelAnimationFrame(raf);
  });
</script>

<span class={className}>{formatter(Math.round(displayed))}</span>
