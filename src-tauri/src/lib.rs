mod core;
mod platform;
mod shortcuts;

use crate::core::actions::Action;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Route a fired action to its behavior. For now only the four directional halves run real
/// geometry; every other action logs a placeholder (a soft beep arrives in issue 021, the
/// remaining geometry in issues 005–019).
fn dispatch(app: &tauri::AppHandle, action: Action) {
    match action.as_half() {
        Some(half) => {
            // AppKit / Accessibility calls must run on the main thread.
            let app = app.clone();
            let _ = app.run_on_main_thread(move || match platform::snap(half) {
                Ok(()) => println!("[jc-grid-manager] snapped {half:?}"),
                Err(e) => eprintln!("[jc-grid-manager] {half:?} — {e}"),
            });
        }
        None => println!("[jc-grid-manager] {} not implemented", action.label()),
    }
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
