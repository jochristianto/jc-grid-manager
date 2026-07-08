//! Platform-specific window control.
//!
//! Exposes a `Half` action and a `snap` entry point, implemented on macOS via the
//! Accessibility API. The shared, fraction-based core and the full `Platform` trait from
//! the design doc get factored out once more of the native surface is proven.

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::{snap, Half};

#[cfg(not(target_os = "macos"))]
#[derive(Debug, Clone, Copy)]
pub enum Half {
    Left,
    Right,
    Top,
    Bottom,
}

#[cfg(not(target_os = "macos"))]
pub fn snap(_half: Half) -> Result<(), String> {
    Err("window control is only implemented on macOS so far".to_string())
}
