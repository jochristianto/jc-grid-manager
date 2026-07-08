//! Platform window I/O.
//!
//! The [`Platform`] trait is the thin per-OS shim from idea.md §5.3: a handful of dumb I/O
//! methods with no placement logic. Everything smart lives in [`crate::core`]. macOS
//! implements the trait via the Accessibility API; Windows arrives in issue 025.

use crate::core::actions::Action;
use crate::core::geometry::{Rect, Tunables};
use crate::core::state::SnapState;

#[cfg(target_os = "macos")]
mod macos;

/// Identity of the application that owns a window. Consumed by the ignore-app list (022).
#[allow(dead_code)] // constructed for real once issue 022 wires identity() into dispatch.
#[derive(Debug, Clone, Default)]
pub struct WindowIdentity {
    pub bundle_id: Option<String>,
    pub name: Option<String>,
}

/// The per-OS I/O shim: raw window reads/writes only, no placement logic (idea.md §5.3).
///
/// `identity` is still forward-looking — wired into dispatch by issue 022 (ignore-app list)
/// and stubbed until then; `frame` / `displays` came online with display selection (003). The
/// whole trait is declared up front so the Windows shim (025) can implement it in one pass.
#[allow(dead_code)]
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
/// so terminals / min-size windows still cycle correctly. `Ok(false)` means the action produced
/// no move — a graceful no-op the caller logs.
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
    state.record_result(p.frame(&win));
    Ok(true)
}

/// Return the focused window to the baseline captured before its current snap run, then clear
/// the run (idea.md §7). `Ok(false)` means there was no baseline — a graceful no-op the caller
/// reports (a soft beep arrives in 021). The record is cleared only after a successful move, so
/// a failed `set_frame` keeps the baseline for a retry.
pub fn restore(state: &mut SnapState) -> Result<bool, String> {
    let Some(baseline) = state.baseline() else {
        return Ok(false);
    };
    let p = platform();
    let win = p.focused_window()?;
    p.set_frame(&win, baseline)?;
    state.clear();
    Ok(true)
}

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
