use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use chrono::Local;
use rusqlite::{params, Connection, OptionalExtension};

use crate::models::*;
use crate::timeutil::{local_day, split_at_midnight, today_str, week_bounds};

pub const DEFAULT_IGNORE: &[&str] = &[
    "NVIDIA App.exe",
    "NVIDIA Overlay.exe",
    "NVIDIA Share.exe",
    "nvsphelper64.exe",
    "Tailscale.exe",
    "Tailscale-IPN.exe",
    "Wallpaper Engine.exe",
    "wallpaper64.exe",
    "wallpaper32.exe",
    "GameOverlayUI.exe",
    "steamwebhelper.exe",
    "DiscordOverlay.exe",
    "DiscordOverlay64.exe",
    "Discord.exe",
    "explorer.exe",
    "SearchHost.exe",
    "StartMenuExperienceHost.exe",
    "ShellExperienceHost.exe",
    "LockApp.exe",
    "TextInputHost.exe",
    "ApplicationFrameHost.exe",
    "MsMpEng.exe",
    "SystemSettings.exe",
    "Taskmgr.exe",
    "Framewise.exe",
];

pub const DEFAULT_ADOBE: &[&str] = &[
    "Adobe Premiere Pro.exe",
    "Adobe After Effects.exe",
    "Photoshop.exe",
    "Illustrator.exe",
    "Adobe Media Encoder.exe",
    "Adobe Audition.exe",
    "Lightroom.exe",
    "Lightroom Classic.exe",
    "Adobe InDesign.exe",
    "Adobe Animate.exe",
    "Adobe Character Animator.exe",
    "Adobe Audition CC.exe",
];

pub struct Db {
    pub conn: Mutex<Connection>,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LiveSlice {
    pub id: i64,
    pub app: String,
    pub project_id: Option<i64>,
    pub adobe: bool,
    pub started_at: i64,
    pub source: String,
}

impl Db {
    pub fn open(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("creating data directory")?;
        }
        let conn = Connection::open(&path).context("opening database")?;
        configure(&conn)?;
        ensure_schema(&conn)?;
        seed(&conn)?;
        cleanup(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path,
        })
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn setting(&self, key: &str) -> Option<String> {
        get_setting(&self.lock(), key)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        set_setting(&self.lock(), key, value)
    }

    pub fn settings(&self, version: &str) -> Result<Settings> {
        read_settings(&self.lock(), &self.path.to_string_lossy(), version)
    }

    pub fn active_project_id(&self) -> Option<i64> {
        setting_i64(&self.lock(), "active_project_id")
    }

    pub fn pause_flag(&self) -> bool {
        setting_bool(&self.lock(), "paused", false)
    }

    pub fn runtime_config(&self) -> Result<RuntimeConfig> {
        let conn = self.lock();
        let active_project_id = setting_i64(&conn, "active_project_id");
        let active_project_name = active_project_id.and_then(|id| {
            conn.query_row(
                "SELECT name FROM projects WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten()
        });
        Ok(RuntimeConfig {
            paused: setting_bool(&conn, "paused", false),
            idle_seconds: setting_i64(&conn, "idle_minutes")
                .unwrap_or(3)
                .clamp(1, 180)
                * 60,
            active_project_id,
            active_project_name,
            active_project_tools: active_project_id
                .map(|id| {
                    project_tools_conn(&conn, id)
                        .unwrap_or_default()
                        .into_iter()
                        .collect()
                })
                .unwrap_or_default(),
            count_media_encoder_background: setting_bool(
                &conn,
                "count_media_encoder_background",
                false,
            ),
            ignore: get_app_rules_conn(&conn, "ignore")
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect(),
            adobe: get_app_rules_conn(&conn, "adobe")
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect(),
            notify_goals: setting_bool(&conn, "notify_goals", true),
        })
    }

    pub fn list_projects(&self, include_archived: bool) -> Result<Vec<Project>> {
        list_projects(&self.lock(), include_archived)
    }

    pub fn create_project(&self, name: &str, client: Option<&str>, color: &str) -> Result<Project> {
        create_project(&self.lock(), name, client, color)
    }

    pub fn update_project(
        &self,
        id: i64,
        name: &str,
        client: Option<&str>,
        color: &str,
        archived: bool,
    ) -> Result<()> {
        update_project(&self.lock(), id, name, client, color, archived)
    }

    pub fn delete_project(&self, id: i64) -> Result<()> {
        let conn = self.lock();
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        if setting_i64(&conn, "active_project_id") == Some(id) {
            set_setting(&conn, "active_project_id", "")?;
        }
        Ok(())
    }

    pub fn select_project(&self, id: Option<i64>) -> Result<()> {
        let conn = self.lock();
        if let Some(id) = id {
            let ok: Option<i64> = conn
                .query_row(
                    "SELECT id FROM projects WHERE id = ?1 AND archived = 0",
                    params![id],
                    |r| r.get(0),
                )
                .optional()?;
            if ok.is_none() {
                return Err(anyhow!("project not found"));
            }
            set_setting(&conn, "active_project_id", &id.to_string())?;
        } else {
            set_setting(&conn, "active_project_id", "")?;
        }
        Ok(())
    }

    pub fn list_goals(&self) -> Result<Vec<Goal>> {
        list_goals(&self.lock())
    }

    pub fn goals_progress(&self, today: &str) -> Result<Vec<GoalProgress>> {
        let conn = self.lock();
        let goals = list_goals(&conn)?;
        progress_for_goals(&conn, &goals, today)
    }

    pub fn create_goal(
        &self,
        scope: &str,
        project_id: Option<i64>,
        period: &str,
        target_seconds: i64,
    ) -> Result<Goal> {
        create_goal(&self.lock(), scope, project_id, period, target_seconds)
    }

    pub fn update_goal(&self, id: i64, project_id: Option<i64>, target_seconds: i64) -> Result<()> {
        update_goal(&self.lock(), id, project_id, target_seconds)
    }

    pub fn delete_goal(&self, id: i64) -> Result<()> {
        self.lock()
            .execute("DELETE FROM goals WHERE id = ?1", params![id])?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn save_session(
        &self,
        id: Option<i64>,
        app: &str,
        project_id: Option<i64>,
        started_at: i64,
        ended_at: i64,
        adobe_override: Option<bool>,
        focused: Option<bool>,
    ) -> Result<()> {
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        let app = app.trim();
        if app.is_empty() {
            return Err(anyhow!("app name is required"));
        }
        if ended_at <= started_at {
            return Err(anyhow!("end must be after start"));
        }
        if let Some(id) = id {
            tx.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        }
        let adobe = adobe_override.unwrap_or_else(|| is_adobe_name(app));
        insert_interval(
            &tx, app, project_id, started_at, ended_at, adobe, "manual", focused,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn delete_session(&self, id: i64) -> Result<()> {
        self.lock()
            .execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn reassign_sessions(&self, ids: &[i64], project_id: Option<i64>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        if let Some(pid) = project_id {
            let ok: Option<i64> = tx
                .query_row("SELECT id FROM projects WHERE id = ?1", params![pid], |r| {
                    r.get(0)
                })
                .optional()?;
            if ok.is_none() {
                return Err(anyhow!("project not found"));
            }
        }
        let placeholders = std::iter::repeat("?")
            .take(ids.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!("UPDATE sessions SET project_id = ?1 WHERE id IN ({placeholders})");
        let mut values: Vec<rusqlite::types::Value> = Vec::with_capacity(ids.len() + 1);
        values.push(project_id.into());
        for id in ids {
            values.push((*id).into());
        }
        tx.execute(&sql, rusqlite::params_from_iter(values))?;
        tx.commit()?;
        Ok(())
    }

    pub fn list_sessions(&self, filter: &SessionFilter) -> Result<Vec<Session>> {
        list_sessions(&self.lock(), filter)
    }

    pub fn report(
        &self,
        start_day: &str,
        end_day: &str,
        project_id: Option<i64>,
        unassigned: bool,
    ) -> Result<Report> {
        let conn = self.lock();
        report_for_range(&conn, start_day, end_day, project_id, unassigned)
    }

    pub fn heatmap(
        &self,
        start_day: &str,
        end_day: &str,
        project_id: Option<i64>,
        unassigned: bool,
    ) -> Result<Vec<HeatmapCell>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT day, SUM(ended_at - started_at) FROM sessions
             WHERE ended_at > started_at AND day BETWEEN ?1 AND ?2
               AND (?3 IS NULL OR project_id = ?3)
               AND (?4 = 0 OR project_id IS NULL)
             GROUP BY day ORDER BY day",
        )?;
        let rows = stmt.query_map(
            params![start_day, end_day, project_id, unassigned as i64],
            |r| {
                Ok(HeatmapCell {
                    day: r.get(0)?,
                    seconds: r.get(1)?,
                })
            },
        )?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn project_detail(&self, id: i64, today: &str) -> Result<ProjectDetail> {
        let conn = self.lock();
        let project = get_project(&conn, id)?;
        let (week_start, week_end) = week_bounds(today);
        let total: i64 = conn.query_row(
            "SELECT COALESCE(SUM(ended_at - started_at),0) FROM sessions
             WHERE ended_at > started_at AND project_id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        let today_seconds = sum_range(&conn, today, today, Some(id), false)?;
        let week_seconds = sum_range(&conn, &week_start, &week_end, Some(id), false)?;
        let per_app = per_app_detail(&conn, id)?;
        let (focused, unfocused): (i64, i64) = conn.query_row(
            "SELECT
                COALESCE(SUM(CASE WHEN in_tools = 1 THEN ended_at - started_at ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN in_tools = 0 THEN ended_at - started_at ELSE 0 END), 0)
             FROM sessions
             WHERE ended_at > started_at AND project_id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let tools = project_tools_conn(&conn, id)?;
        let per_day = per_day(&conn, "0000-01-01", "9999-12-31", Some(id), false)?;
        let (first, last, count): (Option<i64>, Option<i64>, i64) = conn.query_row(
            "SELECT MIN(started_at), MAX(ended_at), COUNT(*) FROM sessions
             WHERE ended_at > started_at AND project_id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        Ok(ProjectDetail {
            project,
            total_seconds: total,
            today_seconds,
            week_seconds,
            focused_seconds: focused,
            unfocused_seconds: unfocused,
            tools,
            per_app,
            per_day,
            first_activity: first,
            last_activity: last,
            session_count: count,
        })
    }

    pub fn status_totals(
        &self,
        today: &str,
        week_start: &str,
        week_end: &str,
        live: &[LiveSlice],
    ) -> Result<StatusTotals> {
        let conn = self.lock();
        status_totals(
            &conn,
            today,
            week_start,
            week_end,
            live,
            chrono::Local::now().timestamp(),
        )
    }

    pub fn get_app_rules(&self, kind: &str) -> Vec<String> {
        get_app_rules_conn(&self.lock(), kind)
    }

    pub fn replace_app_rules(&self, kind: &str, apps: &[String]) -> Result<()> {
        replace_app_rules(&self.lock(), kind, apps)
    }

    pub fn project_tools(&self, project_id: i64) -> Result<Vec<String>> {
        project_tools_conn(&self.lock(), project_id)
    }

    pub fn set_project_tools(&self, project_id: i64, apps: &[String]) -> Result<()> {
        set_project_tools(&self.lock(), project_id, apps)
    }

    pub fn checkpoint(&self) -> Result<()> {
        self.lock()
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .context("checkpointing database")
    }

    pub fn backup_to(&self, dest: &Path) -> Result<()> {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if dest.exists() {
            std::fs::remove_file(dest)?;
        }
        let conn = self.lock();
        let mut out = Connection::open(dest)?;
        let backup = rusqlite::backup::Backup::new(&conn, &mut out)?;
        backup.run_to_completion(10, Duration::from_millis(25), None)?;
        Ok(())
    }

    pub fn import_from(&self, source: &Path) -> Result<()> {
        if source == self.path {
            return Err(anyhow!("source is the active database"));
        }
        let src = Connection::open_with_flags(
            source,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .context("opening backup")?;
        let tables: i64 = src
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('sessions','projects','settings')",
                [],
                |r| r.get(0),
            )
            .context("not a Framewise backup")?;
        if tables < 3 {
            return Err(anyhow!("not a valid Framewise database"));
        }
        let mut conn = self.lock();
        {
            let backup = rusqlite::backup::Backup::new(&src, &mut conn)?;
            backup.run_to_completion(10, Duration::from_millis(25), None)?;
        }
        ensure_schema(&conn)?;
        cleanup(&conn)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub paused: bool,
    pub idle_seconds: i64,
    pub active_project_id: Option<i64>,
    pub active_project_name: Option<String>,
    pub active_project_tools: std::collections::HashSet<String>,
    pub count_media_encoder_background: bool,
    pub ignore: std::collections::HashSet<String>,
    pub adobe: std::collections::HashSet<String>,
    pub notify_goals: bool,
}

#[derive(Debug, Default)]
pub struct StatusTotals {
    pub today_seconds: i64,
    pub week_seconds: i64,
    pub today_by_project: Vec<ProjectTotal>,
    pub week_by_project: Vec<ProjectTotal>,
    pub today_by_app: Vec<AppTotal>,
}

pub fn configure(conn: &Connection) -> Result<()> {
    conn.busy_timeout(Duration::from_millis(5000))?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;
         PRAGMA foreign_keys = ON;
         PRAGMA auto_vacuum = INCREMENTAL;",
    )?;
    Ok(())
}

pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            client TEXT,
            color TEXT NOT NULL DEFAULT '#7dd3fc',
            archived INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app TEXT NOT NULL,
            project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
            started_at INTEGER NOT NULL,
            ended_at INTEGER NOT NULL,
            day TEXT NOT NULL,
            adobe INTEGER NOT NULL DEFAULT 0,
            source TEXT NOT NULL DEFAULT 'auto',
            in_tools INTEGER,
            created_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS project_tools (
            project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            app TEXT NOT NULL,
            PRIMARY KEY (project_id, app)
         );
         CREATE TABLE IF NOT EXISTS goals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scope TEXT NOT NULL DEFAULT 'global',
            project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
            period TEXT NOT NULL,
            target_seconds INTEGER NOT NULL,
            created_at INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS app_rules (
            kind TEXT NOT NULL,
            app TEXT NOT NULL,
            PRIMARY KEY (kind, app)
         );
         CREATE TABLE IF NOT EXISTS goal_notifications (
            goal_id INTEGER NOT NULL,
            period_key TEXT NOT NULL,
            notified_at INTEGER NOT NULL,
            PRIMARY KEY (goal_id, period_key)
         );
         CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS idx_sessions_day ON sessions(day);
         CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project_id);
         CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at);
         CREATE INDEX IF NOT EXISTS idx_sessions_day_project ON sessions(day, project_id);
         CREATE INDEX IF NOT EXISTS idx_sessions_app ON sessions(app);",
    )?;
    if !has_column(conn, "sessions", "in_tools")? {
        conn.execute("ALTER TABLE sessions ADD COLUMN in_tools INTEGER", [])?;
    }
    conn.execute(
        "INSERT INTO meta(key, value) VALUES('schema_version','2')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [],
    )?;
    Ok(())
}

fn has_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name.eq_ignore_ascii_case(column) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn seed(conn: &Connection) -> Result<()> {
    let seeded = get_setting(conn, "seeded").unwrap_or_default();
    if seeded != "1" {
        let tx = conn.unchecked_transaction()?;
        for app in DEFAULT_IGNORE {
            tx.execute(
                "INSERT OR IGNORE INTO app_rules(kind, app) VALUES('ignore', ?1)",
                params![app],
            )?;
        }
        for app in DEFAULT_ADOBE {
            tx.execute(
                "INSERT OR IGNORE INTO app_rules(kind, app) VALUES('adobe', ?1)",
                params![app],
            )?;
        }
        for (k, v) in [
            ("theme", "dark".to_string()),
            ("idle_minutes", "3".to_string()),
            ("paused", "false".to_string()),
            ("active_project_id", "".to_string()),
            ("count_media_encoder_background", "false".to_string()),
            ("launch_at_login", "false".to_string()),
            ("notify_goals", "true".to_string()),
            ("celebrate_sound", "true".to_string()),
            ("seeded", "1".to_string()),
        ] {
            set_setting(&tx, k, &v)?;
        }
        tx.commit()?;
    }

    // v1.1: no goals are created on first launch. Remove the old seeded
    // 4h daily / 20h weekly defaults once, without touching custom goals.
    if get_setting(conn, "default_goals_removed").as_deref() != Some("1") {
        conn.execute(
            "DELETE FROM goals
             WHERE scope = 'global' AND project_id IS NULL
               AND ((period = 'daily' AND target_seconds = 14400)
                 OR (period = 'weekly' AND target_seconds = 72000))",
            [],
        )?;
        set_setting(conn, "default_goals_removed", "1")?;
    }
    Ok(())
}

fn cleanup(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM sessions WHERE ended_at <= started_at", [])?;
    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |r| r.get::<_, String>(0),
    )
    .optional()
    .ok()
    .flatten()
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings(key, value) VALUES(?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn setting_bool(conn: &Connection, key: &str, default: bool) -> bool {
    get_setting(conn, key)
        .map(|v| v == "true" || v == "1")
        .unwrap_or(default)
}

pub fn setting_i64(conn: &Connection, key: &str) -> Option<i64> {
    get_setting(conn, key).and_then(|v| v.parse::<i64>().ok())
}

pub fn read_settings(conn: &Connection, data_path: &str, version: &str) -> Result<Settings> {
    let active_project_id = setting_i64(conn, "active_project_id");
    Ok(Settings {
        theme: get_setting(conn, "theme").unwrap_or_else(|| "dark".into()),
        idle_minutes: setting_i64(conn, "idle_minutes").unwrap_or(3).clamp(1, 180) as u32,
        paused: setting_bool(conn, "paused", false),
        active_project_id,
        count_media_encoder_background: setting_bool(conn, "count_media_encoder_background", false),
        launch_at_login: setting_bool(conn, "launch_at_login", false),
        notify_goals: setting_bool(conn, "notify_goals", true),
        celebrate_sound: setting_bool(conn, "celebrate_sound", true),
        ignore_apps: get_app_rules_conn(conn, "ignore"),
        adobe_apps: get_app_rules_conn(conn, "adobe"),
        data_path: data_path.to_string(),
        version: version.to_string(),
        portable: crate::portable::is_portable(),
    })
}

pub fn get_app_rules_conn(conn: &Connection, kind: &str) -> Vec<String> {
    let mut stmt =
        match conn.prepare("SELECT app FROM app_rules WHERE kind = ?1 ORDER BY lower(app)") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
    let rows = stmt.query_map(params![kind], |r| r.get::<_, String>(0));
    match rows {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    }
}

pub fn replace_app_rules(conn: &Connection, kind: &str, apps: &[String]) -> Result<()> {
    if kind != "ignore" && kind != "adobe" {
        return Err(anyhow!("unknown app list"));
    }
    let mut seen = std::collections::HashSet::new();
    let mut values = Vec::new();
    for app in apps {
        let app = app.trim();
        if app.is_empty() {
            continue;
        }
        let key = app.to_lowercase();
        if seen.insert(key) {
            values.push(app.to_string());
        }
    }
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM app_rules WHERE kind = ?1", params![kind])?;
    for app in &values {
        tx.execute(
            "INSERT OR IGNORE INTO app_rules(kind, app) VALUES(?1, ?2)",
            params![kind, app],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn is_adobe_name(app: &str) -> bool {
    DEFAULT_ADOBE.iter().any(|a| a.eq_ignore_ascii_case(app))
}

pub fn list_projects(conn: &Connection, include_archived: bool) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, client, color, archived, created_at FROM projects
         WHERE (?1 = 1 OR archived = 0)
         ORDER BY archived ASC, lower(name) ASC",
    )?;
    let rows = stmt.query_map(params![include_archived as i64], |r| {
        Ok(Project {
            id: r.get(0)?,
            name: r.get(1)?,
            client: r.get(2)?,
            color: r.get(3)?,
            archived: r.get::<_, i64>(4)? != 0,
            created_at: r.get(5)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn get_project(conn: &Connection, id: i64) -> Result<Project> {
    conn.query_row(
        "SELECT id, name, client, color, archived, created_at FROM projects WHERE id = ?1",
        params![id],
        |r| {
            Ok(Project {
                id: r.get(0)?,
                name: r.get(1)?,
                client: r.get(2)?,
                color: r.get(3)?,
                archived: r.get::<_, i64>(4)? != 0,
                created_at: r.get(5)?,
            })
        },
    )
    .map_err(|e| anyhow!("project not found: {e}"))
}

pub fn create_project(
    conn: &Connection,
    name: &str,
    client: Option<&str>,
    color: &str,
) -> Result<Project> {
    let name = name.trim();
    if name.is_empty() {
        return Err(anyhow!("project name is required"));
    }
    let color = normalize_color(color);
    let now = Local::now().timestamp();
    conn.execute(
        "INSERT INTO projects(name, client, color, archived, created_at) VALUES(?1, ?2, ?3, 0, ?4)",
        params![
            name,
            client.map(str::trim).filter(|s| !s.is_empty()),
            color,
            now
        ],
    )?;
    let id = conn.last_insert_rowid();
    get_project(conn, id)
}

pub fn update_project(
    conn: &Connection,
    id: i64,
    name: &str,
    client: Option<&str>,
    color: &str,
    archived: bool,
) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(anyhow!("project name is required"));
    }
    conn.execute(
        "UPDATE projects SET name = ?1, client = ?2, color = ?3, archived = ?4 WHERE id = ?5",
        params![
            name,
            client.map(str::trim).filter(|s| !s.is_empty()),
            normalize_color(color),
            archived as i64,
            id
        ],
    )?;
    if archived && setting_i64(conn, "active_project_id") == Some(id) {
        set_setting(conn, "active_project_id", "")?;
    }
    Ok(())
}

pub fn normalize_color(color: &str) -> String {
    let c = color.trim();
    if c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        c.to_lowercase()
    } else {
        "#7dd3fc".to_string()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn insert_interval(
    conn: &Connection,
    app: &str,
    project_id: Option<i64>,
    started_at: i64,
    ended_at: i64,
    adobe: bool,
    source: &str,
    focused: Option<bool>,
) -> Result<usize> {
    let now = Local::now().timestamp();
    let mut count = 0;
    for (start, end, day) in split_at_midnight(started_at, ended_at) {
        conn.execute(
            "INSERT INTO sessions(app, project_id, started_at, ended_at, day, adobe, source, in_tools, created_at)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                app,
                project_id,
                start,
                end,
                day,
                adobe as i64,
                source,
                focused.map(|value| value as i64),
                now
            ],
        )?;
        count += 1;
    }
    Ok(count)
}

pub fn insert_open_session(
    conn: &Connection,
    app: &str,
    project_id: Option<i64>,
    started_at: i64,
    adobe: bool,
    source: &str,
    focused: Option<bool>,
) -> Result<i64> {
    let now = Local::now().timestamp();
    conn.execute(
        "INSERT INTO sessions(app, project_id, started_at, ended_at, day, adobe, source, in_tools, created_at)
         VALUES(?1, ?2, ?3, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            app,
            project_id,
            started_at,
            local_day(started_at),
            adobe as i64,
            source,
            focused.map(|value| value as i64),
            now
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn extend_session(conn: &Connection, id: i64, end: i64) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET ended_at = ?1 WHERE id = ?2 AND ended_at < ?1",
        params![end, id],
    )?;
    Ok(())
}

pub fn close_session(conn: &Connection, id: i64, end: i64) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET ended_at = ?1 WHERE id = ?2",
        params![end, id],
    )?;
    conn.execute(
        "DELETE FROM sessions WHERE id = ?1 AND ended_at <= started_at",
        params![id],
    )?;
    Ok(())
}

fn project_filter_sql() -> &'static str {
    " AND (?3 IS NULL OR project_id = ?3) AND (?4 = 0 OR project_id IS NULL) "
}

pub fn sum_range(
    conn: &Connection,
    start_day: &str,
    end_day: &str,
    project_id: Option<i64>,
    unassigned: bool,
) -> Result<i64> {
    let sql = format!(
        "SELECT COALESCE(SUM(ended_at - started_at),0) FROM sessions
         WHERE ended_at > started_at AND day BETWEEN ?1 AND ?2 {}",
        project_filter_sql()
    );
    Ok(conn.query_row(
        &sql,
        params![start_day, end_day, project_id, unassigned as i64],
        |r| r.get(0),
    )?)
}

pub fn per_app(
    conn: &Connection,
    start_day: &str,
    end_day: &str,
    project_id: Option<i64>,
    unassigned: bool,
) -> Result<Vec<AppTotal>> {
    let sql = format!(
        "SELECT app, SUM(ended_at - started_at) AS s, MAX(adobe) FROM sessions
         WHERE ended_at > started_at AND day BETWEEN ?1 AND ?2 {}
         GROUP BY app ORDER BY s DESC",
        project_filter_sql()
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        params![start_day, end_day, project_id, unassigned as i64],
        |r| {
            Ok(AppTotal {
                app: r.get(0)?,
                seconds: r.get(1)?,
                adobe: r.get::<_, i64>(2)? != 0,
            })
        },
    )?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn per_app_detail(conn: &Connection, project_id: i64) -> Result<Vec<ProjectAppTotal>> {
    let mut stmt = conn.prepare(
        "SELECT app, SUM(ended_at - started_at) AS secs, in_tools
         FROM sessions
         WHERE ended_at > started_at AND project_id = ?1
         GROUP BY app, in_tools
         ORDER BY secs DESC, lower(app) ASC",
    )?;
    let rows = stmt.query_map(params![project_id], |r| {
        Ok(ProjectAppTotal {
            app: r.get(0)?,
            seconds: r.get(1)?,
            focused: r.get::<_, Option<i64>>(2)?.map(|value| value != 0),
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn project_tools_conn(conn: &Connection, project_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT app FROM project_tools WHERE project_id = ?1 ORDER BY lower(app) ASC")?;
    let rows = stmt.query_map(params![project_id], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn normalize_tools(apps: &[String]) -> Vec<String> {
    let mut out = std::collections::BTreeSet::new();
    for app in apps {
        let value = app.trim().to_lowercase();
        if !value.is_empty() {
            out.insert(value);
        }
    }
    out.into_iter().collect()
}

pub fn set_project_tools(conn: &Connection, project_id: i64, apps: &[String]) -> Result<()> {
    get_project(conn, project_id)?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM project_tools WHERE project_id = ?1",
        params![project_id],
    )?;
    for app in normalize_tools(apps) {
        tx.execute(
            "INSERT OR IGNORE INTO project_tools(project_id, app) VALUES(?1, ?2)",
            params![project_id, app],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn per_project(
    conn: &Connection,
    start_day: &str,
    end_day: &str,
    project_id: Option<i64>,
    unassigned: bool,
) -> Result<Vec<ProjectTotal>> {
    let sql = format!(
        "SELECT s.project_id, COALESCE(p.name, 'Unassigned'), SUM(s.ended_at - s.started_at) AS secs
         FROM sessions s LEFT JOIN projects p ON p.id = s.project_id
         WHERE s.ended_at > s.started_at AND s.day BETWEEN ?1 AND ?2 {}
         GROUP BY s.project_id ORDER BY secs DESC",
        project_filter_sql().replace("project_id", "s.project_id")
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        params![start_day, end_day, project_id, unassigned as i64],
        |r| {
            Ok(ProjectTotal {
                project_id: r.get(0)?,
                name: r.get(1)?,
                seconds: r.get(2)?,
            })
        },
    )?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn per_day(
    conn: &Connection,
    start_day: &str,
    end_day: &str,
    project_id: Option<i64>,
    unassigned: bool,
) -> Result<Vec<DayTotal>> {
    let sql = format!(
        "SELECT day, SUM(ended_at - started_at) AS secs FROM sessions
         WHERE ended_at > started_at AND day BETWEEN ?1 AND ?2 {}
         GROUP BY day ORDER BY day",
        project_filter_sql()
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        params![start_day, end_day, project_id, unassigned as i64],
        |r| {
            Ok(DayTotal {
                day: r.get(0)?,
                seconds: r.get(1)?,
            })
        },
    )?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn report_for_range(
    conn: &Connection,
    start_day: &str,
    end_day: &str,
    project_id: Option<i64>,
    unassigned: bool,
) -> Result<Report> {
    let total_seconds = sum_range(conn, start_day, end_day, project_id, unassigned)?;
    let per_app = per_app(conn, start_day, end_day, project_id, unassigned)?;
    let per_project = per_project(conn, start_day, end_day, project_id, unassigned)?;
    let per_day = per_day(conn, start_day, end_day, project_id, unassigned)?;
    let best_day = per_day
        .iter()
        .max_by_key(|d| d.seconds)
        .filter(|d| d.seconds > 0)
        .cloned();
    let sql = format!(
        "SELECT MIN(started_at), MAX(ended_at), COUNT(*) FROM sessions
         WHERE ended_at > started_at AND day BETWEEN ?1 AND ?2 {}",
        project_filter_sql()
    );
    let (first_activity, last_activity, session_count): (Option<i64>, Option<i64>, i64) = conn
        .query_row(
            &sql,
            params![start_day, end_day, project_id, unassigned as i64],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
    let goals = progress_for_goals(conn, &list_goals(conn)?, &today_str())?;
    Ok(Report {
        start_day: start_day.to_string(),
        end_day: end_day.to_string(),
        total_seconds,
        project_id,
        per_app,
        per_project,
        per_day,
        best_day,
        first_activity,
        last_activity,
        session_count,
        goals,
    })
}

pub fn status_totals(
    conn: &Connection,
    today: &str,
    week_start: &str,
    week_end: &str,
    live: &[LiveSlice],
    now: i64,
) -> Result<StatusTotals> {
    let ids: Vec<i64> = live.iter().map(|l| l.id).collect();
    let mut out = StatusTotals::default();

    let (today_total, week_total) = if ids.is_empty() {
        (
            sum_range(conn, today, today, None, false)?,
            sum_range(conn, week_start, week_end, None, false)?,
        )
    } else {
        let placeholders = std::iter::repeat("?")
            .take(ids.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql_today = format!(
            "SELECT COALESCE(SUM(ended_at - started_at),0) FROM sessions
             WHERE ended_at > started_at AND day = ?1 AND id NOT IN ({placeholders})"
        );
        let sql_week = format!(
            "SELECT COALESCE(SUM(ended_at - started_at),0) FROM sessions
             WHERE ended_at > started_at AND day BETWEEN ?1 AND ?2 AND id NOT IN ({placeholders})"
        );
        let mut params_today: Vec<rusqlite::types::Value> =
            vec![rusqlite::types::Value::Text(today.to_string())];
        params_today.extend(ids.iter().map(|id| (*id).into()));
        let mut params_week: Vec<rusqlite::types::Value> = vec![
            rusqlite::types::Value::Text(week_start.to_string()),
            rusqlite::types::Value::Text(week_end.to_string()),
        ];
        params_week.extend(ids.iter().map(|id| (*id).into()));
        (
            conn.query_row(&sql_today, rusqlite::params_from_iter(params_today), |r| {
                r.get(0)
            })?,
            conn.query_row(&sql_week, rusqlite::params_from_iter(params_week), |r| {
                r.get(0)
            })?,
        )
    };
    out.today_seconds = today_total;
    out.week_seconds = week_total;

    let mut today_apps: BTreeMap<String, (i64, bool)> = BTreeMap::new();
    let mut today_projects: BTreeMap<Option<i64>, i64> = BTreeMap::new();
    let mut week_projects: BTreeMap<Option<i64>, i64> = BTreeMap::new();

    // Base rows (active rows excluded so their live duration can be added).
    let placeholders = std::iter::repeat("?")
        .take(ids.len().max(1))
        .collect::<Vec<_>>()
        .join(",");
    if ids.is_empty() {
        let mut stmt = conn.prepare(
            "SELECT app, project_id, adobe, SUM(ended_at - started_at) FROM sessions
             WHERE ended_at > started_at AND day = ?1 GROUP BY app, project_id, adobe",
        )?;
        let rows = stmt.query_map(params![today], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<i64>>(1)?,
                r.get::<_, i64>(2)? != 0,
                r.get::<_, i64>(3)?,
            ))
        })?;
        for row in rows {
            let (app, pid, adobe, secs) = row?;
            let entry = today_apps.entry(app).or_insert((0, adobe));
            entry.0 += secs;
            *today_projects.entry(pid).or_insert(0) += secs;
        }
        let mut stmt_w = conn.prepare(
            "SELECT project_id, SUM(ended_at - started_at) FROM sessions
             WHERE ended_at > started_at AND day BETWEEN ?1 AND ?2 GROUP BY project_id",
        )?;
        let rows_w = stmt_w.query_map(params![week_start, week_end], |r| {
            Ok((r.get::<_, Option<i64>>(0)?, r.get::<_, i64>(1)?))
        })?;
        for row in rows_w {
            let (pid, secs) = row?;
            *week_projects.entry(pid).or_insert(0) += secs;
        }
    } else {
        let sql = format!(
            "SELECT app, project_id, adobe, day, SUM(ended_at - started_at) FROM sessions
             WHERE ended_at > started_at AND day BETWEEN ?1 AND ?2 AND id NOT IN ({placeholders})
             GROUP BY app, project_id, adobe, day"
        );
        let mut values: Vec<rusqlite::types::Value> = vec![
            rusqlite::types::Value::Text(week_start.to_string()),
            rusqlite::types::Value::Text(week_end.to_string()),
        ];
        values.extend(ids.iter().map(|id| (*id).into()));
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(values), |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<i64>>(1)?,
                r.get::<_, i64>(2)? != 0,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })?;
        for row in rows {
            let (app, pid, adobe, day, secs) = row?;
            if day == today {
                let entry = today_apps.entry(app).or_insert((0, adobe));
                entry.0 += secs;
            }
            *week_projects.entry(pid).or_insert(0) += secs;
            if day == today {
                *today_projects.entry(pid).or_insert(0) += secs;
            }
        }
    }

    // Add live slivers exactly once.
    for live in live {
        if live.started_at > now {
            continue;
        }
        let secs = now - live.started_at;
        if secs <= 0 {
            continue;
        }
        let day = local_day(live.started_at);
        let in_today = day == today;
        let in_week = day.as_str() >= week_start && day.as_str() <= week_end;
        if in_today {
            out.today_seconds += secs;
            let entry = today_apps
                .entry(live.app.clone())
                .or_insert((0, live.adobe));
            entry.0 += secs;
            *today_projects.entry(live.project_id).or_insert(0) += secs;
        }
        if in_week {
            out.week_seconds += secs;
            *week_projects.entry(live.project_id).or_insert(0) += secs;
        }
        if live.source == "render" {
            // Render slivers are additive project time; keep app totals honest.
            let _ = live;
        }
    }

    let names = project_names(conn)?;
    out.today_by_app = today_apps
        .into_iter()
        .filter(|(_, (s, _))| *s > 0)
        .map(|(app, (seconds, adobe))| AppTotal {
            app,
            seconds,
            adobe,
        })
        .collect();
    out.today_by_app
        .sort_by_key(|item| std::cmp::Reverse(item.seconds));
    out.today_by_project = today_projects
        .into_iter()
        .filter(|(_, s)| *s > 0)
        .map(|(pid, seconds)| ProjectTotal {
            project_id: pid,
            name: pid
                .and_then(|id| names.get(&id).cloned())
                .unwrap_or_else(|| "Unassigned".into()),
            seconds,
        })
        .collect();
    out.today_by_project
        .sort_by_key(|item| std::cmp::Reverse(item.seconds));
    out.week_by_project = week_projects
        .into_iter()
        .filter(|(_, s)| *s > 0)
        .map(|(pid, seconds)| ProjectTotal {
            project_id: pid,
            name: pid
                .and_then(|id| names.get(&id).cloned())
                .unwrap_or_else(|| "Unassigned".into()),
            seconds,
        })
        .collect();
    out.week_by_project
        .sort_by_key(|item| std::cmp::Reverse(item.seconds));
    Ok(out)
}

fn project_names(conn: &Connection) -> Result<std::collections::HashMap<i64, String>> {
    let mut stmt = conn.prepare("SELECT id, name FROM projects")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_sessions(conn: &Connection, filter: &SessionFilter) -> Result<Vec<Session>> {
    let limit = filter.limit.unwrap_or(500).clamp(1, 100_000);
    let mut stmt = conn.prepare(
        "SELECT s.id, s.app, s.project_id, p.name, s.started_at, s.ended_at, s.day, s.adobe, s.source
         FROM sessions s LEFT JOIN projects p ON p.id = s.project_id
         WHERE s.ended_at > s.started_at
           AND (?1 IS NULL OR s.day >= ?1)
           AND (?2 IS NULL OR s.day <= ?2)
           AND (?3 IS NULL OR s.project_id = ?3)
           AND (?4 = 0 OR s.project_id IS NULL)
         ORDER BY s.started_at DESC LIMIT ?5",
    )?;
    let rows = stmt.query_map(
        params![
            filter.start_day,
            filter.end_day,
            filter.project_id,
            filter.unassigned.unwrap_or(false) as i64,
            limit as i64
        ],
        |r| {
            Ok(Session {
                id: r.get(0)?,
                app: r.get(1)?,
                project_id: r.get(2)?,
                project_name: r.get(3)?,
                started_at: r.get(4)?,
                ended_at: r.get(5)?,
                day: r.get(6)?,
                adobe: r.get::<_, i64>(7)? != 0,
                source: r.get(8)?,
            })
        },
    )?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_goals(conn: &Connection) -> Result<Vec<Goal>> {
    let mut stmt = conn.prepare(
        "SELECT g.id, g.scope, g.project_id, p.name, g.period, g.target_seconds, g.created_at
         FROM goals g LEFT JOIN projects p ON p.id = g.project_id
         ORDER BY g.period ASC, g.scope ASC, lower(COALESCE(p.name,'')) ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Goal {
            id: r.get(0)?,
            scope: r.get(1)?,
            project_id: r.get(2)?,
            project_name: r.get(3)?,
            period: r.get(4)?,
            target_seconds: r.get(5)?,
            created_at: r.get(6)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn create_goal(
    conn: &Connection,
    scope: &str,
    project_id: Option<i64>,
    period: &str,
    target_seconds: i64,
) -> Result<Goal> {
    if period != "daily" && period != "weekly" {
        return Err(anyhow!("period must be daily or weekly"));
    }
    if target_seconds <= 0 {
        return Err(anyhow!("goal must be greater than zero"));
    }
    let scope = if scope == "project" {
        "project"
    } else {
        "global"
    };
    if scope == "project" && project_id.is_none() {
        return Err(anyhow!("project goal requires a project"));
    }
    if scope == "global" && project_id.is_some() {
        return Err(anyhow!("global goal cannot target a project"));
    }
    if let Some(pid) = project_id {
        let exists: Option<i64> = conn
            .query_row("SELECT id FROM projects WHERE id = ?1", params![pid], |r| {
                r.get(0)
            })
            .optional()?;
        if exists.is_none() {
            return Err(anyhow!("project not found"));
        }
    }
    let now = Local::now().timestamp();
    conn.execute(
        "INSERT INTO goals(scope, project_id, period, target_seconds, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5)",
        params![scope, project_id, period, target_seconds, now],
    )?;
    let id = conn.last_insert_rowid();
    list_goals(conn)?
        .into_iter()
        .find(|g| g.id == id)
        .ok_or_else(|| anyhow!("goal not found"))
}

pub fn update_goal(
    conn: &Connection,
    id: i64,
    project_id: Option<i64>,
    target_seconds: i64,
) -> Result<()> {
    if target_seconds <= 0 {
        return Err(anyhow!("goal must be greater than zero"));
    }
    let (scope, period): (String, String) = conn.query_row(
        "SELECT scope, period FROM goals WHERE id = ?1",
        params![id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    if scope == "project" && project_id.is_none() {
        return Err(anyhow!("project goal requires a project"));
    }
    let _ = period;
    conn.execute(
        "UPDATE goals SET project_id = ?1, target_seconds = ?2 WHERE id = ?3",
        params![project_id, target_seconds, id],
    )?;
    Ok(())
}

pub fn progress_for_goals(
    conn: &Connection,
    goals: &[Goal],
    today: &str,
) -> Result<Vec<GoalProgress>> {
    let (week_start, week_end) = week_bounds(today);
    let mut out = Vec::new();
    for goal in goals {
        let (start, end, key) = if goal.period == "weekly" {
            (
                week_start.clone(),
                week_end.clone(),
                format!("W{week_start}"),
            )
        } else {
            (today.to_string(), today.to_string(), today.to_string())
        };
        let seconds = sum_range(conn, &start, &end, goal.project_id, false)?;
        let target = goal.target_seconds.max(1);
        out.push(GoalProgress {
            goal: goal.clone(),
            period_key: key,
            start_day: start,
            end_day: end,
            seconds,
            target_seconds: target,
            percent: (seconds as f64 / target as f64) * 100.0,
            reached: seconds >= target,
        });
    }
    Ok(out)
}

pub fn goal_reached_once(conn: &Connection, goal_id: i64, period_key: &str) -> Result<bool> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM goal_notifications WHERE goal_id = ?1 AND period_key = ?2",
            params![goal_id, period_key],
            |r| r.get(0),
        )
        .optional()?;
    if exists.is_some() {
        return Ok(false);
    }
    conn.execute(
        "INSERT OR IGNORE INTO goal_notifications(goal_id, period_key, notified_at)
         VALUES(?1, ?2, ?3)",
        params![goal_id, period_key, Local::now().timestamp()],
    )?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        conn
    }

    fn ts(day: &str, hh: u32, mm: u32) -> i64 {
        let date = crate::timeutil::parse_day(day).unwrap();
        let naive = date.and_hms_opt(hh, mm, 0).unwrap();
        naive
            .and_local_timezone(Local)
            .single()
            .unwrap()
            .timestamp()
    }

    fn project(conn: &Connection, name: &str) -> i64 {
        create_project(conn, name, Some("Client"), "#7dd3fc")
            .unwrap()
            .id
    }

    #[test]
    fn report_groups_apps_projects_and_days() {
        let conn = test_db();
        let p = project(&conn, "YouTube Client 2");
        insert_interval(
            &conn,
            "Adobe Premiere Pro.exe",
            Some(p),
            ts("2026-10-20", 9, 0),
            ts("2026-10-20", 10, 0),
            true,
            "auto",
            None,
        )
        .unwrap();
        insert_interval(
            &conn,
            "Adobe After Effects.exe",
            Some(p),
            ts("2026-10-20", 10, 0),
            ts("2026-10-20", 11, 0),
            true,
            "auto",
            None,
        )
        .unwrap();
        insert_interval(
            &conn,
            "chrome.exe",
            None,
            ts("2026-10-20", 11, 0),
            ts("2026-10-20", 11, 30),
            false,
            "auto",
            None,
        )
        .unwrap();
        insert_interval(
            &conn,
            "Adobe Photoshop.exe",
            Some(p),
            ts("2026-10-21", 14, 0),
            ts("2026-10-21", 15, 0),
            true,
            "auto",
            None,
        )
        .unwrap();

        let report = report_for_range(&conn, "2026-10-20", "2026-10-21", None, false).unwrap();
        assert_eq!(report.total_seconds, 3 * 3600 + 1800);
        assert_eq!(report.session_count, 4);
        assert_eq!(report.per_day.len(), 2);
        assert_eq!(report.best_day.as_ref().unwrap().day, "2026-10-20");
        assert_eq!(report.per_project.len(), 2);
        assert_eq!(report.per_project[0].seconds, 3 * 3600);

        let p1 = report_for_range(&conn, "2026-10-20", "2026-10-21", Some(p), false).unwrap();
        assert_eq!(p1.total_seconds, 3 * 3600);
        assert_eq!(p1.per_app.len(), 3);

        let unassigned = report_for_range(&conn, "2026-10-20", "2026-10-21", None, true).unwrap();
        assert_eq!(unassigned.total_seconds, 1800);
    }

    #[test]
    fn project_tools_are_normalized_and_focus_is_split() {
        let conn = test_db();
        let p = project(&conn, "Focus");
        set_project_tools(
            &conn,
            p,
            &[
                " Photoshop.EXE ".into(),
                "zen.exe".into(),
                "photoshop.exe".into(),
            ],
        )
        .unwrap();
        assert_eq!(
            project_tools_conn(&conn, p).unwrap(),
            vec!["photoshop.exe".to_string(), "zen.exe".to_string()]
        );
        insert_interval(
            &conn,
            "Photoshop.exe",
            Some(p),
            ts("2026-10-20", 9, 0),
            ts("2026-10-20", 10, 0),
            true,
            "auto",
            Some(true),
        )
        .unwrap();
        insert_interval(
            &conn,
            "chrome.exe",
            Some(p),
            ts("2026-10-20", 10, 0),
            ts("2026-10-20", 10, 30),
            false,
            "auto",
            Some(false),
        )
        .unwrap();
        let apps = per_app_detail(&conn, p).unwrap();
        assert_eq!(apps.len(), 2);
        assert_eq!(apps[0].focused, Some(true));
        assert_eq!(apps[1].focused, Some(false));
        let (focused, unfocused): (i64, i64) = conn
            .query_row(
                "SELECT
                    COALESCE(SUM(CASE WHEN in_tools = 1 THEN ended_at - started_at ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN in_tools = 0 THEN ended_at - started_at ELSE 0 END), 0)
                 FROM sessions WHERE project_id = ?1",
                params![p],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(focused, 3600);
        assert_eq!(unfocused, 1800);
    }

    #[test]
    fn midnight_split_lands_in_both_days() {
        let conn = test_db();
        let p = project(&conn, "Night shift");
        insert_interval(
            &conn,
            "Adobe Premiere Pro.exe",
            Some(p),
            ts("2026-10-20", 23, 45),
            ts("2026-10-21", 0, 15),
            true,
            "auto",
            None,
        )
        .unwrap();
        let d1 = sum_range(&conn, "2026-10-20", "2026-10-20", Some(p), false).unwrap();
        let d2 = sum_range(&conn, "2026-10-21", "2026-10-21", Some(p), false).unwrap();
        assert_eq!(d1, 15 * 60);
        assert_eq!(d2, 15 * 60);
    }

    #[test]
    fn reassign_moves_time_between_projects() {
        let conn = test_db();
        let a = project(&conn, "A");
        let b = project(&conn, "B");
        insert_interval(
            &conn,
            "Photoshop.exe",
            Some(a),
            ts("2026-10-20", 9, 0),
            ts("2026-10-20", 10, 0),
            true,
            "auto",
            None,
        )
        .unwrap();
        let id: i64 = conn
            .query_row("SELECT id FROM sessions LIMIT 1", [], |r| r.get(0))
            .unwrap();
        conn.execute(
            "UPDATE sessions SET project_id = ?1 WHERE id = ?2",
            params![b, id],
        )
        .unwrap();
        assert_eq!(
            sum_range(&conn, "2026-10-20", "2026-10-20", Some(a), false).unwrap(),
            0
        );
        assert_eq!(
            sum_range(&conn, "2026-10-20", "2026-10-20", Some(b), false).unwrap(),
            3600
        );
    }

    #[test]
    fn goal_progress_uses_week_bounds() {
        let conn = test_db();
        let p = project(&conn, "A");
        let goals = vec![
            Goal {
                id: 1,
                scope: "global".into(),
                project_id: None,
                project_name: None,
                period: "daily".into(),
                target_seconds: 4 * 3600,
                created_at: 0,
            },
            Goal {
                id: 2,
                scope: "project".into(),
                project_id: Some(p),
                project_name: Some("A".into()),
                period: "weekly".into(),
                target_seconds: 20 * 3600,
                created_at: 0,
            },
        ];
        insert_interval(
            &conn,
            "Photoshop.exe",
            Some(p),
            ts("2026-10-19", 9, 0),
            ts("2026-10-19", 13, 0),
            true,
            "auto",
            None,
        )
        .unwrap();
        insert_interval(
            &conn,
            "Photoshop.exe",
            Some(p),
            ts("2026-10-23", 9, 0),
            ts("2026-10-23", 12, 0),
            true,
            "auto",
            None,
        )
        .unwrap();
        let progress = progress_for_goals(&conn, &goals, "2026-10-24").unwrap();
        assert_eq!(progress[0].seconds, 0);
        assert!(!progress[0].reached);
        assert_eq!(progress[1].seconds, 7 * 3600);
        assert_eq!(progress[1].start_day, "2026-10-19");
        assert_eq!(progress[1].end_day, "2026-10-25");
        assert!((progress[1].percent - 35.0).abs() < 0.01);
    }

    #[test]
    fn live_slice_adds_to_status_and_does_not_double_count() {
        let conn = test_db();
        insert_interval(
            &conn,
            "Photoshop.exe",
            None,
            ts("2026-10-24", 8, 0),
            ts("2026-10-24", 9, 0),
            true,
            "auto",
            None,
        )
        .unwrap();
        let id = insert_open_session(
            &conn,
            "Photoshop.exe",
            None,
            ts("2026-10-24", 10, 0),
            true,
            "auto",
            None,
        )
        .unwrap();
        let live = vec![LiveSlice {
            id,
            app: "Photoshop.exe".into(),
            project_id: None,
            adobe: true,
            started_at: ts("2026-10-24", 10, 0),
            source: "auto".into(),
        }];
        let now = ts("2026-10-24", 10, 30);
        let totals =
            status_totals(&conn, "2026-10-24", "2026-10-19", "2026-10-25", &live, now).unwrap();
        assert_eq!(totals.today_seconds, 5400);
        assert_eq!(totals.today_by_app[0].seconds, 5400);
    }

    #[test]
    fn cleanup_removes_empty_sessions() {
        let conn = test_db();
        let id =
            insert_open_session(&conn, "Photoshop.exe", None, 1000, true, "auto", None).unwrap();
        cleanup(&conn).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }
}
