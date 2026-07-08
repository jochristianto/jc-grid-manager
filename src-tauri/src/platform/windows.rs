//! Windows window control via Win32 (idea.md §1/§5.3).
//!
//! Implements the [`Platform`] shim — raw window I/O only, no placement logic. Win32 already lives
//! in the shared top-left, y-down, physical-pixel space of [`crate::core::geometry::Rect`], so
//! (unlike macOS) no coordinate flip is needed here. Which display a window is on is decided by the
//! pure [`display_for`] in core, not here. Per-monitor-DPI correctness is issue 026; the elevated
//! beep + Ctrl+Alt conflicts are issue 027.
//!
//! The `windows` 0.61 API usage below was cross-checked with
//! `cargo check --target x86_64-pc-windows-msvc`.

use std::mem::size_of;

use windows::core::{BOOL, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};
use windows::Win32::System::Diagnostics::Debug::MessageBeep;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::HiDpi::{
    AreDpiAwarenessContextsEqual, GetThreadDpiAwarenessContext, SetProcessDpiAwarenessContext,
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId, SetWindowPos, HWND_TOP, MB_OK,
    SWP_NOACTIVATE, SWP_NOZORDER,
};

use super::{Platform, WindowIdentity};
use crate::core::geometry::{display_for, Rect};

/// Declare per-monitor-DPI-v2 awareness (idea.md §5.3) so Win32 reports **true physical pixels**
/// across mixed-DPI monitors, instead of the DPI-virtualized coordinates an unaware process gets on
/// secondary displays. The shared core already works in work-area fractions, so once the shim's
/// numbers are honest, multi-monitor "just works".
///
/// Best-effort and idempotent: it must run before any window/monitor query, so [`crate::run`] calls
/// it first thing. If the process is already aware (Tauri/tao may set a context during init), the
/// `Set` call fails harmlessly and we keep whatever context is active — logging whether it ended up
/// v2 so the runtime state is verifiable.
pub fn ensure_dpi_awareness() {
    unsafe {
        // `let _`: setting fails if awareness was already established (manifest or a prior call);
        // that's fine — we only need the process to end up per-monitor-v2 aware.
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let ctx = GetThreadDpiAwarenessContext();
        let is_v2 =
            AreDpiAwarenessContextsEqual(ctx, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).as_bool();
        println!("[jc-grid-manager] per-monitor-DPI-v2 aware: {is_v2}");
    }
}

/// Play the standard Windows alert sound. Used by [`super::notify_no_op`] to signal that an action
/// could do nothing (idea.md §4) — including the common Windows case of an **elevated/admin window**
/// a non-elevated app can't move: `SetWindowPos` silently no-ops there, so the 021 effect check sees
/// no change and routes through here (issue 027). Best-effort; a failed beep is ignored.
pub fn beep() {
    let _ = unsafe { MessageBeep(MB_OK) };
}

/// The Windows implementation of [`Platform`]. `HWND` is a plain copyable handle, so there is no
/// owned-resource wrapper (unlike the macOS `AxWindow`).
pub struct WinPlatform;

impl Platform for WinPlatform {
    type Window = HWND;

    fn focused_window(&self) -> Result<Self::Window, String> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.0.is_null() {
            Err("no foreground window".to_string())
        } else {
            Ok(hwnd)
        }
    }

    fn frame(&self, win: &Self::Window) -> Rect {
        // GetWindowRect returns the outer rect, which on Win10+ includes an invisible resize
        // border of a few px. Compensating via DWMWA_EXTENDED_FRAME_BOUNDS for pixel-accurate
        // visual edges is a follow-up (noted in issue 025).
        let mut r = RECT::default();
        if unsafe { GetWindowRect(*win, &mut r) }.is_ok() {
            rect_from(r)
        } else {
            Rect::ZERO
        }
    }

    fn set_frame(&self, win: &Self::Window, rect: Rect) -> Result<(), String> {
        // SWP_NOZORDER keeps the window's stacking; SWP_NOACTIVATE avoids stealing focus. A window
        // that refuses to move (fixed-size, or an elevated window we can't touch) returns an error
        // the core turns into a soft beep (021 / 027) — it must not crash.
        unsafe {
            SetWindowPos(
                *win,
                Some(HWND_TOP),
                rect.x as i32,
                rect.y as i32,
                rect.w as i32,
                rect.h as i32,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
        }
        .map_err(|e| e.message())
    }

    fn work_area(&self, win: &Self::Window) -> Rect {
        // Work area of the display this window is on, chosen by largest overlap in pure core.
        display_for(self.frame(win), &self.displays())
    }

    fn displays(&self) -> Vec<Rect> {
        unsafe { all_work_areas() }
    }

    fn identity(&self, win: &Self::Window) -> WindowIdentity {
        unsafe { window_identity(*win) }
    }
}

/// Convert a Win32 `RECT` (left/top/right/bottom) into a top-left `Rect` (x/y/w/h).
fn rect_from(r: RECT) -> Rect {
    Rect::new(
        r.left as f64,
        r.top as f64,
        (r.right - r.left) as f64,
        (r.bottom - r.top) as f64,
    )
}

/// `EnumDisplayMonitors` callback: read each monitor's work area (`rcWork`, i.e. excluding the
/// taskbar) and push it into the `Vec<Rect>` passed through `data`.
unsafe extern "system" fn enum_proc(mon: HMONITOR, _hdc: HDC, _r: *mut RECT, data: LPARAM) -> BOOL {
    let out = &mut *(data.0 as *mut Vec<Rect>);
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if GetMonitorInfoW(mon, &mut info).as_bool() {
        out.push(rect_from(info.rcWork));
    }
    TRUE
}

/// Work areas of every display, in the shared top-left space.
unsafe fn all_work_areas() -> Vec<Rect> {
    let mut out: Vec<Rect> = Vec::new();
    let _ = EnumDisplayMonitors(
        None,
        None,
        Some(enum_proc),
        LPARAM(&mut out as *mut Vec<Rect> as isize),
    );
    out
}

/// Owning-application identity of `win` (idea.md §4 ignore list): the full exe path as the stable
/// key, plus the exe file name as the display name. Missing pieces come back as `None`.
unsafe fn window_identity(win: HWND) -> WindowIdentity {
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(win, Some(&mut pid));
    if pid == 0 {
        return WindowIdentity::default();
    }
    let path = exe_path(pid);
    let name = path
        .as_deref()
        .and_then(|p| p.rsplit(['\\', '/']).next())
        .map(str::to_string);
    WindowIdentity {
        bundle_id: path,
        name,
    }
}

/// Full image path of process `pid` via `QueryFullProcessImageNameW`, or `None` if it can't be
/// opened (e.g. a more-privileged process). Uses `PROCESS_QUERY_LIMITED_INFORMATION`, the
/// least-privilege right that works across integrity levels.
unsafe fn exe_path(pid: u32) -> Option<String> {
    let handle: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
    let mut buf = [0u16; 512];
    let mut len = buf.len() as u32;
    let result =
        QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len);
    let _ = CloseHandle(handle);
    result.ok()?;
    Some(String::from_utf16_lossy(&buf[..len as usize]))
}
