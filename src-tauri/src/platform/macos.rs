//! macOS window control via the Accessibility API (`AXUIElement`).
//!
//! First slice: snap the focused window to the left half of the *main* display's
//! visible frame. Multi-display selection and the shared fraction-based core come later.

// `cocoa` is deprecated in favour of the objc2 crates; it still works. Migrating the
// NSScreen access to objc2-app-kit is a follow-up cleanup.
#![allow(deprecated)]

use std::ffi::c_void;
use std::ptr;

use accessibility_sys::{
    kAXErrorSuccess, kAXFocusedApplicationAttribute, kAXFocusedWindowAttribute,
    kAXPositionAttribute, kAXSizeAttribute, kAXTrustedCheckOptionPrompt, kAXValueTypeCGPoint,
    kAXValueTypeCGSize, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
    AXUIElementCopyAttributeValue, AXUIElementCreateSystemWide, AXUIElementRef,
    AXUIElementSetAttributeValue, AXValueCreate,
};
use cocoa::appkit::NSScreen;
use cocoa::base::nil;
use cocoa::foundation::NSRect;
use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_foundation_sys::base::{CFRelease, CFTypeRef};
use core_graphics::geometry::{CGPoint, CGSize};

/// Releases a copied `AXUIElement` (a CoreFoundation object) on drop.
struct AxElement(AXUIElementRef);

impl Drop for AxElement {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0 as CFTypeRef) };
        }
    }
}

/// Whether the process is trusted for the Accessibility API. If not, this pops the
/// system prompt that points the user at System Settings → Privacy & Security.
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

/// Copy an `AXUIElement`-valued attribute (e.g. focused app / focused window).
unsafe fn copy_element_attr(element: AXUIElementRef, attr: &str) -> Option<AxElement> {
    let attr = CFString::new(attr);
    let mut value: CFTypeRef = ptr::null();
    let err = AXUIElementCopyAttributeValue(element, attr.as_concrete_TypeRef(), &mut value);
    if err == kAXErrorSuccess && !value.is_null() {
        Some(AxElement(value as AXUIElementRef))
    } else {
        None
    }
}

/// Left half of the main display's visible frame, in top-left global (AX) coordinates.
fn main_screen_left_half() -> (CGPoint, CGSize) {
    unsafe {
        let screen = NSScreen::mainScreen(nil);
        let full: NSRect = screen.frame();
        let visible: NSRect = screen.visibleFrame();

        // NSScreen uses a bottom-left origin (y up); the Accessibility API uses a
        // top-left origin (y down). Flip the visible frame using the screen's full height.
        let ax_x = visible.origin.x;
        let ax_y = full.size.height - (visible.origin.y + visible.size.height);
        let width = visible.size.width / 2.0;
        let height = visible.size.height;

        (CGPoint::new(ax_x, ax_y), CGSize::new(width, height))
    }
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

/// Snap the currently focused window to the left half of the main display.
pub fn snap_focused_window_left_half() -> Result<(), String> {
    if !ensure_trusted() {
        return Err("Accessibility permission not granted yet — enable JC Grid Manager in \
                    System Settings → Privacy & Security → Accessibility, then try again."
            .to_string());
    }

    unsafe {
        let system_wide = AxElement(AXUIElementCreateSystemWide());
        let app = copy_element_attr(system_wide.0, kAXFocusedApplicationAttribute)
            .ok_or("no focused application")?;
        let window = copy_element_attr(app.0, kAXFocusedWindowAttribute)
            .ok_or("the focused app has no focused window")?;

        let (origin, size) = main_screen_left_half();

        set_axvalue(
            window.0,
            kAXPositionAttribute,
            kAXValueTypeCGPoint,
            &origin as *const CGPoint as *const c_void,
        )?;
        set_axvalue(
            window.0,
            kAXSizeAttribute,
            kAXValueTypeCGSize,
            &size as *const CGSize as *const c_void,
        )?;
    }

    Ok(())
}
