use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

const TRAY_ID: &str = "framewise-tray";

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let state = app.state::<AppState>();
    let active = state.db.active_project_id();
    let paused = state.db.pause_flag();
    let projects = state.db.list_projects(false).unwrap_or_default();

    let open = MenuItem::with_id(app, "open", "Open Framewise", true, None::<&str>)?;
    let mini = MenuItem::with_id(app, "mini", "Mini timer", true, None::<&str>)?;
    let pause = CheckMenuItem::with_id(
        app,
        "pause",
        "Pause tracking",
        true,
        paused,
        None::<&str>,
    )?;
    let project_menu = Submenu::with_id(app, "projects", "Projects", true)?;
    let none = CheckMenuItem::with_id(
        app,
        "project:0",
        "No project (Unassigned)",
        true,
        active.is_none(),
        None::<&str>,
    )?;
    project_menu.append(&none)?;
    for project in &projects {
        let item = CheckMenuItem::with_id(
            app,
            format!("project:{}", project.id),
            &project.name,
            true,
            active == Some(project.id),
            None::<&str>,
        )?;
        project_menu.append(&item)?;
    }
    project_menu.append(&PredefinedMenuItem::separator(app)?)?;
    let new_project = MenuItem::with_id(
        app,
        "new-project",
        "New project...",
        true,
        None::<&str>,
    )?;
    let manage = MenuItem::with_id(
        app,
        "manage-projects",
        "Manage projects...",
        true,
        None::<&str>,
    )?;
    project_menu.append(&new_project)?;
    project_menu.append(&manage)?;

    let quit = MenuItem::with_id(app, "quit", "Quit Framewise", true, None::<&str>)?;
    Menu::with_items(
        app,
        &[
            &open,
            &mini,
            &pause,
            &PredefinedMenuItem::separator(app)?,
            &project_menu,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app)?;
    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("Framewise - offline project timer")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_menu(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                let _ = crate::show_main_window(tray.app_handle(), None);
            }
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } => {
                let _ = crate::show_mini_window(tray.app_handle(), Some(position));
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}

pub fn refresh(app: &AppHandle) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let menu = build_menu(app)?;
        tray.set_menu(Some(menu))?;
        let state = app.state::<AppState>();
        let paused = state.db.pause_flag();
        let active = state
            .db
            .runtime_config()
            .ok()
            .and_then(|c| c.active_project_name);
        let tooltip = match (paused, active) {
            (true, _) => "Framewise - paused".to_string(),
            (false, Some(name)) => format!("Framewise - {name}"),
            (false, None) => "Framewise - no project selected".to_string(),
        };
        tray.set_tooltip(Some(tooltip))?;
    }
    Ok(())
}

fn handle_menu(app: &AppHandle, id: &str) {
    let state = app.state::<AppState>();
    match id {
        "open" => {
            let _ = crate::show_main_window(app, None);
        }
        "mini" => {
            let _ = crate::show_mini_window(app, None);
        }
        "pause" => {
            let paused = !state.db.pause_flag();
            let _ = state
                .db
                .set_setting("paused", if paused { "true" } else { "false" });
            if paused {
                state.tracker.cut_all(&state.db, chrono::Local::now().timestamp());
            }
            let _ = refresh(app);
            let _ = app.emit("framewise:state-changed", ());
        }
        "quit" => {
            use std::sync::atomic::Ordering;
            state.quitting.store(true, Ordering::SeqCst);
            state.tracker.cut_all(&state.db, chrono::Local::now().timestamp());
            let _ = state.db.checkpoint();
            app.exit(0);
        }
        "manage-projects" => {
            let _ = crate::show_main_window(app, Some("projects"));
        }
        "new-project" => {
            let _ = crate::show_main_window(app, Some("projects:new"));
        }
        other if other.starts_with("project:") => {
            let value = other.trim_start_matches("project:");
            let project_id = value.parse::<i64>().ok().filter(|id| *id > 0);
            let _ = state.db.select_project(project_id);
            state.tracker.cut_all(&state.db, chrono::Local::now().timestamp());
            let _ = refresh(app);
            let _ = app.emit("framewise:state-changed", ());
        }
        _ => {}
    }
}
