import { invoke } from "@tauri-apps/api/core";
import type {
  AppDetails,
  AppSettingsUpdate,
  Bootstrap,
  Goal,
  HeatmapCell,
  Project,
  ProjectDetail,
  Report,
  Session,
  SessionFilter,
  Settings,
  Status,
} from "./types";

export function errMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}

export const api = {
  bootstrap: () => invoke<Bootstrap>("get_bootstrap"),
  status: () => invoke<Status>("get_status"),

  listProjects: (includeArchived = false) =>
    invoke<Project[]>("list_projects", { includeArchived }),
  createProject: (name: string, client: string | null, color: string) =>
    invoke<Project>("create_project", { name, client, color }),
  updateProject: (
    id: number,
    name: string,
    client: string | null,
    color: string,
    archived: boolean,
  ) => invoke<void>("update_project", { id, name, client, color, archived }),
  deleteProject: (id: number) => invoke<void>("delete_project", { id }),
  selectProject: (projectId: number | null) =>
    invoke<void>("select_project", { projectId }),
  setPaused: (paused: boolean) => invoke<void>("set_paused", { paused }),

  settings: () => invoke<Settings>("get_settings"),
  updateSettings: (update: AppSettingsUpdate) =>
    invoke<Settings>("update_settings", { update }),
  replaceAppList: (kind: "ignore" | "adobe", apps: string[]) =>
    invoke<void>("replace_app_list", { kind, apps }),

  goals: () => invoke<Goal[]>("list_goals"),
  createGoal: (
    scope: "global" | "project",
    projectId: number | null,
    period: "daily" | "weekly",
    targetSeconds: number,
  ) => invoke<Goal>("create_goal", { scope, projectId, period, targetSeconds }),
  updateGoal: (id: number, projectId: number | null, targetSeconds: number) =>
    invoke<void>("update_goal", { id, projectId, targetSeconds }),
  deleteGoal: (id: number) => invoke<void>("delete_goal", { id }),

  report: (
    startDay: string,
    endDay: string,
    projectId: number | null = null,
    unassigned = false,
  ) => invoke<Report>("get_report", { startDay, endDay, projectId, unassigned }),
  heatmap: (
    startDay: string,
    endDay: string,
    projectId: number | null = null,
    unassigned = false,
  ) =>
    invoke<HeatmapCell[]>("get_heatmap", { startDay, endDay, projectId, unassigned }),

  sessions: (filter: SessionFilter) => invoke<Session[]>("get_sessions", { filter }),
  saveSession: (
    id: number | null,
    appName: string,
    projectId: number | null,
    startedAt: number,
    endedAt: number,
  ) =>
    invoke<void>("save_session", {
      id,
      appName,
      projectId,
      startedAt,
      endedAt,
    }),
  deleteSession: (id: number) => invoke<void>("delete_session", { id }),
  reassignSessions: (ids: number[], projectId: number | null) =>
    invoke<void>("reassign_sessions", { ids, projectId }),

  projectDetail: (id: number) => invoke<ProjectDetail>("get_project_detail", { id }),

  exportData: (
    format: "csv" | "json" | "md",
    startDay: string,
    endDay: string,
    projectId: number | null,
    unassigned: boolean,
    path: string,
  ) =>
    invoke<string>("export_data", {
      format,
      startDay,
      endDay,
      projectId,
      unassigned,
      path,
    }),
  backupDatabase: (path: string) =>
    invoke<string>("backup_database", { path }),
  importDatabase: (path: string) => invoke<void>("import_database", { path }),
  openDataFolder: () => invoke<void>("open_data_folder"),
  appDetails: (name: string) => invoke<AppDetails>("app_details", { name }),
  openFullApp: () => invoke<void>("open_full_app"),
  openMiniWindow: () => invoke<void>("open_mini_window"),
  setMiniPinned: (pinned: boolean) => invoke<void>("set_mini_pinned", { pinned }),
  closeMiniWindow: () => invoke<void>("close_mini_window"),
  quit: () => invoke<void>("quit_app"),
};
