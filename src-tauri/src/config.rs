//! Local, per-machine configuration + live shortcut rebinding (idea.md §2/§4/§5.4/§5.6/§7).
//!
//! Config is stored as a single JSON file in the OS app-config dir — no cloud sync (§3), and
//! hand-editable as a fallback (§9). User overrides layer on top of the issue-004 default binds:
//! the effective bind for an action is its override if present, else the default. Rebinding is
//! validated (reject duplicates and OS-refused combos) and applied **live** — the affected
//! global shortcut is re-registered without a restart.
//!
//! ## IPC contract (frontend, issues 028/029)
//!
//! Commands (Tauri `invoke`):
//! - `get_config() -> Config`
//! - `get_bindings() -> Vec<BindingInfo>`  — `{ action, label, bind, is_default }` per action
//! - `set_binding(action: String, bind: Bind) -> Result<(), BindingError>`
//! - `reset_binding(action: String) -> Result<(), BindingError>`
//! - `reset_all_bindings() -> Result<(), BindingError>`
//! - `set_tunable(key: String, value: f64) -> Result<(), BindingError>`
//!
//! Event: `bindings-changed` — emitted after any successful change so an open settings window
//! can refetch. `BindingError` is `{ code, message }`; codes: `unknown-action`, `unknown-tunable`,
//! `invalid-value`, `duplicate`, `os-refused`, `io`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::core::actions::Action;
use crate::core::geometry::Tunables;
use crate::shortcuts::{default_bind, Bind};

/// Config file name inside the app-config dir.
const CONFIG_FILE: &str = "config.json";
/// Schema version, persisted for future migrations (unused logic-wise for now).
const CONFIG_VERSION: u32 = 1;
/// Event fired after any successful config change.
const BINDINGS_CHANGED_EVENT: &str = "bindings-changed";

/// The persisted, per-machine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Schema version (for forward-compatible migrations).
    pub version: u32,
    /// Shortcut overrides keyed by [`Action::id`]. An entry replaces the 004 default; a missing
    /// entry means "use the default". Menu-only actions gain a shortcut by adding an entry here.
    pub bindings: BTreeMap<String, Bind>,
    /// User-tunable sizing factors (Almost Maximize / Smaller / Larger).
    pub tunables: Tunables,
    /// App bundle ids / names to leave alone. Consumed by issue 022 (not wired here).
    pub ignore_apps: Vec<String>,
    /// Launch at login. Consumed by issue 023 (not wired here).
    pub autostart: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: CONFIG_VERSION,
            bindings: BTreeMap::new(),
            tunables: Tunables::DEFAULT,
            ignore_apps: Vec::new(),
            autostart: false,
        }
    }
}

impl Config {
    /// The effective bind for `action`: the user override if present, else the 004 default.
    /// `None` means the action is unbound (a menu-only action with no override).
    pub fn effective_bind(&self, action: Action) -> Option<Bind> {
        self.bindings
            .get(action.id())
            .copied()
            .or_else(|| default_bind(action))
    }

    /// The effective `Shortcut → Action` registry: every bound action, override-or-default.
    pub fn registry(&self) -> Vec<(Shortcut, Action)> {
        Action::ALL
            .into_iter()
            .filter_map(|action| {
                self.effective_bind(action)
                    .map(|b| (b.to_shortcut(), action))
            })
            .collect()
    }

    /// Whether `action` is currently at its factory default (no override, or one equal to it).
    fn is_default(&self, action: Action) -> bool {
        self.effective_bind(action) == default_bind(action)
    }
}

/// If `bind`'s shortcut is already the effective binding of a **different** action, return that
/// action — the duplicate the validator rejects. Same-action rebinds don't conflict.
pub fn conflicting_action(config: &Config, action: Action, bind: Bind) -> Option<Action> {
    let shortcut = bind.to_shortcut();
    Action::ALL.into_iter().find(|&other| {
        other != action && config.effective_bind(other).map(|b| b.to_shortcut()) == Some(shortcut)
    })
}

/// Live config + its cached effective registry, held in Tauri state behind a `Mutex`. The
/// global-shortcut handler reads `registry` on each keypress; the commands below mutate both.
pub struct ConfigState {
    pub config: Config,
    pub registry: Vec<(Shortcut, Action)>,
}

impl ConfigState {
    pub fn new(config: Config) -> Self {
        let registry = config.registry();
        ConfigState { config, registry }
    }
}

/// A structured command error surfaced to the frontend as `{ code, message }`.
#[derive(Debug, Serialize)]
pub struct BindingError {
    pub code: String,
    pub message: String,
}

impl BindingError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        BindingError {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

/// One row of the shortcut table for the settings UI.
#[derive(Debug, Serialize)]
pub struct BindingInfo {
    /// Stable action id ([`Action::id`]).
    pub action: String,
    /// Human-readable label ([`Action::label`]).
    pub label: String,
    /// The effective bind, or `None` if the action is unbound.
    pub bind: Option<Bind>,
    /// Whether the effective bind is the factory default.
    pub is_default: bool,
}

/// Absolute path to the config file inside the OS app-config dir.
fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(dir.join(CONFIG_FILE))
}

/// Load config from disk. A missing file → defaults; an unparseable file → defaults (logged),
/// never a crash. Unknown keys are tolerated (serde ignores them), so older/newer files load.
pub fn load(app: &AppHandle) -> Config {
    let Ok(path) = config_path(app) else {
        return Config::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            eprintln!("[jc-grid-manager] config parse error ({e}); falling back to defaults");
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}

/// Persist config to disk, creating the app-config dir if needed. Pretty-printed for §9's
/// hand-edit fallback.
fn save(app: &AppHandle, config: &Config) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())
}

/// Resolve a stable action id or return the `unknown-action` error.
fn resolve_action(id: &str) -> Result<Action, BindingError> {
    Action::from_id(id)
        .ok_or_else(|| BindingError::new("unknown-action", format!("no action {id:?}")))
}

// ----- IPC commands --------------------------------------------------------------------------

/// The current persisted config.
#[tauri::command]
pub fn get_config(state: State<'_, Mutex<ConfigState>>) -> Config {
    state.lock().unwrap().config.clone()
}

/// Every action with its label + effective bind, for the settings table.
#[tauri::command]
pub fn get_bindings(state: State<'_, Mutex<ConfigState>>) -> Vec<BindingInfo> {
    let guard = state.lock().unwrap();
    Action::ALL
        .into_iter()
        .map(|action| BindingInfo {
            action: action.id().to_string(),
            label: action.label().to_string(),
            bind: guard.config.effective_bind(action),
            is_default: guard.config.is_default(action),
        })
        .collect()
}

/// Rebind `action` to `bind`: validate (duplicate + OS-refusal), apply live (re-register the
/// affected shortcut), persist, and emit `bindings-changed`. Rolls back the OS registration on
/// failure so a rejected bind leaves the previous one intact.
#[tauri::command]
pub fn set_binding(
    app: AppHandle,
    state: State<'_, Mutex<ConfigState>>,
    action: String,
    bind: Bind,
) -> Result<(), BindingError> {
    let action = resolve_action(&action)?;
    let mut guard = state.lock().unwrap();

    let new_shortcut = bind.to_shortcut();
    if let Some(other) = conflicting_action(&guard.config, action, bind) {
        return Err(BindingError::new(
            "duplicate",
            format!("that shortcut is already bound to {}", other.label()),
        ));
    }

    let old_shortcut = guard.config.effective_bind(action).map(|b| b.to_shortcut());
    if old_shortcut == Some(new_shortcut) {
        return Ok(()); // no change
    }

    // OS-refusal probe == the real registration: if the system rejects the chord, report it.
    app.global_shortcut().register(new_shortcut).map_err(|e| {
        BindingError::new(
            "os-refused",
            format!("the system refused this shortcut: {e}"),
        )
    })?;
    // Free the old chord only after the new one is safely registered.
    if let Some(old) = old_shortcut {
        let _ = app.global_shortcut().unregister(old);
    }

    guard.config.bindings.insert(action.id().to_string(), bind);
    guard.registry = guard.config.registry();
    save(&app, &guard.config).map_err(|e| BindingError::new("io", e))?;
    drop(guard);

    let _ = app.emit(BINDINGS_CHANGED_EVENT, ());
    Ok(())
}

/// Drop `action`'s override so it returns to the 004 default; re-register live, persist, emit.
#[tauri::command]
pub fn reset_binding(
    app: AppHandle,
    state: State<'_, Mutex<ConfigState>>,
    action: String,
) -> Result<(), BindingError> {
    let action = resolve_action(&action)?;
    let mut guard = state.lock().unwrap();

    let Some(old_bind) = guard.config.bindings.remove(action.id()) else {
        return Ok(()); // already at default
    };
    let old_shortcut = old_bind.to_shortcut();
    let default_shortcut = default_bind(action).map(|b| b.to_shortcut());

    if Some(old_shortcut) != default_shortcut {
        let _ = app.global_shortcut().unregister(old_shortcut);
        if let Some(shortcut) = default_shortcut {
            if let Err(e) = app.global_shortcut().register(shortcut) {
                // The default may collide with another action's override; leave it and warn.
                eprintln!(
                    "[jc-grid-manager] reset {}: could not register default ({shortcut:?}): {e}",
                    action.label()
                );
            }
        }
    }

    guard.registry = guard.config.registry();
    save(&app, &guard.config).map_err(|e| BindingError::new("io", e))?;
    drop(guard);

    let _ = app.emit(BINDINGS_CHANGED_EVENT, ());
    Ok(())
}

/// Clear every override, returning all shortcuts to their 004 defaults; re-register, persist, emit.
#[tauri::command]
pub fn reset_all_bindings(
    app: AppHandle,
    state: State<'_, Mutex<ConfigState>>,
) -> Result<(), BindingError> {
    let mut guard = state.lock().unwrap();
    guard.config.bindings.clear();

    let _ = app.global_shortcut().unregister_all();
    let registry = guard.config.registry();
    for (shortcut, action) in &registry {
        if let Err(e) = app.global_shortcut().register(*shortcut) {
            eprintln!(
                "[jc-grid-manager] reset all: could not register {} ({shortcut:?}): {e}",
                action.label()
            );
        }
    }
    guard.registry = registry;
    save(&app, &guard.config).map_err(|e| BindingError::new("io", e))?;
    drop(guard);

    let _ = app.emit(BINDINGS_CHANGED_EVENT, ());
    Ok(())
}

/// Update one sizing tunable (`almost_maximize_factor`, `resize_step`, or `min_size`), persist,
/// and emit `bindings-changed`. Values are fractions of the work area; out-of-range is rejected.
#[tauri::command]
pub fn set_tunable(
    app: AppHandle,
    state: State<'_, Mutex<ConfigState>>,
    key: String,
    value: f64,
) -> Result<(), BindingError> {
    if !value.is_finite() {
        return Err(BindingError::new(
            "invalid-value",
            "value must be a finite number",
        ));
    }
    let mut guard = state.lock().unwrap();
    match key.as_str() {
        // Fill fraction / floor: a fraction in (0, 1].
        "almost_maximize_factor" => {
            range(value, 0.0, 1.0, &key)?;
            guard.config.tunables.almost_maximize_factor = value;
        }
        "min_size" => {
            range(value, 0.0, 1.0, &key)?;
            guard.config.tunables.min_size = value;
        }
        // Per-press step: a smaller positive fraction.
        "resize_step" => {
            range(value, 0.0, 0.5, &key)?;
            guard.config.tunables.resize_step = value;
        }
        _ => {
            return Err(BindingError::new(
                "unknown-tunable",
                format!("no tunable {key:?}"),
            ));
        }
    }
    save(&app, &guard.config).map_err(|e| BindingError::new("io", e))?;
    drop(guard);

    let _ = app.emit(BINDINGS_CHANGED_EVENT, ());
    Ok(())
}

/// Reject a tunable value outside `(min, max]`.
fn range(value: f64, min: f64, max: f64, key: &str) -> Result<(), BindingError> {
    if value > min && value <= max {
        Ok(())
    } else {
        Err(BindingError::new(
            "invalid-value",
            format!("{key} must be in ({min}, {max}]"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri_plugin_global_shortcut::{Code, Modifiers};

    fn bind(code: Code) -> Bind {
        Bind {
            base_modifier: true,
            extra: Modifiers::empty(),
            code,
        }
    }

    #[test]
    fn effective_bind_falls_back_to_default() {
        let config = Config::default();
        // No overrides → the 004 default for a bound action.
        assert_eq!(
            config.effective_bind(Action::LeftHalf),
            default_bind(Action::LeftHalf)
        );
        // A menu-only action is unbound with no override.
        assert_eq!(config.effective_bind(Action::CenterHalf), None);
    }

    #[test]
    fn override_wins_over_default_including_menu_only() {
        let mut config = Config::default();
        // Rebind a defaulted action.
        config
            .bindings
            .insert(Action::LeftHalf.id().to_string(), bind(Code::KeyH));
        assert_eq!(
            config.effective_bind(Action::LeftHalf),
            Some(bind(Code::KeyH))
        );
        assert!(!config.is_default(Action::LeftHalf));
        // Bind a previously menu-only action.
        config
            .bindings
            .insert(Action::CenterHalf.id().to_string(), bind(Code::KeyM));
        assert_eq!(
            config.effective_bind(Action::CenterHalf),
            Some(bind(Code::KeyM))
        );
    }

    #[test]
    fn conflicting_action_detects_duplicates_but_ignores_self() {
        let config = Config::default();
        // LeftHalf's default is ⌃⌥←; binding RightHalf to it collides with LeftHalf.
        let left_bind = default_bind(Action::LeftHalf).unwrap();
        assert_eq!(
            conflicting_action(&config, Action::RightHalf, left_bind),
            Some(Action::LeftHalf)
        );
        // Re-affirming an action's own current bind is not a conflict.
        assert_eq!(
            conflicting_action(&config, Action::LeftHalf, left_bind),
            None
        );
        // A fresh, unused chord conflicts with nothing.
        assert_eq!(
            conflicting_action(&config, Action::CenterHalf, bind(Code::KeyZ)),
            None
        );
    }

    #[test]
    fn registry_reflects_overrides() {
        let mut config = Config::default();
        let base = config.registry().len();
        // Binding a menu-only action adds a registry entry.
        config
            .bindings
            .insert(Action::CenterHalf.id().to_string(), bind(Code::KeyM));
        assert_eq!(config.registry().len(), base + 1);
        assert!(config
            .registry()
            .iter()
            .any(|(sc, a)| *a == Action::CenterHalf && *sc == bind(Code::KeyM).to_shortcut()));
    }

    #[test]
    fn config_round_trips_and_tolerates_unknown_keys() {
        let mut config = Config::default();
        config
            .bindings
            .insert(Action::LeftHalf.id().to_string(), bind(Code::KeyH));
        config.tunables.almost_maximize_factor = 0.85;
        config.autostart = true;

        let json = serde_json::to_string(&config).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back.bindings, config.bindings);
        assert_eq!(back.tunables, config.tunables);
        assert!(back.autostart);

        // A file from a newer version with an unknown key still loads (forward-compatible).
        let with_extra = r#"{"version":1,"bindings":{},"future_flag":true}"#;
        let parsed: Config = serde_json::from_str(with_extra).unwrap();
        assert_eq!(parsed.tunables, Tunables::DEFAULT); // missing fields → defaults
    }
}
