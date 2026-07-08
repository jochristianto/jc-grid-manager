//! Shared, platform-agnostic core: all the window-placement logic, unit-testable with no
//! real windows (idea.md §5.3, §10). The per-OS shims in [`crate::platform`] provide only
//! raw window I/O; everything smart lives here.

pub mod actions;
pub mod geometry;
