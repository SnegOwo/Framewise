use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub client: Option<String>,
    pub color: String,
    pub archived: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: i64,
    pub app: String,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub started_at: i64,
    pub ended_at: i64,
    pub day: String,
    pub adobe: bool,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppTotal {
    pub app: String,
    pub seconds: i64,
    pub adobe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTotal {
    pub project_id: Option<i64>,
    pub name: String,
    pub seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayTotal {
    pub day: String,
    pub seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: i64,
    pub scope: String,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub period: String,
    pub target_seconds: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalProgress {
    pub goal: Goal,
    pub period_key: String,
    pub start_day: String,
    pub end_day: String,
    pub seconds: i64,
    pub target_seconds: i64,
    pub percent: f64,
    pub reached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDetails {
    pub name: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedApp {
    pub app: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    pub idle_minutes: u32,
    pub paused: bool,
    pub active_project_id: Option<i64>,
    pub count_media_encoder_background: bool,
    pub launch_at_login: bool,
    pub notify_goals: bool,
    pub celebrate_sound: bool,
    pub goals_focus_only: bool,
    pub onboarding_complete: bool,
    pub auto_update_check: bool,
    pub ignore_apps: Vec<String>,
    pub adobe_apps: Vec<String>,
    pub data_path: String,
    pub version: String,
    pub portable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub paused: bool,
    pub locked: bool,
    pub idle: bool,
    pub idle_seconds: i64,
    pub active_project_id: Option<i64>,
    pub active_project_name: Option<String>,
    pub current_app: Option<String>,
    pub current_project_id: Option<i64>,
    pub tracking_since: Option<i64>,
    pub today_seconds: i64,
    pub week_seconds: i64,
    pub project_today_seconds: i64,
    pub project_week_seconds: i64,
    pub today_by_project: Vec<ProjectTotal>,
    pub week_by_project: Vec<ProjectTotal>,
    pub today_focused_by_project: Vec<ProjectTotal>,
    pub week_focused_by_project: Vec<ProjectTotal>,
    pub today_by_app: Vec<AppTotal>,
    pub render_active: bool,
    pub active_project_has_tools: bool,
    pub current_app_focused: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub start_day: String,
    pub end_day: String,
    pub total_seconds: i64,
    pub project_id: Option<i64>,
    pub per_app: Vec<AppTotal>,
    pub per_project: Vec<ProjectTotal>,
    pub per_day: Vec<DayTotal>,
    pub best_day: Option<DayTotal>,
    pub first_activity: Option<i64>,
    pub last_activity: Option<i64>,
    pub session_count: i64,
    pub goals: Vec<GoalProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapCell {
    pub day: String,
    pub seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub settings: Settings,
    pub projects: Vec<Project>,
    pub goals: Vec<Goal>,
    pub status: Status,
    pub today: String,
    pub week_start: String,
    pub week_end: String,
    pub streak_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFilter {
    pub start_day: Option<String>,
    pub end_day: Option<String>,
    pub project_id: Option<i64>,
    pub unassigned: Option<bool>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail {
    pub project: Project,
    pub total_seconds: i64,
    pub today_seconds: i64,
    pub week_seconds: i64,
    pub focused_seconds: i64,
    pub unfocused_seconds: i64,
    pub tools: Vec<String>,
    pub per_app: Vec<ProjectAppTotal>,
    pub per_day: Vec<DayTotal>,
    pub first_activity: Option<i64>,
    pub last_activity: Option<i64>,
    pub session_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAppTotal {
    pub app: String,
    pub seconds: i64,
    pub focused: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsUpdate {
    pub theme: Option<String>,
    pub idle_minutes: Option<u32>,
    pub count_media_encoder_background: Option<bool>,
    pub launch_at_login: Option<bool>,
    pub notify_goals: Option<bool>,
    pub celebrate_sound: Option<bool>,
    pub goals_focus_only: Option<bool>,
    pub onboarding_complete: Option<bool>,
    pub auto_update_check: Option<bool>,
}
