//! macOS window control via the Accessibility API (`AXUIElement`).
//!
//! Implements the [`Platform`] shim (idea.md §5.3): raw window I/O only. Coordinate
//! conversion lives here — AX positions are top-left-origin points, while `NSScreen` frames
//! are bottom-left-origin; both are converted into the shared top-left space of
//! [`crate::core::geometry::Rect`]. Because everything is in points, mixing Retina and
//! non-Retina displays needs no special handling.

// `cocoa` is deprecated in favour of the objc2 crates; it still works. Migrating the
// NSScreen / NSWorkspace access to objc2 is a follow-up cleanup.
#![allow(deprecated)]

use std::ffi::c_void;

use accessibility_sys::{
    kAXErrorSuccess, kAXFocusedWindowAttribute, kAXPositionAttribute, kAXSizeAttribute,
    kAXTrustedCheckOptionPrompt, kAXValueTypeCGPoint, kAXValueTypeCGSize, AXIsProcessTrusted,
    AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue, AXUIElementCreateApplication,
    AXUIElementRef, AXUIElementSetAttributeValue, AXValueCreate,
};
use cocoa::appkit::NSScreen;
use cocoa::base::{id, nil};
use cocoa::foundation::NSRect;
use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_foundation_sys::base::{CFRelease, CFTypeRef};
use core_graphics::geometry::{CGPoint, CGSize};
use objc::{class, msg_send, sel, sel_impl};

use super::{Platform, WindowIdentity};
use crate::core::geometry::Rect;

/// The macOS implementation of [`Platform`].
pub struct MacPlatform;

/// An owned `AXUIElement` window handle; releases the underlying CF object on drop.
pub struct AxWindow(AXUIElementRef);

impl Drop for AxWindow {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0 as CFTypeRef) };
        }
    }
}

impl Platform for MacPlatform {
    type Window = AxWindow;

    fn focused_window(&self) -> Result<Self::Window, String> {
        // Accessibility permission is required to read/move other apps' windows. Prompt
        // once if we don't have it yet, and return a helpful error meanwhile.
        if !ensure_trusted() {
            return Err("Accessibility permission not granted yet — enable JC Grid Manager in \
                        System Settings → Privacy & Security → Accessibility, then try again."
                .to_string());
        }
        unsafe { focused_window() }
    }

    fn set_frame(&self, win: &Self::Window, rect: Rect) -> Result<(), String> {
        let origin = CGPoint::new(rect.x, rect.y);
        let size = CGSize::new(rect.w, rect.h);
        unsafe { set_frame(win.0, origin, size) }
    }

    fn work_area(&self, _win: &Self::Window) -> Rect {
        // Issue 001 preserves the original behavior: always the MAIN display's work area.
        // Issue 003 upgrades this to the display `win` is actually on (by largest overlap).
        main_work_area()
    }

    // --- Contract methods with no caller in this slice --------------------------------
    // These complete the §5.3 shim so Windows (025) can mirror the full trait, but nothing
    // dispatches to them yet. Each is implemented for real by the slice that first needs
    // it; until then they are cheap, never-invoked placeholders.

    fn frame(&self, _win: &Self::Window) -> Rect {
        Rect::ZERO
    }

    fn displays(&self) -> Vec<Rect> {
        Vec::new()
    }

    fn identity(&self, _win: &Self::Window) -> WindowIdentity {
        WindowIdentity::default()
    }
}

/// Whether the process is trusted for the Accessibility API. If not, this pops the system
/// prompt that points the user at System Settings → Privacy & Security.
fn ensure_trusted() -> bool {
    unsafe {
        if AXIsProcessTrusted() {
            return true;
        }
        let key = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
        let value = CFBoolean::true_value();
        let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
        AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef());
        false
    }
}

/// The focused window of the frontmost application.
///
/// Uses `NSWorkspace.frontmostApplication` (reliable) instead of the system-wide
/// `AXFocusedApplication` attribute, which returns nothing during app/focus transitions.
unsafe fn focused_window() -> Result<AxWindow, String> {
    let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
    let app: id = msg_send![workspace, frontmostApplication];
    if app == nil {
        return Err("no frontmost application".into());
    }
    let pid: i32 = msg_send![app, processIdentifier];
    let app_element = AxWindow(AXUIElementCreateApplication(pid));
    copy_element_attr(app_element.0, kAXFocusedWindowAttribute)
        .ok_or_else(|| "the focused app has no movable window".to_string())
}

/// Copy an `AXUIElement`-valued attribute (e.g. the focused window) as an owned handle.
unsafe fn copy_element_attr(element: AXUIElementRef, attr: &str) -> Option<AxWindow> {
    let attr = CFString::new(attr);
    let mut value: CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(element, attr.as_concrete_TypeRef(), &mut value);
    if err == kAXErrorSuccess && !value.is_null() {
        Some(AxWindow(value as AXUIElementRef))
    } else {
        None
    }
}

/// The main display's visible frame (work area) in the shared top-left space.
fn main_work_area() -> Rect {
    unsafe {
        let screen = NSScreen::mainScreen(nil);
        let visible: NSRect = screen.visibleFrame();
        nsrect_to_toplevel(visible, primary_screen_height())
    }
}

/// Convert a bottom-left-origin `NSScreen` frame into the shared top-left space.
///
/// Cocoa uses a bottom-left origin (y up) with (0,0) at the *primary* screen's bottom-left;
/// the Accessibility API uses a top-left origin (y down) with (0,0) at the primary screen's
/// top-left. The flip must therefore use the PRIMARY screen's height — using the focused
/// screen's height breaks on multi-monitor setups where a secondary display sits at a
/// negative/large offset.
fn nsrect_to_toplevel(rect: NSRect, primary_h: f64) -> Rect {
    let x = rect.origin.x;
    let y = primary_h - (rect.origin.y + rect.size.height);
    Rect::new(x, y, rect.size.width, rect.size.height)
}

/// Height of the primary (menu-bar) screen — the origin of the global AX coordinate space.
unsafe fn primary_screen_height() -> f64 {
    let screens: id = msg_send![class!(NSScreen), screens];
    let primary: id = msg_send![screens, firstObject];
    if primary == nil {
        return 0.0;
    }
    let frame: NSRect = msg_send![primary, frame];
    frame.size.height
}

/// Build an `AXValue` from a raw pointer and set it on `window`'s `attr`.
unsafe fn set_axvalue(
    window: AXUIElementRef,
    attr: &str,
    value_type: u32,
    value_ptr: *const c_void,
) -> Result<(), String> {
    let ax_value = AXValueCreate(value_type, value_ptr);
    if ax_value.is_null() {
        return Err(format!("failed to create AXValue for {attr}"));
    }
    let err = AXUIElementSetAttributeValue(
        window,
        CFString::new(attr).as_concrete_TypeRef(),
        ax_value as CFTypeRef,
    );
    CFRelease(ax_value as CFTypeRef);
    if err == kAXErrorSuccess {
        Ok(())
    } else {
        Err(format!("could not set {attr} (AXError {err})"))
    }
}

/// Move + resize a window to `origin` / `size`.
///
/// macOS clamps a move or resize to keep the window on screen, so a naive
/// position-then-size lands wrong when the window starts larger than the target (e.g. a
/// full-height window sent to the bottom half gets shoved back up to the top). Setting
/// size, then position, then size again is the robust recipe (as Rectangle does).
unsafe fn set_frame(window: AXUIElementRef, origin: CGPoint, size: CGSize) -> Result<(), String> {
    let size_ptr = &size as *const CGSize as *const c_void;
    let origin_ptr = &origin as *const CGPoint as *const c_void;
    set_axvalue(window, kAXSizeAttribute, kAXValueTypeCGSize, size_ptr)?;
    set_axvalue(window, kAXPositionAttribute, kAXValueTypeCGPoint, origin_ptr)?;
    set_axvalue(window, kAXSizeAttribute, kAXValueTypeCGSize, size_ptr)?;
    Ok(())
}
