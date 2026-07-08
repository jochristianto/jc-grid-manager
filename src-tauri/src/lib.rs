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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ⌃⌥←  — Control+Option+Left on macOS, Ctrl+Alt+Left on Windows.
    // (ALT maps to the Option key on macOS.)
    let left_half = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::ArrowLeft);
    let left_half_for_handler = left_half.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |_app, shortcut, event| {
                    if *shortcut == left_half_for_handler
                        && matches!(event.state(), ShortcutState::Pressed)
                    {
                        // TODO(next slice): move the focused window to the left half via the
                        // platform layer (macOS AXUIElement / Windows SetWindowPos).
                        println!("[jc-grid-manager] Left Half (⌃⌥←) pressed");
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![greet])
        .setup(move |app| {
            // Register the global shortcut(s).
            app.global_shortcut().register(left_half)?;

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
