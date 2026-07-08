mod config;
mod core;
mod platform;
mod shortcuts;
mod tray;

use std::sync::{LazyLock, Mutex};

use crate::config::ConfigState;
use crate::core::actions::Action;
use crate::core::state::SnapState;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// The one snap state machine (idea.md §7). It is global + mutable because a repeated shortcut
/// must see the previous snap. Guarded by a `Mutex`; today it is only ever touched from the
/// main thread (snaps run there), so contention is nil.
static SNAP_STATE: LazyLock<Mutex<SnapState>> = LazyLock::new(|| Mutex::new(SnapState::new()));

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Route a fired action through the §7 state machine. Restore returns to the pre-snap baseline;
/// every other action runs through the geometry table with the live user tunables. Shared by the
/// global-shortcut handler and the tray menu (issue 024).
pub(crate) fn dispatch(app: &tauri::AppHandle, action: Action) {
    // Snapshot the live config the action needs — sizing tunables (may have changed via
    // `set_tunable`) and the ignore list (§4) — so we don't hold the config lock across window I/O.
    let (tunables, ignore_apps) = {
        let state = app.state::<Mutex<ConfigState>>();
        let guard = state.lock().unwrap();
        (guard.config.tunables, guard.config.ignore_apps.clone())
    };
    // AppKit / Accessibility calls must run on the main thread.
    let _ = app.run_on_main_thread(move || {
        // Ignore-app list (§4): if the frontmost app is ignored, skip silently (the user opted
        // in). Only resolve identity when something is actually ignored — that avoids an extra
        // Accessibility round-trip on every keypress in the common (empty-list) case.
        if !ignore_apps.is_empty() {
            if let Ok(identity) = platform::focused_app_identity() {
                if identity.is_ignored(&ignore_apps) {
                    println!(
                        "[jc-grid-manager] {} — {} is ignored, skipping",
                        action.label(),
                        identity.display_label()
                    );
                    return;
                }
            }
        }

        let mut state = SNAP_STATE.lock().unwrap();
        if action == Action::Restore {
            match platform::restore(&mut state) {
                Ok(true) => {}
                Ok(false) => {
                    println!("[jc-grid-manager] nothing to restore");
                    platform::notify_no_op();
                }
                Err(e) => eprintln!("[jc-grid-manager] restore — {e}"),
            }
            return;
        }
        match platform::perform(action, &mut state, tunables) {
            Ok(true) => {}
            Ok(false) => {
                println!("[jc-grid-manager] {} — nothing to do", action.label());
                platform::notify_no_op();
            }
            Err(e) => eprintln!("[jc-grid-manager] {} — {e}", action.label()),
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Launch at login (idea.md §8) — off by default; toggled via get/set_autostart. The OS
        // login item is the source of truth; config (020) mirrors it for the settings UI.
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if !matches!(event.state(), ShortcutState::Pressed) {
                        return;
                    }
                    // Resolve against the live effective registry (rebinding may have changed it).
                    let action = {
                        let state = app.state::<Mutex<ConfigState>>();
                        let guard = state.lock().unwrap();
                        guard
                            .registry
                            .iter()
                            .find(|(s, _)| s == shortcut)
                            .map(|(_, action)| *action)
                    };
                    if let Some(action) = action {
                        dispatch(app, action);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            greet,
            config::get_config,
            config::get_bindings,
            config::set_binding,
            config::reset_binding,
            config::reset_all_bindings,
            config::set_tunable,
            config::get_autostart,
            config::set_autostart,
            config::get_frontmost_app,
            config::toggle_ignore_current_app,
            config::get_accessibility_state,
            config::prompt_accessibility,
            config::open_accessibility_settings
        ])
        .setup(|app| {
            // Load the persisted per-machine config (or defaults on first run) and register the
            // effective shortcuts — user overrides layered over the §4 defaults (issue 020). A
            // bind the OS refuses is logged, not fatal.
            let state = ConfigState::new(config::load(app.handle()));
            for (shortcut, action) in &state.registry {
                if let Err(e) = app.global_shortcut().register(*shortcut) {
                    eprintln!(
                        "[jc-grid-manager] could not register {} ({shortcut:?}): {e}",
                        action.label()
                    );
                }
            }
            app.manage(Mutex::new(state));

            // Menu-bar / system-tray icon with the full §4 menu (issue 024).
            tray::setup(app.handle())?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
