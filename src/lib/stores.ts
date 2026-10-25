import { derived, get, writable } from "svelte/store";
import { api, errMessage } from "./api";
import { todayStr } from "./format";
import type {
  Bootstrap,
  Goal,
  Project,
  Status,
  Toast,
  ToastTone,
} from "./types";

export const boot = writable<Bootstrap | null>(null);
export const status = writable<Status | null>(null);
export const loading = writable(true);
export const fatalError = writable<string | null>(null);
export const refreshTick = writable(0);
export const toasts = writable<Toast[]>([]);
export const appInfo = writable<string | null>(null);

export function showAppInfo(app: string) {
  appInfo.set(app);
}

export const projectSetup = writable<{ project: Project; isNew: boolean } | null>(null);

export function openProjectSetup(project: Project, isNew = false) {
  projectSetup.set({ project, isNew });
}

export function closeProjectSetup() {
  projectSetup.set(null);
}

export const projects = derived(boot, ($boot) => $boot?.projects ?? []);
export const activeProjects = derived(projects, ($projects) =>
  $projects.filter((project) => !project.archived),
);
export const archivedProjects = derived(projects, ($projects) =>
  $projects.filter((project) => project.archived),
);
export const settings = derived(boot, ($boot) => $boot?.settings ?? null);
export const goals = derived(boot, ($boot) => $boot?.goals ?? []);
export const today = derived(boot, ($boot) => $boot?.today ?? todayStr());
export const streakDays = derived(boot, ($boot) => $boot?.streakDays ?? 0);
export const activeProject = derived([boot, status], ([$boot, $status]) => {
  const id = $status?.activeProjectId ?? $boot?.settings.activeProjectId ?? null;
  if (id == null) return null;
  return $boot?.projects.find((project) => project.id === id) ?? null;
});
export const dailyGoal = derived(goals, ($goals) =>
  $goals.find((goal) => goal.scope === "global" && goal.period === "daily"),
);
export const weeklyGoal = derived(goals, ($goals) =>
  $goals.find((goal) => goal.scope === "global" && goal.period === "weekly"),
);

export function goalSeconds(
  goal: Goal,
  $status: Status | null,
  focusOnly = false,
): number {
  if (!$status) return 0;
  if (goal.period === "daily") {
    if (goal.scope === "global") {
      return focusOnly
        ? $status.todayFocusedByProject.reduce((sum, row) => sum + row.seconds, 0)
        : $status.todaySeconds;
    }
    const rows = focusOnly ? $status.todayFocusedByProject : $status.todayByProject;
    return rows.find((row) => row.projectId === goal.projectId)?.seconds ?? 0;
  }
  if (goal.scope === "global") {
    return focusOnly
      ? $status.weekFocusedByProject.reduce((sum, row) => sum + row.seconds, 0)
      : $status.weekSeconds;
  }
  const rows = focusOnly ? $status.weekFocusedByProject : $status.weekByProject;
  return rows.find((row) => row.projectId === goal.projectId)?.seconds ?? 0;
}

export function projectSecondsToday(projectId: number, $status: Status | null): number {
  return (
    $status?.todayByProject.find((row) => row.projectId === projectId)?.seconds ?? 0
  );
}

export function projectSecondsWeek(projectId: number, $status: Status | null): number {
  return $status?.weekByProject.find((row) => row.projectId === projectId)?.seconds ?? 0;
}

let toastSeq = 0;

export function toast(message: string, tone: ToastTone = "neutral", duration = 3600) {
  const id = ++toastSeq;
  toasts.update((items) => [...items, { id, message, tone }].slice(-4));
  window.setTimeout(() => dismissToast(id), duration);
}

export function dismissToast(id: number) {
  toasts.update((items) => items.filter((item) => item.id !== id));
}

let bootstrapInFlight: Promise<void> | null = null;

export async function loadBootstrap(): Promise<void> {
  if (bootstrapInFlight) return bootstrapInFlight;
  bootstrapInFlight = (async () => {
    try {
      const data = await api.bootstrap();
      boot.set(data);
      status.set(data.status);
      fatalError.set(null);
    } catch (error) {
      fatalError.set(errMessage(error));
    } finally {
      loading.set(false);
      bootstrapInFlight = null;
    }
  })();
  return bootstrapInFlight;
}

export async function refreshStatus(): Promise<void> {
  if (!get(boot)) {
    await loadBootstrap();
    return;
  }
  try {
    const next = await api.status();
    status.set(next);
  } catch {
    // Tracker keeps running even if a poll fails momentarily.
  }
}

export async function switchProject(projectId: number | null): Promise<void> {
  try {
    await api.selectProject(projectId);
    await loadBootstrap();
    if (projectId == null) {
      toast("Project cleared. Adobe time is unassigned.");
    } else {
      const project = get(boot)?.projects.find((item) => item.id === projectId);
      toast(`Tracking to ${project?.name ?? "project"}.`, "success");
    }
  } catch (error) {
    toast(errMessage(error), "error");
  }
}

export async function togglePaused(): Promise<void> {
  const current = get(status);
  if (!current) return;
  try {
    await api.setPaused(!current.paused);
    await loadBootstrap();
    toast(current.paused ? "Tracking resumed." : "Tracking paused.");
  } catch (error) {
    toast(errMessage(error), "error");
  }
}

export function findProject(id: number | null | undefined): Project | undefined {
  if (id == null) return undefined;
  return get(projects).find((project) => project.id === id);
}

export function bumpRefresh() {
  refreshTick.update((n) => n + 1);
}
