<script lang="ts">
  let { name, size = 16, stroke = 1.6 }: { name: string; size?: number; stroke?: number } =
    $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === "dashboard"}
    <rect x="3" y="3" width="7" height="7" rx="1.4" />
    <rect x="14" y="3" width="7" height="7" rx="1.4" />
    <rect x="3" y="14" width="7" height="7" rx="1.4" />
    <rect x="14" y="14" width="7" height="7" rx="1.4" />
  {:else if name === "projects"}
    <path d="M3.5 7.5V6a1.5 1.5 0 0 1 1.5-1.5h4l2 2h8A1.5 1.5 0 0 1 20.5 8v9A1.5 1.5 0 0 1 19 18.5H5A1.5 1.5 0 0 1 3.5 17Z" />
  {:else if name === "reports"}
    <path d="M4 20V4" />
    <path d="M4 20h16" />
    <rect x="7.5" y="12" width="3" height="5" rx="0.6" />
    <rect x="12.5" y="8" width="3" height="9" rx="0.6" />
    <rect x="17.5" y="5" width="3" height="12" rx="0.6" />
  {:else if name === "sessions"}
    <path d="M8 6h12M8 12h12M8 18h12" />
    <circle cx="4" cy="6" r="1" fill="currentColor" stroke="none" />
    <circle cx="4" cy="12" r="1" fill="currentColor" stroke="none" />
    <circle cx="4" cy="18" r="1" fill="currentColor" stroke="none" />
  {:else if name === "settings"}
    <path d="M4 7h10M18 7h2M4 17h2M10 17h10" />
    <circle cx="16" cy="7" r="2" />
    <circle cx="8" cy="17" r="2" />
  {:else if name === "pause"}
    <rect x="7" y="5" width="3.4" height="14" rx="1" fill="currentColor" stroke="none" />
    <rect x="13.6" y="5" width="3.4" height="14" rx="1" fill="currentColor" stroke="none" />
  {:else if name === "play"}
    <path d="M8 5.5 18 12 8 18.5Z" fill="currentColor" stroke="none" />
  {:else if name === "chevron-down"}
    <path d="m6 9 6 6 6-6" />
  {:else if name === "chevron-right"}
    <path d="m9 6 6 6-6 6" />
  {:else if name === "arrow-right"}
    <path d="M4 12h16m-6-6 6 6-6 6" />
  {:else if name === "arrow-up-right"}
    <path d="M7 17 17 7M9 7h8v8" />
  {:else if name === "check"}
    <path d="m5 12.5 4.5 4.5L19 7" />
  {:else if name === "plus"}
    <path d="M12 5v14M5 12h14" />
  {:else if name === "minus"}
    <path d="M5 12h14" />
  {:else if name === "search"}
    <circle cx="10.5" cy="10.5" r="6" />
    <path d="m15 15 5 5" />
  {:else if name === "x"}
    <path d="M6 6l12 12M18 6 6 18" />
  {:else if name === "download"}
    <path d="M12 3v12m-4.5-4.5L12 15l4.5-4.5M4 19.5h16" />
  {:else if name === "upload"}
    <path d="M12 15V3m-4.5 4.5L12 3l4.5 4.5M4 19.5h16" />
  {:else if name === "folder"}
    <path d="M3.5 7.5V6A1.5 1.5 0 0 1 5 4.5h4l2 2h8A1.5 1.5 0 0 1 20.5 8v9A1.5 1.5 0 0 1 19 18.5H5A1.5 1.5 0 0 1 3.5 17Z" />
  {:else if name === "trash"}
    <path d="M4 7h16M9 7V4.5h6V7M6.5 7l.8 12.5h9.4L17.5 7M10 11v5M14 11v5" />
  {:else if name === "pencil"}
    <path d="m4 20 4.5-1 10-10a2.1 2.1 0 0 0-3-3l-10 10Z" />
    <path d="m14.5 7.5 3 3" />
  {:else if name === "archive"}
    <path d="M4 8h16v11H4zM3 4h18v4H3zM10 12h4" />
  {:else if name === "clock"}
    <circle cx="12" cy="12" r="8.5" />
    <path d="M12 7.5V12l3 2" />
  {:else if name === "target"}
    <circle cx="12" cy="12" r="8.5" />
    <circle cx="12" cy="12" r="4.5" />
    <circle cx="12" cy="12" r="1" fill="currentColor" stroke="none" />
  {:else if name === "flame"}
    <path d="M12 3s5.5 5.2 5.5 9.4A5.5 5.5 0 0 1 12 18a5.5 5.5 0 0 1-5.5-5.6C6.5 9.8 9 8 9 8s.2 2 1.5 2.7C11.2 8.2 12 3 12 3Z" />
  {:else if name === "calendar"}
    <rect x="3.5" y="5" width="17" height="15.5" rx="2.4" />
    <path d="M3.5 10h17M8 3.5V7M16 3.5V7" />
  {:else if name === "database"}
    <ellipse cx="12" cy="5.5" rx="8" ry="3" />
    <path d="M4 5.5v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6M4 11.5v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6" />
  {:else if name === "sun"}
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M19.1 4.9l-1.4 1.4M6.3 17.7l-1.4 1.4" />
  {:else if name === "moon"}
    <path d="M20 14.5A8 8 0 0 1 9.5 4a8.2 8.2 0 1 0 10.5 10.5Z" />
  {:else if name === "monitor"}
    <rect x="3" y="4" width="18" height="12" rx="1.5" />
    <path d="M8 20h8M12 16v4" />
  {:else if name === "more"}
    <circle cx="5" cy="12" r="1.2" fill="currentColor" stroke="none" />
    <circle cx="12" cy="12" r="1.2" fill="currentColor" stroke="none" />
    <circle cx="19" cy="12" r="1.2" fill="currentColor" stroke="none" />
  {:else if name === "external"}
    <path d="M14 4h6v6M20 4l-9 9M18 14v5.5H4.5V6H10" />
  {:else if name === "pin"}
    <path d="M9 3.5h6l-1 5 3 3H7l3-3-1-5Z" />
    <path d="M12 11.5v9" />
  {:else if name === "info"}
    <circle cx="12" cy="12" r="8.5" />
    <path d="M12 10.5V17M12 7.5h.01" />
  {:else if name === "minus-circle"}
    <circle cx="12" cy="12" r="8.5" />
    <path d="M8.5 12h7" />
  {:else}
    <circle cx="12" cy="12" r="8" />
  {/if}
</svg>
