//! macOS window control via the Accessibility API (`AXUIElement`).
//!
//! Snaps the focused window to a half of the *main* display's visible frame.
//! Multi-display selection, thirds/quarters, and the repeat-to-cycle state machine come
//! in later slices; the shared fraction-based core will be factored out then.

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

/// Which half of the screen to snap the focused window to.
#[derive(Debug, Clone, Copy)]
pub enum Half {
    Left,
    Right,
    Top,
    Bottom,
}

impl Half {
    /// Target rectangle as a fraction `(x, y, w, h)` of the work area, each in `0.0..=1.0`.
    fn fraction(self) -> (f64, f64, f64, f64) {
        match self {
            Half::Left => (0.0, 0.0, 0.5, 1.0),
            Half::Right => (0.5, 0.0, 0.5, 1.0),
            Half::Top => (0.0, 0.0, 1.0, 0.5),
            Half::Bottom => (0.0, 0.5, 1.0, 0.5),
        }
    }
}

/// Releases a copied `AXUIElement` (a CoreFoundation object) on drop.
struct AxElement(AXUIElementRef);

impl Drop for AxElement {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0 as CFTypeRef) };
        }
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

/// Copy an `AXUIElement`-valued attribute (e.g. the focused window).
unsafe fn copy_element_attr(element: AXUIElementRef, attr: &str) -> Option<AxElement> {
    let attr = CFString::new(attr);
    let mut value: CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(element, attr.as_concrete_TypeRef(), &mut value);
    if err == kAXErrorSuccess && !value.is_null() {
        Some(AxElement(value as AXUIElementRef))
    } else {
        None
    }
}

/// The focused window of the frontmost application.
///
/// Uses `NSWorkspace.frontmostApplication` (reliable) instead of the system-wide
/// `AXFocusedApplication` attribute, which returns nothing during app/focus transitions.
unsafe fn focused_window() -> Result<AxElement, String> {
    let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
    let app: id = msg_send![workspace, frontmostApplication];
    if app == nil {
        return Err("no frontmost application".into());
    }
    let pid: i32 = msg_send![app, processIdentifier];
    let app_element = AxElement(AXUIElementCreateApplication(pid));
    copy_element_attr(app_element.0, kAXFocusedWindowAttribute)
        .ok_or_else(|| "the focused app has no movable window".to_string())
}

/// The main display's visible frame (work area) in top-left global (AX) coordinates.
fn main_work_area() -> (f64, f64, f64, f64) {
    unsafe {
        let screen = NSScreen::mainScreen(nil);
        let visible: NSRect = screen.visibleFrame();

        // Cocoa uses a bottom-left origin (y up) with (0,0) at the *primary* screen's
        // bottom-left; the Accessibility API uses a top-left origin (y down) with (0,0)
        // at the primary screen's top-left. The flip must therefore use the PRIMARY
        // screen's height — using the focused screen's height breaks on multi-monitor
        // setups where a secondary display sits at a negative/large offset.
        let x = visible.origin.x;
        let y = primary_screen_height() - (visible.origin.y + visible.size.height);
        (x, y, visible.size.width, visible.size.height)
    }
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
/// position-then-size lands wrong when the window starts larger than the target
/// (e.g. a full-height window sent to the bottom half gets shoved back up to the top).
/// Setting size, then position, then size again is the robust recipe (as Rectangle does).
unsafe fn set_frame(window: AXUIElementRef, origin: CGPoint, size: CGSize) -> Result<(), String> {
    let size_ptr = &size as *const CGSize as *const c_void;
    let origin_ptr = &origin as *const CGPoint as *const c_void;
    set_axvalue(window, kAXSizeAttribute, kAXValueTypeCGSize, size_ptr)?;
    set_axvalue(window, kAXPositionAttribute, kAXValueTypeCGPoint, origin_ptr)?;
    set_axvalue(window, kAXSizeAttribute, kAXValueTypeCGSize, size_ptr)?;
    Ok(())
}

/// Snap the currently focused window to the given half of the main display.
pub fn snap(half: Half) -> Result<(), String> {
    if !ensure_trusted() {
        return Err("Accessibility permission not granted yet — enable JC Grid Manager in \
                    System Settings → Privacy & Security → Accessibility, then try again."
            .to_string());
    }

    unsafe {
        let window = focused_window()?;
        let (wx, wy, ww, wh) = main_work_area();
        let (fx, fy, fw, fh) = half.fraction();

        let origin = CGPoint::new(wx + fx * ww, wy + fy * wh);
        let size = CGSize::new(fw * ww, fh * wh);

        set_frame(window.0, origin, size)?;
    }

    Ok(())
}
