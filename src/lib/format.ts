import { writable } from "svelte/store";

export type DateStyle = "english" | "system";

const DATE_KEY = "framewise-date-format";

export function getDateStyle(): DateStyle {
  try {
    return localStorage.getItem(DATE_KEY) === "system" ? "system" : "english";
  } catch {
    return "english";
  }
}

export function setDateStyle(style: DateStyle) {
  try {
    localStorage.setItem(DATE_KEY, style);
  } catch {
    // ignore
  }
  localeVersion.update((n) => n + 1);
}

// Bumped so components can re-render when the date style changes.
export const localeVersion = writable(0);

/** Locale for user-facing dates. English by default, or the OS locale. */
export function dateLocale(): string | undefined {
  return getDateStyle() === "english" ? "en-US" : undefined;
}


export function toDay(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

export function parseDay(day: string): Date {
  const [y, m, d] = day.split("-").map(Number);
  return new Date(y, (m ?? 1) - 1, d ?? 1, 12, 0, 0, 0);
}

export function todayStr(): string {
  return toDay(new Date());
}

export function shiftDay(day: string, delta: number): string {
  const d = parseDay(day);
  d.setDate(d.getDate() + delta);
  return toDay(d);
}

export function addDays(date: Date, delta: number): Date {
  const next = new Date(date);
  next.setDate(next.getDate() + delta);
  return next;
}

export function mondayOf(day: string): string {
  const d = parseDay(day);
  const offset = (d.getDay() + 6) % 7;
  d.setDate(d.getDate() - offset);
  return toDay(d);
}

export function weekBounds(day: string): { start: string; end: string } {
  const start = mondayOf(day);
  return { start, end: shiftDay(start, 6) };
}

export function monthStart(day: string): string {
  const d = parseDay(day);
  return toDay(new Date(d.getFullYear(), d.getMonth(), 1, 12));
}

export function daysBetween(start: string, end: string): string[] {
  const out: string[] = [];
  let cur = start;
  let guard = 0;
  while (cur <= end && guard < 3660) {
    out.push(cur);
    cur = shiftDay(cur, 1);
    guard += 1;
  }
  return out;
}

export function formatDuration(seconds: number, zero = "0m"): string {
  const s = Math.max(0, Math.floor(seconds));
  if (s <= 0) return zero;
  if (s < 60) return "<1m";
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  if (h > 0) return `${h}h ${String(m).padStart(2, "0")}m`;
  return `${m}m`;
}

export function formatDurationLong(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  if (h > 0) return `${h}h ${String(m).padStart(2, "0")}m ${String(sec).padStart(2, "0")}s`;
  if (m > 0) return `${m}m ${String(sec).padStart(2, "0")}s`;
  return `${sec}s`;
}

export function formatHMS(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  return [Math.floor(s / 3600), Math.floor((s % 3600) / 60), s % 60]
    .map((v) => String(v).padStart(2, "0"))
    .join(":");
}

export function formatHours(seconds: number): string {
  const h = Math.max(0, seconds) / 3600;
  if (h >= 10) return `${h.toFixed(0)}h`;
  if (h >= 1) return `${h.toFixed(1).replace(/\.0$/, "")}h`;
  return `${Math.round(h * 60)}m`;
}

export function formatDate(day: string, opts?: Intl.DateTimeFormatOptions): string {
  return parseDay(day).toLocaleDateString(dateLocale(), opts ?? { month: "short", day: "numeric" });
}

export function formatWeekday(day: string): string {
  return parseDay(day).toLocaleDateString(dateLocale(), { weekday: "short" });
}

export function formatMonth(day: string): string {
  return parseDay(day).toLocaleDateString(dateLocale(), { month: "short" });
}

export function formatMinutes(minutes: number): string {
  return `${Math.round(minutes)} min`;
}

export function formatDateLong(day: string): string {
  return parseDay(day).toLocaleDateString(dateLocale(), {
    weekday: "long",
    month: "long",
    day: "numeric",
    year: "numeric",
  });
}

export function formatRange(start: string, end: string): string {
  if (start === end) return formatDateLong(start);
  const s = parseDay(start);
  const e = parseDay(end);
  const sameYear = s.getFullYear() === e.getFullYear();
  const left = s.toLocaleDateString(dateLocale(), {
    month: "short",
    day: "numeric",
    ...(sameYear ? {} : { year: "numeric" }),
  });
  const right = e.toLocaleDateString(dateLocale(), {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
  return `${left} - ${right}`;
}

export function formatTimestamp(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString(dateLocale(), {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
}

export function formatDateTime(ts: number): string {
  return new Date(ts * 1000).toLocaleString(dateLocale(), {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
}

export function toDateTimeLocal(ts: number): string {
  const d = new Date(ts * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function fromDateTimeLocal(value: string): number {
  return Math.floor(new Date(value).getTime() / 1000);
}

export function daysAgo(n: number, from = todayStr()): string {
  return shiftDay(from, -n);
}

export function plural(n: number, word: string): string {
  return `${n} ${word}${n === 1 ? "" : "s"}`;
}

type AppMeta = { name: string; description: string; initials?: string };

const APP_META: Record<string, AppMeta> = {
  "adobe premiere pro": { name: "Premiere Pro", description: "Video editing", initials: "Pr" },
  "adobe after effects": {
    name: "After Effects",
    description: "Motion design and compositing",
    initials: "Ae",
  },
  afterfx: { name: "After Effects", description: "Motion design and compositing", initials: "Ae" },
  "adobe media encoder": {
    name: "Media Encoder",
    description: "Encoding and background renders",
    initials: "Me",
  },
  photoshop: { name: "Photoshop", description: "Image editing and compositing", initials: "Ps" },
  illustrator: { name: "Illustrator", description: "Vector design", initials: "Ai" },
  "adobe audition": { name: "Audition", description: "Audio editing and mixing", initials: "Au" },
  lightroom: { name: "Lightroom", description: "Photo editing", initials: "Lr" },
  "lightroom classic": {
    name: "Lightroom Classic",
    description: "Photo editing and cataloguing",
    initials: "LrC",
  },
  "adobe indesign": { name: "InDesign", description: "Page layout", initials: "Id" },
  "adobe animate": { name: "Animate", description: "2D animation", initials: "An" },
  "adobe character animator": {
    name: "Character Animator",
    description: "Puppet animation",
    initials: "Ch",
  },
  mpvnet: { name: "mpv.net", description: "Video player", initials: "mp" },
  mpv: { name: "mpv", description: "Video player", initials: "mp" },
  vlc: { name: "VLC", description: "Video player", initials: "VL" },
  chrome: { name: "Google Chrome", description: "Web browser", initials: "Ch" },
  msedge: { name: "Microsoft Edge", description: "Web browser", initials: "Ed" },
  firefox: { name: "Firefox", description: "Web browser", initials: "Ff" },
  zen: { name: "Zen Browser", description: "Web browser", initials: "Ze" },
  code: { name: "Visual Studio Code", description: "Code editor", initials: "VS" },
  windowsterminal: { name: "Windows Terminal", description: "Terminal", initials: "Wt" },
  explorer: { name: "File Explorer", description: "File manager", initials: "Ex" },
  spotify: { name: "Spotify", description: "Music", initials: "Sp" },
  slack: { name: "Slack", description: "Team chat", initials: "Sl" },
  discord: { name: "Discord", description: "Voice and chat", initials: "Di" },
  obs64: { name: "OBS Studio", description: "Screen recording and streaming", initials: "Ob" },
  qview: { name: "qView", description: "Image viewer", initials: "qV" },
  lightshot: { name: "Lightshot", description: "Screenshot tool", initials: "Li" },
  notepad: { name: "Notepad", description: "Text editor", initials: "No" },
  blender: { name: "Blender", description: "3D creation", initials: "Bl" },
  resolve: { name: "DaVinci Resolve", description: "Video editing and color", initials: "DR" },
  "davinci resolve": {
    name: "DaVinci Resolve",
    description: "Video editing and color",
    initials: "DR",
  },
  figma: { name: "Figma", description: "Interface design", initials: "Fi" },
  notion: { name: "Notion", description: "Notes and docs", initials: "No" },
  steam: { name: "Steam", description: "Games", initials: "St" },
  "1password": { name: "1Password", description: "Password manager", initials: "1P" },
};

function appKey(app: string): string {
  return app.replace(/\.exe$/i, "").trim().toLowerCase();
}

export function appLabel(app: string): string {
  return APP_META[appKey(app)]?.name ?? app.replace(/\.exe$/i, "");
}

export function appDescription(app: string): string {
  return APP_META[appKey(app)]?.description ?? "Unknown app";
}

export function appInitial(app: string): string {
  const key = appKey(app);
  const meta = APP_META[key];
  if (meta?.initials) return meta.initials;
  const clean = meta?.name ?? app.replace(/\.exe$/i, "");
  const parts = clean.split(/\s+/).filter(Boolean);
  if (parts.length >= 2) return (parts[0][0] + parts[1][0]).toUpperCase();
  return clean.slice(0, 2);
}

export function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.max(0, Math.min(100, value));
}

export function fillDays(
  start: string,
  end: string,
  rows: { day: string; seconds: number }[],
): { day: string; seconds: number }[] {
  const map = new Map(rows.map((r) => [r.day, r.seconds]));
  return daysBetween(start, end).map((day) => ({ day, seconds: map.get(day) ?? 0 }));
}
