use std::collections::BTreeMap;
use std::path::PathBuf;

use chrono::Local;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;

use crate::db::is_adobe_name;
use crate::export;
use crate::goals;
use crate::models::*;
use crate::state::{AppState, VERSION};
use crate::timeutil::{shift_day, today_str, week_bounds};
use crate::tray;

fn e(err: impl std::fmt::Display) -> String {
    err.to_string()
}

fn emit_state(app: &AppHandle) {
    let _ = app.emit("framewise:state-changed", ());
}

fn build_status(state: &AppState) -> Result<Status, String> {
    let cfg = state.db.runtime_config().map_err(e)?;
    let today = today_str();
    let (week_start, week_end) = week_bounds(&today);
    let snapshot = state.tracker.snapshot();
    let totals = state
        .db
        .status_totals(&today, &week_start, &week_end, &snapshot.slices)
        .map_err(e)?;
    let project_today = cfg
        .active_project_id
        .map(|id| {
            totals
                .today_by_project
                .iter()
                .find(|p| p.project_id == Some(id))
                .map(|p| p.seconds)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let project_week = cfg
        .active_project_id
        .map(|id| {
            totals
                .week_by_project
                .iter()
                .find(|p| p.project_id == Some(id))
                .map(|p| p.seconds)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let inner = state.tracker.lock();
    let idle = inner.idle_seconds >= cfg.idle_seconds;
    let idle_seconds = inner.idle_seconds;
    let paused = cfg.paused;
    let locked = inner.locked;
    let render_active = inner.render.is_some();
    drop(inner);
    let active_project_has_tools = !cfg.active_project_tools.is_empty();
    let current_app_focused = snapshot.current_app.as_ref().and_then(|app| {
        if active_project_has_tools && snapshot.current_project_id.is_some() {
            Some(cfg.active_project_tools.contains(&app.to_lowercase()))
        } else {
            None
        }
    });
    Ok(Status {
        paused,
        locked,
        idle,
        idle_seconds,
        active_project_id: cfg.active_project_id,
        active_project_name: cfg.active_project_name.clone(),
        current_app: snapshot.current_app,
        current_project_id: snapshot.current_project_id,
        tracking_since: snapshot.tracking_since,
        today_seconds: totals.today_seconds,
        week_seconds: totals.week_seconds,
        project_today_seconds: project_today,
        project_week_seconds: project_week,
        today_by_project: totals.today_by_project,
        week_by_project: totals.week_by_project,
        today_focused_by_project: totals.today_focused_by_project,
        week_focused_by_project: totals.week_focused_by_project,
        today_by_app: totals.today_by_app,
        render_active,
        active_project_has_tools,
        current_app_focused,
    })
}

fn idle_seconds_snapshot(state: &AppState) -> i64 {
    state.tracker.lock().idle_seconds
}

fn streak_days(state: &AppState, today: &str) -> i64 {
    let goals = match state.db.list_goals() {
        Ok(g) => g,
        Err(_) => return 0,
    };
    let target = goals
        .iter()
        .find(|g| g.scope == "global" && g.period == "daily")
        .map(|g| g.target_seconds)
        .unwrap_or(0);
    if target <= 0 {
        return 0;
    }
    let start = shift_day(today, -366);
    let focus_only = state
        .db
        .setting("goals_focus_only")
        .map(|value| value == "true")
        .unwrap_or(true);
    let per_day = if focus_only {
        match state.db.focus_per_day(&start, today) {
            Ok(days) => days,
            Err(_) => return 0,
        }
    } else {
        match state.db.report(&start, today, None, false) {
            Ok(r) => r.per_day,
            Err(_) => return 0,
        }
    };
    let map: BTreeMap<String, i64> = per_day.into_iter().map(|d| (d.day, d.seconds)).collect();
    goals::current_streak(&map, today, target)
}

#[tauri::command]
pub fn get_bootstrap(state: State<'_, AppState>) -> Result<Bootstrap, String> {
    let today = today_str();
    let (week_start, week_end) = week_bounds(&today);
    let settings = state.db.settings(VERSION).map_err(e)?;
    let projects = state.db.list_projects(true).map_err(e)?;
    let goals = state.db.list_goals().map_err(e)?;
    let status = build_status(&state)?;
    Ok(Bootstrap {
        settings,
        projects,
        goals,
        status,
        streak_days: streak_days(&state, &today),
        today,
        week_start,
        week_end,
    })
}

#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> Result<Status, String> {
    let mut status = build_status(&state)?;
    status.idle_seconds = idle_seconds_snapshot(&state);
    Ok(status)
}

#[tauri::command]
pub fn list_projects(
    state: State<'_, AppState>,
    include_archived: Option<bool>,
) -> Result<Vec<Project>, String> {
    state
        .db
        .list_projects(include_archived.unwrap_or(false))
        .map_err(e)
}

#[tauri::command]
pub async fn scan_adobe_apps() -> Result<Vec<DetectedApp>, String> {
    tauri::async_runtime::spawn_blocking(crate::adobe_scan::scan)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_known_apps(state: State<'_, AppState>) -> Result<Vec<AppTotal>, String> {
    state.db.known_apps().map_err(e)
}

#[tauri::command]
pub fn create_project(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    client: Option<String>,
    color: Option<String>,
) -> Result<Project, String> {
    let project = state
        .db
        .create_project(
            &name,
            client.as_deref(),
            color.as_deref().unwrap_or("#7dd3fc"),
        )
        .map_err(e)?;
    let _ = tray::refresh(&app);
    emit_state(&app);
    Ok(project)
}

#[tauri::command]
pub fn set_project_tools(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: i64,
    apps: Vec<String>,
) -> Result<(), String> {
    state.db.set_project_tools(project_id, &apps).map_err(e)?;
    if state.db.active_project_id() == Some(project_id) {
        state.tracker.cut_all(&state.db, Local::now().timestamp());
    }
    emit_state(&app);
    Ok(())
}

#[tauri::command]
pub fn update_project(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    name: String,
    client: Option<String>,
    color: Option<String>,
    archived: bool,
) -> Result<(), String> {
    let was_active = state.db.active_project_id() == Some(id);
    state
        .db
        .update_project(
            id,
            &name,
            client.as_deref(),
            color.as_deref().unwrap_or("#7dd3fc"),
            archived,
        )
        .map_err(e)?;
    if was_active {
        state.tracker.cut_all(&state.db, Local::now().timestamp());
    }
    let _ = tray::refresh(&app);
    emit_state(&app);
    Ok(())
}

#[tauri::command]
pub fn delete_project(app: AppHandle, state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.tracker.cut_all(&state.db, Local::now().timestamp());
    state.db.delete_project(id).map_err(e)?;
    let _ = tray::refresh(&app);
    emit_state(&app);
    Ok(())
}

#[tauri::command]
pub fn select_project(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: Option<i64>,
) -> Result<(), String> {
    state.db.select_project(project_id).map_err(e)?;
    state.tracker.cut_all(&state.db, Local::now().timestamp());
    let _ = tray::refresh(&app);
    emit_state(&app);
    Ok(())
}

#[tauri::command]
pub fn set_paused(app: AppHandle, state: State<'_, AppState>, paused: bool) -> Result<(), String> {
    state
        .db
        .set_setting("paused", if paused { "true" } else { "false" })
        .map_err(e)?;
    if paused {
        state.tracker.cut_all(&state.db, Local::now().timestamp());
    }
    let _ = tray::refresh(&app);
    emit_state(&app);
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    state.db.settings(VERSION).map_err(e)
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    update: AppSettingsUpdate,
) -> Result<Settings, String> {
    if let Some(theme) = update.theme.as_deref() {
        if !matches!(theme, "dark" | "light" | "system") {
            return Err("theme must be dark, light or system".into());
        }
        state.db.set_setting("theme", theme).map_err(e)?;
    }
    if let Some(idle) = update.idle_minutes {
        let idle = idle.clamp(1, 180);
        state
            .db
            .set_setting("idle_minutes", &idle.to_string())
            .map_err(e)?;
    }
    if let Some(v) = update.count_media_encoder_background {
        state
            .db
            .set_setting(
                "count_media_encoder_background",
                if v { "true" } else { "false" },
            )
            .map_err(e)?;
    }
    if let Some(v) = update.notify_goals {
        state
            .db
            .set_setting("notify_goals", if v { "true" } else { "false" })
            .map_err(e)?;
    }
    if let Some(v) = update.celebrate_sound {
        state
            .db
            .set_setting("celebrate_sound", if v { "true" } else { "false" })
            .map_err(e)?;
    }
    if let Some(v) = update.goals_focus_only {
        state
            .db
            .set_setting("goals_focus_only", if v { "true" } else { "false" })
            .map_err(e)?;
    }
    if let Some(v) = update.onboarding_complete {
        state
            .db
            .set_setting("onboarding_complete", if v { "true" } else { "false" })
            .map_err(e)?;
    }
    if let Some(v) = update.launch_at_login {
        if crate::portable::is_portable() {
            // Portable builds never touch the registry for autostart.
            state
                .db
                .set_setting("launch_at_login", "false")
                .map_err(e)?;
        } else {
            let result = if v {
                app.autolaunch().enable()
            } else {
                app.autolaunch().disable()
            };
            result.map_err(e)?;
            state
                .db
                .set_setting("launch_at_login", if v { "true" } else { "false" })
                .map_err(e)?;
        }
    }
    emit_state(&app);
    state.db.settings(VERSION).map_err(e)
}

#[tauri::command]
pub fn replace_app_list(
    app: AppHandle,
    state: State<'_, AppState>,
    kind: String,
    apps: Vec<String>,
) -> Result<(), String> {
    state.db.replace_app_rules(&kind, &apps).map_err(e)?;
    let _ = tray::refresh(&app);
    emit_state(&app);
    Ok(())
}

#[tauri::command]
pub fn list_goals(state: State<'_, AppState>) -> Result<Vec<Goal>, String> {
    state.db.list_goals().map_err(e)
}

#[tauri::command]
pub fn create_goal(
    app: AppHandle,
    state: State<'_, AppState>,
    scope: String,
    project_id: Option<i64>,
    period: String,
    target_seconds: i64,
) -> Result<Goal, String> {
    let goal = state
        .db
        .create_goal(&scope, project_id, &period, target_seconds)
        .map_err(e)?;
    emit_state(&app);
    Ok(goal)
}

#[tauri::command]
pub fn update_goal(
    app: AppHandle,
    state: State<'_, AppState>,
    id: i64,
    project_id: Option<i64>,
    target_seconds: i64,
) -> Result<(), String> {
    state
        .db
        .update_goal(id, project_id, target_seconds)
        .map_err(e)?;
    emit_state(&app);
    Ok(())
}

#[tauri::command]
pub fn delete_goal(app: AppHandle, state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_goal(id).map_err(e)?;
    emit_state(&app);
    Ok(())
}

#[tauri::command]
pub fn get_report(
    state: State<'_, AppState>,
    start_day: String,
    end_day: String,
    project_id: Option<i64>,
    unassigned: Option<bool>,
) -> Result<Report, String> {
    state
        .db
        .report(
            &start_day,
            &end_day,
            project_id,
            unassigned.unwrap_or(false),
        )
        .map_err(e)
}

#[tauri::command]
pub fn get_heatmap(
    state: State<'_, AppState>,
    start_day: String,
    end_day: String,
    project_id: Option<i64>,
    unassigned: Option<bool>,
) -> Result<Vec<HeatmapCell>, String> {
    state
        .db
        .heatmap(
            &start_day,
            &end_day,
            project_id,
            unassigned.unwrap_or(false),
        )
        .map_err(e)
}

#[tauri::command]
pub fn get_sessions(
    state: State<'_, AppState>,
    filter: SessionFilter,
) -> Result<Vec<Session>, String> {
    state.db.list_sessions(&filter).map_err(e)
}

#[tauri::command]
pub fn save_session(
    app: AppHandle,
    state: State<'_, AppState>,
    id: Option<i64>,
    app_name: String,
    project_id: Option<i64>,
    started_at: i64,
    ended_at: i64,
) -> Result<(), String> {
    let adobe = state
        .db
        .get_app_rules("adobe")
        .iter()
        .any(|a| a.eq_ignore_ascii_case(app_name.trim()))
        || is_adobe_name(&app_name);
    let focused = project_id
        .and_then(|id| state.db.project_tools(id).ok())
        .and_then(|tools| {
            if tools.is_empty() {
                None
            } else {
                Some(
                    tools
                        .iter()
                        .any(|tool| tool.eq_ignore_ascii_case(app_name.trim())),
                )
            }
        });
    state
        .db
        .save_session(
            id,
            &app_name,
            project_id,
            started_at,
            ended_at,
            Some(adobe),
            focused,
        )
        .map_err(e)?;
    emit_state(&app);
    Ok(())
}

#[tauri::command]
pub fn delete_session(app: AppHandle, state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_session(id).map_err(e)?;
    emit_state(&app);
    Ok(())
}

#[tauri::command]
pub fn reassign_sessions(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<i64>,
    project_id: Option<i64>,
) -> Result<(), String> {
    state.db.reassign_sessions(&ids, project_id).map_err(e)?;
    emit_state(&app);
    Ok(())
}

#[tauri::command]
pub fn get_project_detail(state: State<'_, AppState>, id: i64) -> Result<ProjectDetail, String> {
    state.db.project_detail(id, &today_str()).map_err(e)
}

#[tauri::command]
pub fn export_data(
    state: State<'_, AppState>,
    format: String,
    start_day: String,
    end_day: String,
    project_id: Option<i64>,
    unassigned: Option<bool>,
    path: String,
) -> Result<String, String> {
    let report = state
        .db
        .report(
            &start_day,
            &end_day,
            project_id,
            unassigned.unwrap_or(false),
        )
        .map_err(e)?;
    let sessions = state
        .db
        .list_sessions(&SessionFilter {
            start_day: Some(start_day),
            end_day: Some(end_day),
            project_id,
            unassigned,
            limit: Some(100_000),
        })
        .map_err(e)?;
    let path = PathBuf::from(path);
    export::export_report(&format, &report, &sessions, &path).map_err(e)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn backup_database(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let dest = PathBuf::from(path);
    state.db.backup_to(&dest).map_err(e)?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
pub fn import_database(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<(), String> {
    state.tracker.cut_all(&state.db, Local::now().timestamp());
    state.db.import_from(&PathBuf::from(path)).map_err(e)?;
    let _ = tray::refresh(&app);
    emit_state(&app);
    Ok(())
}

#[tauri::command]
pub fn open_data_folder(state: State<'_, AppState>) -> Result<(), String> {
    let folder = state
        .db
        .path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| state.db.path.clone());
    std::process::Command::new("explorer.exe")
        .arg(folder)
        .spawn()
        .map_err(e)?;
    Ok(())
}

#[tauri::command]
pub fn app_details(name: String) -> AppDetails {
    AppDetails {
        path: crate::winapi::process_path(&name),
        name,
    }
}

#[tauri::command]
pub async fn open_full_app(app: AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        let _ = crate::show_main_window(&handle, None);
    })
    .map_err(e)?;
    Ok(())
}

#[tauri::command]
pub async fn close_mini_window(app: AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window("mini") {
            let _ = window.close();
        }
    })
    .map_err(e)?;
    Ok(())
}

#[tauri::command]
pub async fn open_mini_window(app: AppHandle) -> Result<(), String> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        let _ = crate::show_mini_window(&handle, None);
    })
    .map_err(e)?;
    Ok(())
}

#[tauri::command]
pub fn set_mini_pinned(state: State<'_, AppState>, pinned: bool) {
    use std::sync::atomic::Ordering;
    state.mini_pinned.store(pinned, Ordering::Relaxed);
}

#[tauri::command]
pub fn quit_app(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    state.quitting.store(true, Ordering::SeqCst);
    state.tracker.cut_all(&state.db, Local::now().timestamp());
    let _ = state.db.checkpoint();
    app.exit(0);
    Ok(())
}
