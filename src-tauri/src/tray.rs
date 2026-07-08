//! System-tray menu (idea.md §4 / §5.5) — built in Rust, not the frontend.
//!
//! The full §4 command set is reachable here without a shortcut, including the menu-only actions
//! (Center Half, Move to Edge, Fourths, Sixths). Every leaf carries its [`Action::id`] as the menu
//! id and dispatches through the shared [`crate::dispatch`] — the same path as the global
//! shortcuts — so nothing reimplements placement logic. The dynamic **Ignore "[App]"** item
//! (issue 022) refreshes its label + checkmark to the frontmost app when the cursor enters the
//! tray icon (there is no cross-platform "menu about to open" event; hover precedes the click that
//! opens the menu). Shortcut hints are the effective binds at build time (idea.md §4).

use std::sync::Mutex;

use tauri::menu::{
    AboutMetadata, CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu,
};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::config::{self, Config, ConfigState};
use crate::core::actions::Action;

/// Menu id of the dynamic "Ignore [App]" item; the rest of the leaves use [`Action::id`].
const IGNORE_ID: &str = "ignore-current-app";
const SETTINGS_ID: &str = "settings";
const QUIT_ID: &str = "quit";

/// Build the full §4 tray menu and install it, wiring menu clicks and the hover refresh of the
/// Ignore item. Reads the current config for shortcut hints, so it runs after the config state is
/// managed.
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let config = {
        let state = app.state::<Mutex<ConfigState>>();
        let guard = state.lock().unwrap();
        guard.config.clone()
    };

    use Action::*;
    let halves = group(
        app,
        "Halves",
        &[LeftHalf, RightHalf, TopHalf, BottomHalf, CenterHalf],
        &config,
    )?;
    let corners = group(
        app,
        "Corners",
        &[TopLeft, TopRight, BottomLeft, BottomRight],
        &config,
    )?;
    let thirds = group(
        app,
        "Thirds",
        &[FirstThird, CenterThird, LastThird, FirstTwoThirds, LastTwoThirds],
        &config,
    )?;
    let sizing = group(
        app,
        "Sizing",
        &[Maximize, AlmostMaximize, MaximizeHeight, Smaller, Larger, Center, Restore],
        &config,
    )?;
    let displays = group(app, "Displays", &[NextDisplay, PreviousDisplay], &config)?;
    let move_edge = group(
        app,
        "Move to Edge",
        &[MoveLeft, MoveRight, MoveUp, MoveDown],
        &config,
    )?;
    let fourths = group(
        app,
        "Fourths",
        &[FirstFourth, SecondFourth, ThirdFourth, LastFourth],
        &config,
    )?;
    let sixths = group(
        app,
        "Sixths",
        &[
            SixthTopLeft,
            SixthTopCenter,
            SixthTopRight,
            SixthBottomLeft,
            SixthBottomCenter,
            SixthBottomRight,
        ],
        &config,
    )?;

    // Dynamic "Ignore [App]" (022): generic until the first hover fills in the frontmost app.
    let ignore =
        CheckMenuItem::with_id(app, IGNORE_ID, "Ignore Current App", true, false, None::<&str>)?;
    let settings = MenuItem::with_id(app, SETTINGS_ID, "Settings…", true, None::<&str>)?;
    // Native About panel (name + version from the bundle), no frontend needed. No "Check for
    // Updates…" item — out of scope for v1 (§4/§9).
    let pkg = app.package_info();
    let about = PredefinedMenuItem::about(
        app,
        Some("About JC Grid Manager"),
        Some(AboutMetadata {
            name: Some(pkg.name.clone()),
            version: Some(pkg.version.to_string()),
            ..Default::default()
        }),
    )?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit JC Grid Manager", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &halves, &corners, &thirds, &sizing, &displays, &move_edge, &fourths, &sixths, &sep1,
            &ignore, &sep2, &settings, &about, &sep3, &quit,
        ],
    )?;

    let ignore_for_menu = ignore.clone();
    let ignore_for_hover = ignore.clone();
    let app_for_hover = app.clone();

    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("JC Grid Manager")
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            QUIT_ID => app.exit(0),
            SETTINGS_ID => open_main_window(app),
            IGNORE_ID => match config::toggle_ignore_current_app(app.clone(), app.state()) {
                Ok(status) => apply_ignore_status(&ignore_for_menu, &status.name, status.ignored),
                Err(e) => eprintln!("[jc-grid-manager] toggle ignore — {}", e.message),
            },
            // Every other id is an action leaf → same dispatch path as the shortcuts.
            other => {
                if let Some(action) = Action::from_id(other) {
                    crate::dispatch(app, action);
                }
            }
        })
        .on_tray_icon_event(move |_tray, event| {
            // Refresh the Ignore label just before the menu opens (Enter precedes the click).
            if let TrayIconEvent::Enter { .. } = event {
                refresh_ignore_label(&app_for_hover, &ignore_for_hover);
            }
        })
        .build(app)?;

    Ok(())
}

/// One action leaf: the menu id is the action's stable id (so `on_menu_event` can dispatch it),
/// and the label carries the effective shortcut as a trailing hint when one is bound.
fn action_item(app: &AppHandle, action: Action, config: &Config) -> tauri::Result<MenuItem<Wry>> {
    let text = match config.effective_bind(action) {
        Some(bind) => format!("{}   {}", action.label(), bind.hint()),
        None => action.label().to_string(),
    };
    MenuItem::with_id(app, action.id(), text, true, None::<&str>)
}

/// A titled submenu of action leaves.
fn group(
    app: &AppHandle,
    title: &str,
    actions: &[Action],
    config: &Config,
) -> tauri::Result<Submenu<Wry>> {
    let items = actions
        .iter()
        .map(|&a| action_item(app, a, config))
        .collect::<tauri::Result<Vec<_>>>()?;
    let refs: Vec<&dyn IsMenuItem<Wry>> = items.iter().map(|i| i as &dyn IsMenuItem<Wry>).collect();
    Submenu::with_items(app, title, true, &refs)
}

/// Set the Ignore item's label to `Ignore "[name]"` and its checkmark to `ignored`.
fn apply_ignore_status(item: &CheckMenuItem<Wry>, name: &str, ignored: bool) {
    let _ = item.set_text(format!("Ignore \"{name}\""));
    let _ = item.set_checked(ignored);
}

/// Refresh the Ignore item from the current frontmost app (022). If there's no focused window or
/// Accessibility isn't granted yet, fall back to a generic, unchecked label.
fn refresh_ignore_label(app: &AppHandle, item: &CheckMenuItem<Wry>) {
    match config::get_frontmost_app(app.state()) {
        Ok(status) => apply_ignore_status(item, &status.name, status.ignored),
        Err(_) => {
            let _ = item.set_text("Ignore Current App");
            let _ = item.set_checked(false);
        }
    }
}

/// Show + focus the main window. Pre-028 stand-in for opening the Settings window.
fn open_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
