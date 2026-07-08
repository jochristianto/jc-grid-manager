//! Platform window I/O.
//!
//! The [`Platform`] trait is the thin per-OS shim from idea.md §5.3: a handful of dumb I/O
//! methods with no placement logic. Everything smart lives in [`crate::core`]. macOS
//! implements the trait via the Accessibility API; Windows arrives in issue 025.

use crate::core::actions::Half;
use crate::core::geometry::{fraction_to_rect, Rect};

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
/// `frame` / `displays` / `identity` are the forward-looking half of the §5.3 contract —
/// wired into dispatch by issues 005 / 003 / 022. They are declared now (and stubbed in the
/// impls) so the Windows shim (025) can implement the whole trait in one pass.
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

/// Snap the focused window to `half` of its display's work area.
///
/// NOTE (issue 001): behavior is intentionally identical to the original shim — the work
/// area is still the **main** display's (see the macOS `work_area` impl), pending
/// per-window display selection by largest overlap in issue 003.
pub fn snap(half: Half) -> Result<(), String> {
    let p = platform();
    let win = p.focused_window()?;
    let work = p.work_area(&win);
    let target = fraction_to_rect(work, half.fraction());
    p.set_frame(&win, target)
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
