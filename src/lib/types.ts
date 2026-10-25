export interface Project {
  id: number;
  name: string;
  client: string | null;
  color: string;
  archived: boolean;
  createdAt: number;
}

export interface ProjectTotal {
  projectId: number | null;
  name: string;
  seconds: number;
}

export interface AppTotal {
  app: string;
  seconds: number;
  adobe: boolean;
}

export interface DayTotal {
  day: string;
  seconds: number;
}

export interface Goal {
  id: number;
  scope: "global" | "project";
  projectId: number | null;
  projectName: string | null;
  period: "daily" | "weekly";
  targetSeconds: number;
  createdAt: number;
}

export interface GoalProgress {
  goal: Goal;
  periodKey: string;
  startDay: string;
  endDay: string;
  seconds: number;
  targetSeconds: number;
  percent: number;
  reached: boolean;
}

export interface Settings {
  theme: "dark" | "light" | "system";
  idleMinutes: number;
  paused: boolean;
  activeProjectId: number | null;
  countMediaEncoderBackground: boolean;
  launchAtLogin: boolean;
  notifyGoals: boolean;
  ignoreApps: string[];
  adobeApps: string[];
  dataPath: string;
  version: string;
  portable: boolean;
}

export interface Status {
  paused: boolean;
  locked: boolean;
  idle: boolean;
  idleSeconds: number;
  activeProjectId: number | null;
  activeProjectName: string | null;
  currentApp: string | null;
  currentProjectId: number | null;
  trackingSince: number | null;
  todaySeconds: number;
  weekSeconds: number;
  projectTodaySeconds: number;
  projectWeekSeconds: number;
  todayByProject: ProjectTotal[];
  weekByProject: ProjectTotal[];
  todayByApp: AppTotal[];
  renderActive: boolean;
}

export interface Session {
  id: number;
  app: string;
  projectId: number | null;
  projectName: string | null;
  startedAt: number;
  endedAt: number;
  day: string;
  adobe: boolean;
  source: string;
}

export interface Report {
  startDay: string;
  endDay: string;
  totalSeconds: number;
  projectId: number | null;
  perApp: AppTotal[];
  perProject: ProjectTotal[];
  perDay: DayTotal[];
  bestDay: DayTotal | null;
  firstActivity: number | null;
  lastActivity: number | null;
  sessionCount: number;
  goals: GoalProgress[];
}

export interface HeatmapCell {
  day: string;
  seconds: number;
}

export interface Bootstrap {
  settings: Settings;
  projects: Project[];
  goals: Goal[];
  status: Status;
  today: string;
  weekStart: string;
  weekEnd: string;
  streakDays: number;
}

export interface SessionFilter {
  startDay?: string | null;
  endDay?: string | null;
  projectId?: number | null;
  unassigned?: boolean | null;
  limit?: number | null;
}

export interface ProjectDetail {
  project: Project;
  totalSeconds: number;
  todaySeconds: number;
  weekSeconds: number;
  perApp: AppTotal[];
  perDay: DayTotal[];
  firstActivity: number | null;
  lastActivity: number | null;
  sessionCount: number;
}

export interface AppSettingsUpdate {
  theme?: "dark" | "light" | "system";
  idleMinutes?: number;
  countMediaEncoderBackground?: boolean;
  launchAtLogin?: boolean;
  notifyGoals?: boolean;
}

export type ToastTone = "neutral" | "success" | "error";

export interface Toast {
  id: number;
  message: string;
  tone: ToastTone;
}
