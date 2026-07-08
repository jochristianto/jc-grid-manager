//! Platform window I/O.
//!
//! The [`Platform`] trait is the thin per-OS shim from idea.md §5.3: a handful of dumb I/O
//! methods with no placement logic. Everything smart lives in [`crate::core`]. macOS
//! implements the trait via the Accessibility API; Windows arrives in issue 025.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::core::actions::Action;
use crate::core::geometry::{Rect, Tunables};
use crate::core::state::SnapState;

#[cfg(target_os = "macos")]
mod macos;

/// Identity of the application that owns a window — the ignore-app list keys on this (idea.md §4).
#[derive(Debug, Clone, Default)]
pub struct WindowIdentity {
    pub bundle_id: Option<String>,
    pub name: Option<String>,
}

impl WindowIdentity {
    /// Stable key for the ignore list: the bundle id when the OS reports one, else the display
    /// name. `None` for a fully anonymous process — nothing to key on, so it can't be ignored.
    pub fn key(&self) -> Option<String> {
        self.bundle_id.clone().or_else(|| self.name.clone())
    }

    /// Whether this app is present in the ignore list `list` (compared by [`Self::key`]).
    pub fn is_ignored(&self, list: &[String]) -> bool {
        self.key().is_some_and(|k| list.contains(&k))
    }

    /// Human label for the tray "Ignore [App]" item — the display name, falling back to the
    /// bundle id, then a generic phrase.
    pub fn display_label(&self) -> String {
        self.name
            .clone()
            .or_else(|| self.bundle_id.clone())
            .unwrap_or_else(|| "this app".to_string())
    }
}

/// The per-OS I/O shim: raw window reads/writes only, no placement logic (idea.md §5.3).
///
/// `identity` backs the ignore-app list (022); `frame` / `displays` came online with display
/// selection (003). The whole trait is declared up front so the Windows shim (025) can implement
/// it in one pass.
pub trait Platform {
    /// Opaque handle to a native window.
    type Window;

    /// The focused window of the frontmost application.
    fn focused_window(&self) -> Result<Self::Window, String>;

    /// Current frame of `win`, in the shared top-left absolute space.
    fn frame(&self, win: &Self::Window) -> Rect;

    /// Move + resize `win` to `rect` (shared top-left absolute space).
    fn set_frame(&self, win: &Self::Window, rect: Rect) -> Result<(), String>;

    /// Work area (excluding menu bar / dock / taskbar) of the display `win` is on.
    fn work_area(&self, win: &Self::Window) -> Rect;

    /// Work areas of all displays.
    fn displays(&self) -> Vec<Rect>;

    /// Owning-application identity of `win`.
    fn identity(&self, win: &Self::Window) -> WindowIdentity;
}

#[cfg(target_os = "macos")]
fn platform() -> macos::MacPlatform {
    macos::MacPlatform
}

#[cfg(not(target_os = "macos"))]
fn platform() -> stub::StubPlatform {
    stub::StubPlatform
}

/// Perform `action` on the focused window through the §7 state machine: geometry comes from
/// [`crate::core::geometry::target_for_with`] (with the live user `tunables`), the state machine
/// decides the cycle step and restore baseline, and the resulting frame is re-read after the move
/// so terminals / min-size windows still cycle correctly.
///
/// `Ok(false)` is a **total no-op** (idea.md §4): either the state machine produced no target
/// (e.g. a display move with a single display) or `set_frame` changed nothing at all because the
/// window is stubborn — fixed-size, native-fullscreen, or (on Windows) elevated. The caller turns
/// that into a soft beep. A window that only *partially* complies (clamps to its min size but does
/// move) still counts as effective and returns `Ok(true)`.
pub fn perform(action: Action, state: &mut SnapState, tunables: Tunables) -> Result<bool, String> {
    let p = platform();
    let win = p.focused_window()?;
    let current = p.frame(&win);
    let work = p.work_area(&win);
    let displays = p.displays();

    let target = state.next_target(action, current, work, action.cycle_len(), |step| {
        crate::core::geometry::target_for_with(action, step, current, work, &displays, tunables)
    });
    let Some(target) = target else {
        return Ok(false);
    };
    p.set_frame(&win, target)?;
    // Re-read once: the state machine needs the landed frame, and so does the no-op check.
    let actual = p.frame(&win);
    state.record_result(actual);
    Ok(is_effective(current, target, actual))
}

/// Return the focused window to the baseline captured before its current snap run, then clear
/// the run (idea.md §7). `Ok(false)` is a total no-op the caller beeps for: either there was no
/// baseline, or the window is stubborn and couldn't move back at all. The record is cleared only
/// after a successful `set_frame`, so a failed move keeps the baseline for a retry.
pub fn restore(state: &mut SnapState) -> Result<bool, String> {
    let Some(baseline) = state.baseline() else {
        return Ok(false);
    };
    let p = platform();
    let win = p.focused_window()?;
    let before = p.frame(&win);
    p.set_frame(&win, baseline)?;
    let after = p.frame(&win);
    state.clear();
    Ok(is_effective(before, baseline, after))
}

/// Identity of the focused window's owning app (idea.md §4 ignore list). Resolves through the
/// same focused-window path as [`perform`], so it needs Accessibility and — like `perform` — must
/// run on the main thread. Backs the ignore check in dispatch and the tray "Ignore [App]" item.
pub fn focused_app_identity() -> Result<WindowIdentity, String> {
    let p = platform();
    let win = p.focused_window()?;
    Ok(p.identity(&win))
}

/// Whether an action that started at `before`, aimed for `target`, and landed at `actual` did
/// anything effective. It is a no-op (returns `false`) only when the action *meant* to move the
/// window (`target` differs from `before`) but the window did not budge at all (`actual` still
/// equals `before`) — the §4 "stubborn window". An action already at its target, or one that
/// moved only partially, is effective. All comparisons use the shared frame tolerance.
fn is_effective(before: Rect, target: Rect, actual: Rect) -> bool {
    let intended_change = !before.approx_eq(target);
    let effected_change = !before.approx_eq(actual);
    !intended_change || effected_change
}

/// Minimum gap between beeps so holding a shortcut down (auto-repeat) doesn't machine-gun the
/// system alert sound.
const BEEP_DEBOUNCE: Duration = Duration::from_millis(300);

/// Signal a total no-op (idea.md §4) by playing the system alert sound, debounced so a key
/// auto-repeat produces one beep, not a burst. The sound itself lives behind the platform
/// boundary ([`beep`]) — macOS `NSBeep` now; a Windows `MessageBeep` lands in issue 027.
pub fn notify_no_op() {
    static LAST_BEEP: Mutex<Option<Instant>> = Mutex::new(None);
    let now = Instant::now();
    {
        let mut last = LAST_BEEP.lock().unwrap();
        if last.is_some_and(|t| now.duration_since(t) < BEEP_DEBOUNCE) {
            return;
        }
        *last = Some(now);
    }
    beep();
}

#[cfg(target_os = "macos")]
fn beep() {
    macos::beep();
}

/// Windows `MessageBeep` arrives with the rest of the Windows shim (issue 027); silent until then.
#[cfg(not(target_os = "macos"))]
fn beep() {}

/// Non-macOS placeholder so the crate builds off macOS. Real Windows I/O lands in 025.
#[cfg(not(target_os = "macos"))]
mod stub {
    use super::{Platform, Rect, WindowIdentity};

    pub struct StubPlatform;

    impl Platform for StubPlatform {
        type Window = ();

        fn focused_window(&self) -> Result<Self::Window, String> {
            Err("window control is only implemented on macOS so far".to_string())
        }
        fn frame(&self, _win: &Self::Window) -> Rect {
            Rect::ZERO
        }
        fn set_frame(&self, _win: &Self::Window, _rect: Rect) -> Result<(), String> {
            Err("window control is only implemented on macOS so far".to_string())
        }
        fn work_area(&self, _win: &Self::Window) -> Rect {
            Rect::ZERO
        }
        fn displays(&self) -> Vec<Rect> {
            Vec::new()
        }
        fn identity(&self, _win: &Self::Window) -> WindowIdentity {
            WindowIdentity::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_effective, WindowIdentity};
    use crate::core::geometry::Rect;

    fn identity(bundle: Option<&str>, name: Option<&str>) -> WindowIdentity {
        WindowIdentity {
            bundle_id: bundle.map(str::to_string),
            name: name.map(str::to_string),
        }
    }

    #[test]
    fn key_prefers_bundle_id_then_name() {
        assert_eq!(
            identity(Some("com.apple.Safari"), Some("Safari")).key(),
            Some("com.apple.Safari".to_string())
        );
        assert_eq!(identity(None, Some("Some App")).key(), Some("Some App".to_string()));
        assert_eq!(identity(None, None).key(), None);
    }

    #[test]
    fn is_ignored_matches_on_key() {
        let list = vec!["com.apple.Safari".to_string(), "com.foo.Bar".to_string()];
        // Keyed by bundle id, present in the list → ignored.
        assert!(identity(Some("com.foo.Bar"), Some("Bar")).is_ignored(&list));
        // Not in the list → managed.
        assert!(!identity(Some("com.other.App"), Some("App")).is_ignored(&list));
        // Anonymous (no key) → never ignored, even against a non-empty list.
        assert!(!identity(None, None).is_ignored(&list));
        // Falls back to the name as the key when there's no bundle id.
        let name_list = vec!["Legacy App".to_string()];
        assert!(identity(None, Some("Legacy App")).is_ignored(&name_list));
    }

    #[test]
    fn display_label_falls_back() {
        assert_eq!(identity(Some("com.x.Y"), Some("Why")).display_label(), "Why");
        assert_eq!(identity(Some("com.x.Y"), None).display_label(), "com.x.Y");
        assert_eq!(identity(None, None).display_label(), "this app");
    }

    const START: Rect = Rect {
        x: 100.0,
        y: 100.0,
        w: 300.0,
        h: 300.0,
    };

    #[test]
    fn immovable_window_is_a_noop() {
        // Asked to move to a different frame, but the window didn't budge at all.
        let target = Rect::new(0.0, 0.0, 700.0, 500.0);
        assert!(!is_effective(START, target, START));
    }

    #[test]
    fn partial_comply_still_counts_as_effective() {
        // Aimed for a big frame; the window moved but clamped to a min size — still moved.
        let target = Rect::new(0.0, 0.0, 700.0, 500.0);
        let landed = Rect::new(0.0, 0.0, 700.0, 420.0);
        assert!(is_effective(START, target, landed));
    }

    #[test]
    fn already_at_target_does_not_beep() {
        // Pressing the same snap again: nothing to do, but that's not a stubborn window.
        assert!(is_effective(START, START, START));
    }

    #[test]
    fn ordinary_move_is_effective() {
        let target = Rect::new(500.0, 100.0, 300.0, 300.0);
        assert!(is_effective(START, target, target));
    }

    #[test]
    fn no_op_uses_the_shared_tolerance() {
        // A few points of drift (rounding) from the start is still "didn't move".
        let target = Rect::new(0.0, 0.0, 700.0, 500.0);
        let drifted = Rect::new(START.x + 3.0, START.y - 2.0, START.w + 1.0, START.h);
        assert!(!is_effective(START, target, drifted));
    }
}
