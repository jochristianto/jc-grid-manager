//! Global-shortcut bindings: the default key scheme and its resolution to plugin shortcuts.
//!
//! [`Bind`] is a platform-neutral description of a chord — the base modifier (Control+Option
//! on macOS / Ctrl+Alt on Windows), optional extra modifiers, and a key. [`default_bind`]
//! holds the idea.md §4 default scheme; [`default_registry`] resolves it to concrete
//! [`Shortcut`]s paired with their [`Action`]. Config persistence, rebinding, and conflict
//! validation arrive in issue 020; [`Bind`] is kept small and serialization-friendly for it.

use serde::{Deserialize, Serialize};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

use crate::core::actions::Action;

/// A platform-neutral key binding. `base_modifier` is the app's base chord (Control+Option /
/// Ctrl+Alt); `extra` adds Shift and/or Super (⌘ on macOS, ⊞ Win on Windows); `code` is the key.
///
/// Serialized (config file + IPC, issue 020) as a stable, hand-editable shape rather than the
/// plugin's internal enums: `{ "base_modifier": true, "shift": false, "super": false,
/// "code": "ArrowLeft" }`. `code` uses the W3C UI Events key-code names (`Code`'s `Display` /
/// `FromStr`), so the JSON survives plugin upgrades and a person can edit it (idea.md §9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "BindRepr", try_from = "BindRepr")]
pub struct Bind {
    pub base_modifier: bool,
    pub extra: Modifiers,
    pub code: Code,
}

/// The serialized shape of a [`Bind`] (see its docs). `extra` is split into explicit `shift` /
/// `super` flags so the JSON is self-describing and forward-compatible.
#[derive(Serialize, Deserialize)]
struct BindRepr {
    base_modifier: bool,
    #[serde(default)]
    shift: bool,
    #[serde(default, rename = "super")]
    super_key: bool,
    code: String,
}

impl From<Bind> for BindRepr {
    fn from(bind: Bind) -> Self {
        BindRepr {
            base_modifier: bind.base_modifier,
            shift: bind.extra.contains(Modifiers::SHIFT),
            super_key: bind.extra.contains(Modifiers::SUPER),
            code: bind.code.to_string(),
        }
    }
}

impl TryFrom<BindRepr> for Bind {
    type Error = String;

    fn try_from(repr: BindRepr) -> Result<Self, Self::Error> {
        let code = repr
            .code
            .parse::<Code>()
            .map_err(|_| format!("unrecognized key code {:?}", repr.code))?;
        let mut extra = Modifiers::empty();
        if repr.shift {
            extra |= Modifiers::SHIFT;
        }
        if repr.super_key {
            extra |= Modifiers::SUPER;
        }
        Ok(Bind {
            base_modifier: repr.base_modifier,
            extra,
            code,
        })
    }
}

impl Bind {
    /// The base modifier + `code`.
    fn base(code: Code) -> Bind {
        Bind {
            base_modifier: true,
            extra: Modifiers::empty(),
            code,
        }
    }

    /// The base modifier + `extra` modifiers + `code`.
    fn base_with(extra: Modifiers, code: Code) -> Bind {
        Bind {
            base_modifier: true,
            extra,
            code,
        }
    }

    /// Resolve to a concrete plugin [`Shortcut`] for the current OS. The base modifier is
    /// Control+Alt on every platform (Option ≡ Alt); ⌘/Win is `Super` (`extra`). The plugin
    /// normalizes `Meta → Super` and maps `Super` to Command on macOS.
    pub fn to_shortcut(self) -> Shortcut {
        let mut mods = self.extra;
        if self.base_modifier {
            mods |= Modifiers::CONTROL | Modifiers::ALT;
        }
        Shortcut::new(Some(mods), self.code)
    }
}

/// The idea.md §4 default binding for `action`, or `None` for menu-only actions.
pub fn default_bind(action: Action) -> Option<Bind> {
    use Action::*;
    match action {
        // Halves — cycle ½→⅔→⅓ (state machine, issue 005)
        LeftHalf => Some(Bind::base(Code::ArrowLeft)),
        RightHalf => Some(Bind::base(Code::ArrowRight)),
        TopHalf => Some(Bind::base(Code::ArrowUp)),
        BottomHalf => Some(Bind::base(Code::ArrowDown)),
        // Corners
        TopLeft => Some(Bind::base(Code::KeyU)),
        TopRight => Some(Bind::base(Code::KeyI)),
        BottomLeft => Some(Bind::base(Code::KeyJ)),
        BottomRight => Some(Bind::base(Code::KeyK)),
        // Thirds
        FirstThird => Some(Bind::base(Code::KeyD)),
        CenterThird => Some(Bind::base(Code::KeyF)),
        LastThird => Some(Bind::base(Code::KeyG)),
        FirstTwoThirds => Some(Bind::base(Code::KeyE)),
        LastTwoThirds => Some(Bind::base(Code::KeyT)),
        // Sizing
        Maximize => Some(Bind::base(Code::Enter)),
        MaximizeHeight => Some(Bind::base_with(Modifiers::SHIFT, Code::ArrowUp)),
        Smaller => Some(Bind::base(Code::Minus)),
        Larger => Some(Bind::base(Code::Equal)),
        Center => Some(Bind::base(Code::KeyC)),
        Restore => Some(Bind::base(Code::Backspace)),
        // Displays — base + ⌘/Win
        NextDisplay => Some(Bind::base_with(Modifiers::SUPER, Code::ArrowRight)),
        PreviousDisplay => Some(Bind::base_with(Modifiers::SUPER, Code::ArrowLeft)),
        // Menu-only (no default shortcut); the user can bind these later (issue 020).
        CenterHalf | AlmostMaximize | MoveLeft | MoveRight | MoveUp | MoveDown | FirstFourth
        | SecondFourth | ThirdFourth | LastFourth | SixthTopLeft | SixthTopCenter
        | SixthTopRight | SixthBottomLeft | SixthBottomCenter | SixthBottomRight => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default scheme resolved to concrete shortcuts, each paired with its action (menu-only
    /// actions omitted). At runtime the effective registry ([`crate::config::Config::registry`])
    /// drives registration; this stays as the pinned, collision-free default scheme.
    fn default_registry() -> Vec<(Shortcut, Action)> {
        Action::ALL
            .into_iter()
            .filter_map(|action| default_bind(action).map(|bind| (bind.to_shortcut(), action)))
            .collect()
    }

    fn shortcut_of(action: Action) -> Shortcut {
        default_bind(action)
            .expect("action should have a default bind")
            .to_shortcut()
    }

    #[test]
    fn base_modifier_is_control_alt() {
        // ⌃⌥←
        assert_eq!(
            shortcut_of(Action::LeftHalf),
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::ArrowLeft)
        );
    }

    #[test]
    fn maximize_is_control_alt_enter() {
        // ⌃⌥↩
        assert_eq!(
            shortcut_of(Action::Maximize),
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Enter)
        );
    }

    #[test]
    fn maximize_height_adds_shift() {
        // ⌃⌥⇧↑
        assert_eq!(
            shortcut_of(Action::MaximizeHeight),
            Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT),
                Code::ArrowUp
            )
        );
    }

    #[test]
    fn display_move_adds_super() {
        // ⌃⌥⌘→
        assert_eq!(
            shortcut_of(Action::NextDisplay),
            Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER),
                Code::ArrowRight
            )
        );
    }

    #[test]
    fn menu_only_actions_have_no_default() {
        assert_eq!(default_bind(Action::CenterHalf), None);
        assert_eq!(default_bind(Action::AlmostMaximize), None);
        assert_eq!(default_bind(Action::FirstFourth), None);
        assert_eq!(default_bind(Action::SixthTopLeft), None);
    }

    #[test]
    fn bind_serializes_to_stable_hand_editable_json() {
        let bind = default_bind(Action::NextDisplay).unwrap(); // base + Super + ArrowRight
        let json = serde_json::to_value(bind).unwrap();
        assert_eq!(json["base_modifier"], true);
        assert_eq!(json["super"], true);
        assert_eq!(json["shift"], false);
        assert_eq!(json["code"], "ArrowRight");
    }

    #[test]
    fn bind_round_trips_through_json() {
        for action in Action::ALL {
            if let Some(bind) = default_bind(action) {
                let json = serde_json::to_string(&bind).unwrap();
                let back: Bind = serde_json::from_str(&json).unwrap();
                assert_eq!(bind, back, "round-trip mismatch for {action:?}");
            }
        }
    }

    #[test]
    fn bind_deserialize_rejects_unknown_code() {
        let err = serde_json::from_str::<Bind>(r#"{"base_modifier":true,"code":"NopeKey"}"#);
        assert!(err.is_err(), "unknown code should fail to parse");
    }

    #[test]
    fn registry_covers_bound_actions_with_no_chord_collisions() {
        let registry = default_registry();
        let bound = Action::ALL
            .into_iter()
            .filter(|a| default_bind(*a).is_some())
            .count();
        assert_eq!(registry.len(), bound);
        // Two actions resolving to the same chord would make dispatch ambiguous.
        for i in 0..registry.len() {
            for j in (i + 1)..registry.len() {
                assert_ne!(
                    registry[i].0, registry[j].0,
                    "chord collision: {:?} vs {:?}",
                    registry[i].1, registry[j].1
                );
            }
        }
    }
}
