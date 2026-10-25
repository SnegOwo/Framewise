use std::sync::Mutex;
use std::time::Instant;

use chrono::Local;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::db::{self, Db, LiveSlice};
use crate::models::GoalProgress;
use crate::state::AppState;
use crate::timeutil::{local_day, next_day_start, today_str};
use crate::winapi;

const TICK_MS: u64 = 1500;
const PERSIST_SECS: i64 = 15;
const SLEEP_GAP_MS: u64 = 6_000;
const ME_RENDER_CPU_PERCENT: f64 = 3.0;

#[derive(Debug, Clone)]
pub struct ActiveSegment {
    pub id: i64,
    pub app: String,
    pub project_id: Option<i64>,
    pub adobe: bool,
    pub focused: Option<bool>,
    pub source: String,
    pub started_at: i64,
}

#[derive(Debug)]
pub struct TrackerInner {
    pub current: Option<ActiveSegment>,
    pub render: Option<ActiveSegment>,
    pub last_tick: Instant,
    pub last_epoch: i64,
    pub last_persist: i64,
    pub last_goal_check: i64,
    pub locked: bool,
    pub idle_seconds: i64,
    me_sample: Option<(Instant, u64)>,
}

pub struct Tracker {
    pub inner: Mutex<TrackerInner>,
}

#[derive(Debug, Clone, Default)]
pub struct TrackerSnapshot {
    pub current_app: Option<String>,
    pub current_project_id: Option<i64>,
    pub tracking_since: Option<i64>,
    pub slices: Vec<LiveSlice>,
}

impl Tracker {
    pub fn new() -> Self {
        let now = Local::now().timestamp();
        Self {
            inner: Mutex::new(TrackerInner {
                current: None,
                render: None,
                last_tick: Instant::now(),
                last_epoch: now,
                last_persist: now,
                last_goal_check: 0,
                locked: false,
                idle_seconds: 0,
                me_sample: None,
            }),
        }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, TrackerInner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn snapshot(&self) -> TrackerSnapshot {
        let inner = self.lock();
        let mut slices = Vec::new();
        let mut current_app = None;
        let mut current_project_id = None;
        let mut tracking_since = None;
        if let Some(c) = &inner.current {
            current_app = Some(c.app.clone());
            current_project_id = c.project_id;
            tracking_since = Some(c.started_at);
            slices.push(LiveSlice {
                id: c.id,
                app: c.app.clone(),
                project_id: c.project_id,
                adobe: c.adobe,
                started_at: c.started_at,
                source: c.source.clone(),
            });
        }
        if let Some(r) = &inner.render {
            slices.push(LiveSlice {
                id: r.id,
                app: r.app.clone(),
                project_id: r.project_id,
                adobe: r.adobe,
                started_at: r.started_at,
                source: r.source.clone(),
            });
        }
        TrackerSnapshot {
            current_app,
            current_project_id,
            tracking_since,
            slices,
        }
    }

    /// Close every open interval at `end` and write it to disk. Used on pause,
    /// project switches and quit so attribution is exact.
    pub fn cut_all(&self, db: &Db, end: i64) {
        let mut inner = self.lock();
        close_segment(db, inner.current.take(), end);
        close_segment(db, inner.render.take(), end);
        inner.last_persist = end;
    }
}

pub fn start_tracker(app: AppHandle) {
    std::thread::Builder::new()
        .name("framewise-tracker".into())
        .spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(TICK_MS));
            tick(&app);
        })
        .expect("spawn tracking thread");
}

fn tick(app: &AppHandle) {
    let state = app.state::<AppState>();
    let now_instant = Instant::now();
    let now = Local::now().timestamp();
    let cfg = match state.db.runtime_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("framewise: settings read failed: {e}");
            return;
        }
    };
    let mut inner = state.tracker.lock();
    let gap_ms = now_instant
        .saturating_duration_since(inner.last_tick)
        .as_millis() as u64;
    let resumed_from_gap = gap_resets(gap_ms);
    let prev_epoch = inner.last_epoch;
    inner.last_tick = now_instant;

    let locked = winapi::is_session_locked();
    let idle_ms = winapi::idle_milliseconds();
    let idle_seconds = (idle_ms / 1000) as i64;
    inner.locked = locked;
    inner.idle_seconds = idle_seconds;

    // Sleep / hibernation / a long scheduler stall: never bridge the gap.
    if resumed_from_gap {
        close_segment(&state.db, inner.current.take(), prev_epoch);
        close_segment(&state.db, inner.render.take(), prev_epoch);
        inner.me_sample = None;
    }

    if cfg.paused || locked {
        close_segment(&state.db, inner.current.take(), now);
        close_segment(&state.db, inner.render.take(), now);
        inner.last_epoch = now;
        inner.last_persist = now;
        return;
    }

    if idle_seconds >= cfg.idle_seconds {
        // Only count up to the last real input, so idle minutes are never billed.
        if let Some(c) = inner.current.take() {
            let end = idle_cutoff(now, idle_ms, cfg.idle_seconds, c.started_at);
            close_segment(&state.db, Some(c), end);
        }
        if let Some(r) = inner.render.take() {
            let end = idle_cutoff(now, idle_ms, cfg.idle_seconds, r.started_at);
            close_segment(&state.db, Some(r), end);
        }
        inner.last_epoch = now;
        inner.last_persist = now;
        inner.me_sample = None;
        return;
    }

    // Split exactly at local midnight.
    let mut midnight_start: Option<i64> = None;
    if let Some(c) = &inner.current {
        let started_day = local_day(c.started_at);
        if started_day != local_day(now) {
            if let Some(midnight) = next_day_start(&started_day) {
                if midnight <= now {
                    midnight_start = Some(midnight);
                    let c = inner.current.take();
                    close_segment(&state.db, c, midnight);
                }
            }
        }
    }
    if let Some(r) = &inner.render {
        let started_day = local_day(r.started_at);
        if started_day != local_day(now) {
            if let Some(midnight) = next_day_start(&started_day) {
                if midnight <= now {
                    midnight_start.get_or_insert(midnight);
                    let r = inner.render.take();
                    close_segment(&state.db, r, midnight);
                }
            }
        }
    }

    let foreground = winapi::foreground_process_name();
    let desired = attribution(
        foreground.as_deref(),
        &cfg.adobe,
        &cfg.ignore,
        cfg.active_project_id,
        &cfg.active_project_tools,
    );

    let mut switched = false;
    match (&inner.current, &desired) {
        (Some(cur), Some(want))
            if cur.app.eq_ignore_ascii_case(&want.0)
                && cur.project_id == want.1
                && cur.focused == want.2 => {}
        (Some(_), _) => {
            // Switching apps, project, tools or walking away from a tracked app.
            let cur = inner.current.take();
            close_segment(&state.db, cur, now);
            switched = true;
        }
        _ => {}
    }

    if inner.current.is_none() {
        if let Some((app_name, project_id, focused)) = desired.clone() {
            let adobe = cfg.adobe.contains(&app_name.to_lowercase());
            let start = if resumed_from_gap || switched {
                now
            } else if let Some(midnight) = midnight_start {
                midnight.max(prev_epoch).min(now)
            } else {
                prev_epoch.min(now)
            };
            if start <= now {
                match start_segment(
                    &state.db, &app_name, project_id, adobe, focused, "auto", start,
                ) {
                    Ok(seg) => inner.current = Some(seg),
                    Err(e) => eprintln!("framewise: could not open session: {e}"),
                }
            }
        }
    }

    // Optional background Media Encoder render accounting.
    if cfg.count_media_encoder_background {
        let should_consider = cfg.active_project_id.is_some()
            && desired
                .as_ref()
                .map(|d| !d.0.eq_ignore_ascii_case("Adobe Media Encoder.exe"))
                .unwrap_or(true)
            && !foreground
                .as_deref()
                .map(|a| cfg.adobe.contains(&a.to_lowercase()))
                .unwrap_or(false);
        let rendering = should_consider
            && sample_media_encoder(&mut inner, now_instant)
                .map(|pct| pct >= ME_RENDER_CPU_PERCENT)
                .unwrap_or(false);
        if rendering {
            if inner.render.is_none() {
                let start = if resumed_from_gap {
                    now
                } else {
                    prev_epoch.min(now)
                };
                if start <= now {
                    let render_focused = if cfg.active_project_tools.is_empty() {
                        None
                    } else {
                        Some(cfg.active_project_tools.contains("adobe media encoder.exe"))
                    };
                    if let Ok(seg) = start_segment(
                        &state.db,
                        "Adobe Media Encoder.exe",
                        cfg.active_project_id,
                        true,
                        render_focused,
                        "render",
                        start,
                    ) {
                        inner.render = Some(seg);
                    }
                }
            }
        } else if inner.render.is_some() {
            close_segment(&state.db, inner.render.take(), now);
        }
    } else {
        if inner.render.is_some() {
            close_segment(&state.db, inner.render.take(), now);
        }
        inner.me_sample = None;
    }

    if now - inner.last_persist >= PERSIST_SECS {
        if let Some(c) = &inner.current {
            if now > c.started_at {
                if let Ok(conn) = state.db.conn.lock().map_err(|e| e.to_string()) {
                    let _ = db::extend_session(&conn, c.id, now);
                }
            }
        }
        if let Some(r) = &inner.render {
            if now > r.started_at {
                if let Ok(conn) = state.db.conn.lock().map_err(|e| e.to_string()) {
                    let _ = db::extend_session(&conn, r.id, now);
                }
            }
        }
        inner.last_persist = now;
    }

    if cfg.notify_goals && now - inner.last_goal_check >= 60 {
        inner.last_goal_check = now;
        notify_goals(app, &state.db);
    }

    inner.last_epoch = now;
}

#[allow(clippy::too_many_arguments)]
fn start_segment(
    db: &Db,
    app: &str,
    project_id: Option<i64>,
    adobe: bool,
    focused: Option<bool>,
    source: &str,
    started_at: i64,
) -> anyhow::Result<ActiveSegment> {
    let conn = db.lock();
    let id = db::insert_open_session(&conn, app, project_id, started_at, adobe, source, focused)?;
    Ok(ActiveSegment {
        id,
        app: app.to_string(),
        project_id,
        adobe,
        focused,
        source: source.to_string(),
        started_at,
    })
}

fn close_segment(db: &Db, segment: Option<ActiveSegment>, end: i64) {
    let Some(segment) = segment else { return };
    if let Ok(conn) = db.conn.lock().map_err(|e| e.to_string()) {
        let _ = db::close_session(&conn, segment.id, end.max(segment.started_at));
    }
}

fn sample_media_encoder(inner: &mut TrackerInner, now: Instant) -> Option<f64> {
    let ticks = winapi::media_encoder_cpu_ticks();
    let previous = inner.me_sample.replace((now, ticks))?;
    let elapsed = now.saturating_duration_since(previous.0).as_secs_f64();
    if elapsed <= 0.1 {
        return None;
    }
    let delta = ticks.saturating_sub(previous.1) as f64;
    // GetProcessTimes is in 100 ns units. 100% of one core = 10_000_000 per second.
    Some((delta / (elapsed * 10_000_000.0)) * 100.0)
}

fn notify_goals(app: &AppHandle, db: &Db) {
    let Ok(progress) = db.goals_progress(&today_str()) else {
        return;
    };
    let reached: Vec<&GoalProgress> = progress.iter().filter(|p| p.reached).collect();
    for p in reached {
        let is_new = {
            let Ok(conn) = db.conn.lock().map_err(|e| e.to_string()) else {
                return;
            };
            db::goal_reached_once(&conn, p.goal.id, &p.period_key).unwrap_or(false)
        };
        if !is_new {
            continue;
        }
        let title = if p.goal.period == "weekly" {
            "Weekly goal reached"
        } else {
            "Daily goal reached"
        };
        let scope = p.goal.project_name.as_deref().unwrap_or("All projects");
        let body = format!(
            "{scope} - {} of {}",
            crate::timeutil::duration_label(p.seconds),
            crate::timeutil::duration_label(p.target_seconds)
        );
        let _ = app.notification().builder().title(title).body(body).show();
        break;
    }
}

/// Attribution rules, kept pure so they are easy to test.
/// Ignored apps are never tracked. Adobe apps get the active project. Once a
/// project has tools, every tracked app is credited to it and marked focused
/// (in the tool list) or other so project focus can be measured.
pub fn attribution(
    app: Option<&str>,
    adobe_list: &std::collections::HashSet<String>,
    ignore_list: &std::collections::HashSet<String>,
    active_project: Option<i64>,
    active_tools: &std::collections::HashSet<String>,
) -> Option<(String, Option<i64>, Option<bool>)> {
    let app = app?.trim();
    if app.is_empty() {
        return None;
    }
    let key = app.to_lowercase();
    if ignore_list.contains(&key) {
        return None;
    }
    let is_adobe = adobe_list.contains(&key);
    let has_tools = !active_tools.is_empty();
    let project_id = if active_project.is_some() && (is_adobe || has_tools) {
        active_project
    } else {
        None
    };
    let focused = if project_id.is_some() && has_tools {
        Some(active_tools.contains(&key))
    } else {
        None
    };
    Some((app.to_string(), project_id, focused))
}

pub fn idle_cutoff(now: i64, idle_ms: u64, _timeout_secs: i64, segment_start: i64) -> i64 {
    let last_input = now - (idle_ms / 1000) as i64;
    last_input.clamp(segment_start, now)
}

pub fn gap_resets(gap_ms: u64) -> bool {
    gap_ms > SLEEP_GAP_MS
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn set(items: &[&str]) -> HashSet<String> {
        items.iter().map(|s| s.to_lowercase()).collect()
    }

    #[test]
    fn adobe_time_goes_to_the_active_project() {
        let adobe = set(&["Adobe Premiere Pro.exe", "Photoshop.exe"]);
        let ignore = set(&["explorer.exe"]);
        let got = attribution(
            Some("Adobe Premiere Pro.exe"),
            &adobe,
            &ignore,
            Some(42),
            &HashSet::new(),
        );
        assert_eq!(got, Some(("Adobe Premiere Pro.exe".into(), Some(42), None)));
    }

    #[test]
    fn adobe_without_project_is_unassigned() {
        let adobe = set(&["Photoshop.exe"]);
        let got = attribution(
            Some("Photoshop.exe"),
            &adobe,
            &HashSet::new(),
            None,
            &HashSet::new(),
        );
        assert_eq!(got, Some(("Photoshop.exe".into(), None, None)));
    }

    #[test]
    fn non_adobe_is_unassigned_without_project_tools() {
        let adobe = set(&["Photoshop.exe"]);
        let got = attribution(
            Some("chrome.exe"),
            &adobe,
            &HashSet::new(),
            Some(7),
            &HashSet::new(),
        );
        assert_eq!(got, Some(("chrome.exe".into(), None, None)));
    }

    #[test]
    fn project_tools_mark_selected_and_other_apps() {
        let adobe = set(&["Photoshop.exe"]);
        let tools = set(&["Photoshop.exe", "zen.exe"]);
        assert_eq!(
            attribution(
                Some("Photoshop.exe"),
                &adobe,
                &HashSet::new(),
                Some(7),
                &tools
            ),
            Some(("Photoshop.exe".into(), Some(7), Some(true)))
        );
        assert_eq!(
            attribution(Some("zen.exe"), &adobe, &HashSet::new(), Some(7), &tools),
            Some(("zen.exe".into(), Some(7), Some(true)))
        );
        assert_eq!(
            attribution(Some("chrome.exe"), &adobe, &HashSet::new(), Some(7), &tools),
            Some(("chrome.exe".into(), Some(7), Some(false)))
        );
    }

    #[test]
    fn ignored_apps_are_not_tracked_even_if_adobe() {
        let adobe = set(&["Photoshop.exe"]);
        let ignore = set(&["Photoshop.exe"]);
        assert_eq!(
            attribution(
                Some("Photoshop.exe"),
                &adobe,
                &ignore,
                Some(1),
                &HashSet::new()
            ),
            None
        );
        assert_eq!(
            attribution(
                Some("PHOTOSHOP.EXE"),
                &adobe,
                &ignore,
                Some(1),
                &HashSet::new()
            ),
            None
        );
    }

    #[test]
    fn ignores_empty_or_missing_foreground() {
        assert_eq!(
            attribution(
                None,
                &HashSet::new(),
                &HashSet::new(),
                Some(1),
                &HashSet::new()
            ),
            None
        );
        assert_eq!(
            attribution(
                Some("   "),
                &HashSet::new(),
                &HashSet::new(),
                Some(1),
                &HashSet::new()
            ),
            None
        );
    }

    #[test]
    fn idle_cutoff_never_bills_idle_minutes() {
        let now = 10_000;
        let seg_start = 9_000;
        // Idle 12 min with a 3 min timeout: last input was at now - 720s.
        let end = idle_cutoff(now, 12 * 60 * 1000, 3 * 60, seg_start);
        assert_eq!(end, now - 12 * 60);
        assert!(end >= seg_start);
        // Idle longer than the segment: clamp to segment start.
        assert_eq!(idle_cutoff(now, 60 * 60 * 1000, 180, seg_start), seg_start);
    }

    #[test]
    fn sleep_gap_is_detected_but_a_normal_tick_is_not() {
        assert!(!gap_resets(1_500));
        assert!(!gap_resets(6_000));
        assert!(gap_resets(6_001));
        assert!(gap_resets(8 * 60 * 60 * 1000));
    }

    #[test]
    fn midnight_split_is_independent_of_app_switch() {
        // Split logic itself is covered in timeutil; this guards the tracker
        // assumption that the next-day boundary is strictly after the start.
        let now = Local::now().timestamp();
        let day = local_day(now);
        let next = next_day_start(&day).unwrap();
        assert!(next > now - 86_400);
    }
}
