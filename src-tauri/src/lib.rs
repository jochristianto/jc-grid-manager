mod core;
mod platform;

use crate::core::actions::Half;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// A shortcut with the app's base modifier (Control+Option on macOS / Ctrl+Alt on Windows).
fn ctrl_alt(code: Code) -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), code)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let left = ctrl_alt(Code::ArrowLeft);
    let right = ctrl_alt(Code::ArrowRight);
    let up = ctrl_alt(Code::ArrowUp);
    let down = ctrl_alt(Code::ArrowDown);

    // Clones for the shortcut handler; the originals are registered in `setup`.
    let (h_left, h_right, h_up, h_down) = (left.clone(), right.clone(), up.clone(), down.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if !matches!(event.state(), ShortcutState::Pressed) {
                        return;
                    }
                    let half = if *shortcut == h_left {
                        Half::Left
                    } else if *shortcut == h_right {
                        Half::Right
                    } else if *shortcut == h_up {
                        Half::Top
                    } else if *shortcut == h_down {
                        Half::Bottom
                    } else {
                        return;
                    };
                    // AppKit / Accessibility calls must run on the main thread.
                    let app = app.clone();
                    let _ = app.run_on_main_thread(move || match platform::snap(half) {
                        Ok(()) => println!("[jc-grid-manager] snapped {half:?}"),
                        Err(e) => eprintln!("[jc-grid-manager] {half:?} — {e}"),
                    });
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![greet])
        .setup(move |app| {
            // Register the half-snapping shortcuts. A conflicting bind is logged, not fatal.
            for shortcut in [left, right, up, down] {
                if let Err(e) = app.global_shortcut().register(shortcut) {
                    eprintln!("[jc-grid-manager] could not register a shortcut: {e}");
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
