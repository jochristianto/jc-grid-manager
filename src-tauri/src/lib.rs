mod core;
mod platform;
mod shortcuts;

use std::sync::{LazyLock, Mutex};

use crate::core::actions::Action;
use crate::core::state::SnapState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
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

/// Route a fired action. The four directional halves run the cycling snap (§7); every other
/// action logs a placeholder until its slice lands (geometry 007–019, soft beep 021).
fn dispatch(app: &tauri::AppHandle, action: Action) {
    // AppKit / Accessibility calls must run on the main thread.
    let _ = app.run_on_main_thread(move || {
        let mut state = SNAP_STATE.lock().unwrap();
        // Restore returns the focused window to its pre-snap baseline (§7); every other action
        // runs through the geometry table + state machine.
        if action == Action::Restore {
            match platform::restore(&mut state) {
                Ok(true) => {}
                Ok(false) => println!("[jc-grid-manager] nothing to restore"),
                Err(e) => eprintln!("[jc-grid-manager] restore — {e}"),
            }
            return;
        }
        match platform::perform(action, &mut state) {
            Ok(true) => {}
            Ok(false) => println!("[jc-grid-manager] {} not implemented", action.label()),
            Err(e) => eprintln!("[jc-grid-manager] {} — {e}", action.label()),
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // The default key scheme (idea.md §4), resolved to concrete shortcuts. One copy drives
    // the fired-shortcut → action lookup in the handler; the other registers them at startup.
    let registry = shortcuts::default_registry();
    let handler_registry = registry.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if !matches!(event.state(), ShortcutState::Pressed) {
                        return;
                    }
                    if let Some(&(_, action)) =
                        handler_registry.iter().find(|(s, _)| s == shortcut)
                    {
                        dispatch(app, action);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![greet])
        .setup(move |app| {
            // Register every default-bound shortcut. A bind the OS refuses is logged, not
            // fatal (full conflict validation is issue 020).
            for (shortcut, action) in &registry {
                if let Err(e) = app.global_shortcut().register(*shortcut) {
                    eprintln!(
                        "[jc-grid-manager] could not register {} ({shortcut:?}): {e}",
                        action.label()
                    );
                }
            }

            // Menu-bar / system-tray icon with a minimal menu (just Quit for now).
            let quit = MenuItem::with_id(app, "quit", "Quit JC Grid Manager", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;
            TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("JC Grid Manager")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
