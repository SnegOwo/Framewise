import { writable } from "svelte/store";

export type Page = "dashboard" | "projects" | "reports" | "sessions" | "settings";

export const PAGES: { id: Page; label: string; key: string }[] = [
  { id: "dashboard", label: "Dashboard", key: "1" },
  { id: "projects", label: "Projects", key: "2" },
  { id: "reports", label: "Reports", key: "3" },
  { id: "sessions", label: "Sessions", key: "4" },
  { id: "settings", label: "Settings", key: "5" },
];

function validPage(value: string | null): Page {
  const found = PAGES.find((page) => page.id === value);
  return found?.id ?? "dashboard";
}

const params = new URLSearchParams(window.location.search);
const initialPage = validPage(params.get("page"));
const initialNewProject = params.get("new") === "1";

export const page = writable<Page>(initialPage);
export const commandOpen = writable(false);
export const newProjectSignal = writable(initialNewProject ? 1 : 0);
export const sessionsDay = writable<string | null>(null);
export const sessionsProject = writable<number | null>(null);

export function navigate(next: Page) {
  page.set(next);
  commandOpen.set(false);
  try {
    const url = new URL(window.location.href);
    url.searchParams.delete("page");
    url.searchParams.delete("new");
    window.history.replaceState(null, "", url.pathname + url.hash);
  } catch {
    // ignore
  }
}

export function requestNewProject() {
  page.set("projects");
  newProjectSignal.update((n) => n + 1);
}

if (initialPage !== "dashboard" || initialNewProject) {
  try {
    const url = new URL(window.location.href);
    url.searchParams.delete("page");
    url.searchParams.delete("new");
    window.history.replaceState(null, "", url.pathname + url.hash);
  } catch {
    // ignore
  }
}
