//! macOS window control via the Accessibility API (`AXUIElement`).
//!
//! Implements the [`Platform`] shim (idea.md §5.3): raw window I/O only. Coordinate
//! conversion lives here — AX positions are top-left-origin points, while `NSScreen` frames
//! are bottom-left-origin; both are converted into the shared top-left space of
//! [`crate::core::geometry::Rect`]. Because everything is in points, mixing Retina and
//! non-Retina displays needs no special handling. Which display a window is on is decided by
//! the pure [`display_for`] in core, not here.

// `cocoa` is deprecated in favour of the objc2 crates; it still works. Migrating the
// NSScreen / NSWorkspace access to objc2 is a follow-up cleanup.
#![allow(deprecated)]

use std::ffi::c_void;

use accessibility_sys::{
    kAXErrorSuccess, kAXFocusedWindowAttribute, kAXPositionAttribute, kAXSizeAttribute,
    kAXTrustedCheckOptionPrompt, kAXValueTypeCGPoint, kAXValueTypeCGSize, AXIsProcessTrusted,
    AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue, AXUIElementCreateApplication,
    AXUIElementGetPid, AXUIElementRef, AXUIElementSetAttributeValue, AXValueCreate,
    AXValueGetValue, AXValueRef,
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
use crate::core::geometry::{display_for, Rect};

/// The macOS implementation of [`Platform`].
pub struct MacPlatform;

/// Play the standard macOS alert sound. Used by [`super::notify_no_op`] to signal that an
/// action could do nothing (idea.md §4). `NSBeep` is a free AppKit function; AppKit is already
/// linked for `NSScreen` / `NSWorkspace`.
pub fn beep() {
    unsafe { NSBeep() }
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    fn NSBeep();
}

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

    fn frame(&self, win: &Self::Window) -> Rect {
        unsafe {
            let origin =
                copy_point_attr(win.0, kAXPositionAttribute).unwrap_or(CGPoint::new(0.0, 0.0));
            let size = copy_size_attr(win.0, kAXSizeAttribute).unwrap_or(CGSize::new(0.0, 0.0));
            Rect::new(origin.x, origin.y, size.width, size.height)
        }
    }

    fn set_frame(&self, win: &Self::Window, rect: Rect) -> Result<(), String> {
        let origin = CGPoint::new(rect.x, rect.y);
        let size = CGSize::new(rect.w, rect.h);
        unsafe { set_frame(win.0, origin, size) }
    }

    fn work_area(&self, win: &Self::Window) -> Rect {
        // Work area of the display this window is actually on, chosen by largest overlap
        // (idea.md §5.3). Selection is pure core logic; the shim only supplies the raw
        // window frame + the list of display work areas.
        display_for(self.frame(win), &self.displays())
    }

    fn displays(&self) -> Vec<Rect> {
        unsafe { all_work_areas() }
    }

    fn identity(&self, win: &Self::Window) -> WindowIdentity {
        // Owning-app bundle id + display name, resolved from the window's process id (§4).
        unsafe { window_identity(win.0) }
    }
}

/// Owning-application identity of the app that owns `window` (idea.md §4 ignore list): its bundle
/// id + localized name, looked up from the window's process id via `NSRunningApplication`. Any
/// piece the OS doesn't provide comes back as `None`.
unsafe fn window_identity(window: AXUIElementRef) -> WindowIdentity {
    let mut pid: i32 = 0;
    if AXUIElementGetPid(window, &mut pid) != kAXErrorSuccess {
        return WindowIdentity::default();
    }
    let app: id = msg_send![
        class!(NSRunningApplication),
        runningApplicationWithProcessIdentifier: pid
    ];
    if app == nil {
        return WindowIdentity::default();
    }
    let bundle: id = msg_send![app, bundleIdentifier];
    let name: id = msg_send![app, localizedName];
    WindowIdentity {
        bundle_id: nsstring_to_string(bundle),
        name: nsstring_to_string(name),
    }
}

/// Copy an `NSString` (`id`) into an owned Rust `String`; `nil` or a null UTF-8 buffer → `None`.
unsafe fn nsstring_to_string(s: id) -> Option<String> {
    if s == nil {
        return None;
    }
    let bytes: *const std::os::raw::c_char = msg_send![s, UTF8String];
    if bytes.is_null() {
        return None;
    }
    Some(std::ffi::CStr::from_ptr(bytes).to_string_lossy().into_owned())
}

/// Whether the process is currently trusted for the Accessibility API. Pure query, no prompt.
pub fn is_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Show the system Accessibility prompt (idea.md §8) that points the user at System Settings →
/// Privacy & Security. Safe to call when already trusted (the system just no-ops).
pub fn prompt() {
    unsafe {
        let key = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
        let value = CFBoolean::true_value();
        let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
        AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef());
    }
}

/// Open System Settings → Privacy & Security → Accessibility via `NSWorkspace` (idea.md §8). The
/// `x-apple.systempreferences:` scheme is why this goes through the shim rather than the opener
/// plugin (whose default policy only allows http/https/mailto/tel).
pub fn open_accessibility_settings() {
    const PANE: &str = "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";
    unsafe {
        let Ok(cstr) = std::ffi::CString::new(PANE) else {
            return;
        };
        let url_string: id = msg_send![class!(NSString), stringWithUTF8String: cstr.as_ptr()];
        let url: id = msg_send![class!(NSURL), URLWithString: url_string];
        if url == nil {
            return;
        }
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let _: bool = msg_send![workspace, openURL: url];
    }
}

/// Whether the process is trusted; if not, pop the system prompt once. Used when an action is
/// triggered without going through the onboarding flow (issue 030).
fn ensure_trusted() -> bool {
    if is_trusted() {
        return true;
    }
    prompt();
    false
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
    copy_attr(element, attr).map(|value| AxWindow(value as AXUIElementRef))
}

/// Copy an attribute as a raw CFType. The caller owns the +1 reference: wrap it in something
/// that `CFRelease`s (e.g. [`AxWindow`]) or release it directly.
unsafe fn copy_attr(element: AXUIElementRef, attr: &str) -> Option<CFTypeRef> {
    let attr = CFString::new(attr);
    let mut value: CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(element, attr.as_concrete_TypeRef(), &mut value);
    if err == kAXErrorSuccess && !value.is_null() {
        Some(value)
    } else {
        None
    }
}

/// Read a `CGPoint`-valued attribute (e.g. `kAXPositionAttribute`).
unsafe fn copy_point_attr(element: AXUIElementRef, attr: &str) -> Option<CGPoint> {
    let value = copy_attr(element, attr)?;
    let mut point = CGPoint::new(0.0, 0.0);
    let ok = AXValueGetValue(
        value as AXValueRef,
        kAXValueTypeCGPoint,
        &mut point as *mut CGPoint as *mut c_void,
    );
    CFRelease(value);
    ok.then_some(point)
}

/// Read a `CGSize`-valued attribute (e.g. `kAXSizeAttribute`).
unsafe fn copy_size_attr(element: AXUIElementRef, attr: &str) -> Option<CGSize> {
    let value = copy_attr(element, attr)?;
    let mut size = CGSize::new(0.0, 0.0);
    let ok = AXValueGetValue(
        value as AXValueRef,
        kAXValueTypeCGSize,
        &mut size as *mut CGSize as *mut c_void,
    );
    CFRelease(value);
    ok.then_some(size)
}

/// Work areas of every display, in the shared top-left space.
unsafe fn all_work_areas() -> Vec<Rect> {
    let primary_h = primary_screen_height();
    let screens: id = msg_send![class!(NSScreen), screens];
    let count: usize = msg_send![screens, count];
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let screen: id = msg_send![screens, objectAtIndex: i];
        if screen == nil {
            continue;
        }
        let visible: NSRect = screen.visibleFrame();
        out.push(nsrect_to_toplevel(visible, primary_h));
    }
    out
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
