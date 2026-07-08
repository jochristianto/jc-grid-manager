//! Platform-specific window control.
//!
//! For now this exposes a single action (`snap_focused_window_left_half`), implemented
//! on macOS via the Accessibility API. The shared, fraction-based core and the full
//! `Platform` trait from the design doc get factored out once the native path is proven.

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::snap_focused_window_left_half;

#[cfg(not(target_os = "macos"))]
pub fn snap_focused_window_left_half() -> Result<(), String> {
    Err("window control is only implemented on macOS so far".to_string())
}
