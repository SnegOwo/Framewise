mod commands;
mod db;
mod export;
mod goals;
mod models;
mod portable;
mod state;
mod timeutil;
mod tracker;
mod tray;
mod winapi;

use std::sync::atomic::Ordering;

use tauri::{
    AppHandle, Emitter, Manager, RunEvent, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use tauri_plugin_autostart::ManagerExt;

use crate::db::Db;
use crate::state::AppState;

fn window_setting(app: &AppHandle, key: &str) -> Option<String> {
    app.try_state::<AppState>()
        .and_then(|state| state.db.setting(key))
}

fn window_number(app: &AppHandle, key: &str) -> Option<f64> {
    window_setting(app, key).and_then(|value| value.parse::<f64>().ok())
}

fn apply_saved_window_state(app: &AppHandle, window: &WebviewWindow) {
    if window_setting(app, "window_maximized").as_deref() == Some("true") {
        let _ = window.maximize();
        return;
    }
    if let (Some(width), Some(height)) = (
        window_number(app, "window_width"),
        window_number(app, "window_height"),
    ) {
        let _ = window.set_size(tauri::LogicalSize::new(width, height));
    }
    if let (Some(x), Some(y)) = (
        window_number(app, "window_x"),
        window_number(app, "window_y"),
    ) {
        let _ = window.set_position(tauri::LogicalPosition::new(x, y));
    }
    let _ = window.unminimize();
}

fn save_window_state(window: &tauri::Window) {
    let app = window.app_handle();
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let maximized = window.is_maximized().unwrap_or(false);
    let _ = state
        .db
        .set_setting("window_maximized", if maximized { "true" } else { "false" });
    if maximized {
        return;
    }
    let Ok(scale) = window.scale_factor() else {
        return;
    };
    if let Ok(size) = window.inner_size() {
        let _ = state.db.set_setting(
            "window_width",
            &(size.width as f64 / scale).round().to_string(),
        );
        let _ = state.db.set_setting(
            "window_height",
            &(size.height as f64 / scale).round().to_string(),
        );
    }
    if let Ok(position) = window.outer_position() {
        let _ = state
            .db
            .set_setting("window_x", &(position.x as f64 / scale).round().to_string());
        let _ = state
            .db
            .set_setting("window_y", &(position.y as f64 / scale).round().to_string());
    }
}

pub fn show_main_window(app: &AppHandle, page: Option<&str>) -> tauri::Result<()> {
    if let Some(mini) = app.get_webview_window("mini") {
        let _ = mini.close();
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        if let Some(page) = page {
            let _ = app.emit("framewise:navigate", page);
        }
        return Ok(());
    }

    let mut url = "index.html".to_string();
    if let Some(page) = page {
        if let Some(project_page) = page.strip_prefix("projects:new") {
            let _ = project_page;
            url = "index.html?page=projects&new=1".to_string();
        } else {
            url = format!("index.html?page={page}");
        }
    }

    let width = window_number(app, "window_width").unwrap_or(1200.0);
    let height = window_number(app, "window_height").unwrap_or(840.0);
    let maximized = window_setting(app, "window_maximized").as_deref() == Some("true");

    let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::App(url.into()))
        .title("Framewise")
        .inner_size(width, height)
        .min_inner_size(760.0, 600.0);

    if let (Some(x), Some(y)) = (
        window_number(app, "window_x"),
        window_number(app, "window_y"),
    ) {
        builder = builder.position(x, y);
    } else {
        builder = builder.center();
    }

    let window = builder.build()?;
    if maximized {
        let _ = window.maximize();
    }
    let _ = window.set_focus();
    Ok(())
}

/// Tiny always-on-top timer opened by a single tray click.
pub fn show_mini_window(
    app: &AppHandle,
    anchor: Option<tauri::PhysicalPosition<f64>>,
) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("mini") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let mut builder = WebviewWindowBuilder::new(app, "mini", WebviewUrl::App("index.html".into()))
        .title("Framewise")
        .inner_size(360.0, 132.0)
        .min_inner_size(150.0, 64.0)
        .max_inner_size(640.0, 420.0)
        .resizable(true)
        .maximizable(false)
        .minimizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true);

    // Place the window before it is created so it never flashes at the default
    // position before jumping next to the tray.
    if let Some(monitor) = app.primary_monitor()? {
        let scale = monitor.scale_factor();
        let size = monitor.size();
        let origin = monitor.position();
        let width = 360.0 * scale;
        let height = 132.0 * scale;

        let (x, y) = if let Some(point) = anchor {
            (
                point.x - width + 24.0 * scale,
                point.y - height - 14.0 * scale,
            )
        } else {
            (
                origin.x as f64 + size.width as f64 - width - 12.0 * scale,
                origin.y as f64 + size.height as f64 - height - 56.0 * scale,
            )
        };

        let min_x = origin.x as f64 + 8.0;
        let min_y = origin.y as f64 + 8.0;
        let max_x = origin.x as f64 + size.width as f64 - width - 8.0;
        let max_y = origin.y as f64 + size.height as f64 - height - 8.0;

        builder = builder.position(
            x.clamp(min_x, max_x.max(min_x)) / scale,
            y.clamp(min_y, max_y.max(min_y)) / scale,
        );
    } else {
        builder = builder.center();
    }

    let window = builder.build()?;
    let _ = window.set_focus();
    Ok(())
}

pub fn run() {
    portable::prepare_webview_data();
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_main_window(app, None);
            let _ = app.emit("framewise:state-changed", ());
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let data_dir = portable::data_dir(app.path().app_data_dir()?);
            std::fs::create_dir_all(&data_dir)?;
            let db = Db::open(data_dir.join("framewise.sqlite3"))?;
            let launch_at_login = db.setting("launch_at_login").as_deref() == Some("true");
            app.manage(AppState::new(db));
            tray::create(app.handle())?;
            if launch_at_login && !portable::is_portable() {
                let _ = app.autolaunch().enable();
            }
            tracker::start_tracker(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    save_window_state(window);
                }
            }
            if window.label() == "mini" {
                if let tauri::WindowEvent::Focused(false) = event {
                    let state = window.state::<AppState>();
                    if !state.mini_pinned.load(Ordering::Relaxed) {
                        let _ = window.close();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_bootstrap,
            commands::get_status,
            commands::list_projects,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            commands::select_project,
            commands::set_paused,
            commands::get_settings,
            commands::update_settings,
            commands::replace_app_list,
            commands::list_goals,
            commands::create_goal,
            commands::update_goal,
            commands::delete_goal,
            commands::get_report,
            commands::get_heatmap,
            commands::get_sessions,
            commands::save_session,
            commands::delete_session,
            commands::reassign_sessions,
            commands::get_project_detail,
            commands::export_data,
            commands::backup_database,
            commands::import_database,
            commands::open_data_folder,
            commands::open_full_app,
            commands::app_details,
            commands::open_mini_window,
            commands::set_mini_pinned,
            commands::close_mini_window,
            commands::quit_app,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Framewise");

    builder.run(|app_handle, event| match event {
        RunEvent::Ready => {
            if let Some(window) = app_handle.get_webview_window("main") {
                apply_saved_window_state(app_handle, &window);
            }
        }
        RunEvent::ExitRequested { api, code, .. } => {
            let quitting = app_handle
                .state::<AppState>()
                .quitting
                .load(Ordering::SeqCst);
            if !quitting && code.is_none() {
                api.prevent_exit();
            }
        }
        RunEvent::Exit => {
            let state = app_handle.state::<AppState>();
            state
                .tracker
                .cut_all(&state.db, chrono::Local::now().timestamp());
            let _ = state.db.checkpoint();
        }
        _ => {}
    });
}
