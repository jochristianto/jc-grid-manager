# 025 — Windows Win32 core shim

| | |
|---|---|
| **Issue ID** | 025 |
| **Layer** | windows (per-OS I/O shim) |
| **Depends on** | 001 (the `Platform` trait). Best validated **after** the macOS actions exist so behavior is proven. |
| **Blocks** | 026, 027 |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §1, §5.3, §5.6, §8 |
| **Status** | ☐ Not started |

## Summary

The whole reason the app exists: bring the shared engine to **Windows** (§1). Implement the `Platform` trait for Windows with Win32 so **every** action, the cycling state machine, config, and the tray all work on Windows via the shared core — no per-action Windows code. This slice is the **core shim** (basic single-monitor snapping working end-to-end); DPI/multi-monitor correctness is 026 and Windows-specific quirks are 027.

## Context (self-contained)

- After 001, `platform/mod.rs` defines the trait and cfg-selects `macos` / `windows`. Today the non-macOS path is a stub (`platform/windows.rs` doesn't exist yet).
- Use `windows` (windows-rs) crate. Map the ~6 trait methods to Win32:
  - `focused_window()` → `GetForegroundWindow`
  - `frame(win)` → `GetWindowRect` (note: this is the *outer* rect; extended frame bounds via `DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS)` may be needed for pixel-accurate edges — see decision)
  - `set_frame(win, rect)` → `SetWindowPos` (with `SWP_NOZORDER | SWP_NOACTIVATE`)
  - `work_area(win)` → `MonitorFromWindow` + `GetMonitorInfo` → `rcWork`
  - `displays()` → enumerate monitors (`EnumDisplayMonitors`) → each `rcWork`
  - `identity(win)` → owning process (`GetWindowThreadProcessId` → exe path / AUMID) for the ignore list (022)
- Win32 works in **physical pixels**; the DPI manifest that makes those correct on mixed-DPI setups is issue 026 (declare it there). This slice targets a single-DPI setup first.
- No special permission is needed for user-level windows; **elevated/admin windows can't be moved** unless the app is elevated (it won't be) — those simply won't respond (handled by 027's beep). Don't crash on them.

## What to build

1. Create `platform/windows.rs` implementing the `Platform` trait via windows-rs; add the crate under a `cfg(windows)` target in `Cargo.toml`.
2. Wire `platform/mod.rs` to select it on Windows (remove/replace the stub).
3. Get the four halves + a couple of other actions working end-to-end on Windows (single monitor).
4. Handle failures gracefully (a window that won't move returns an error the core turns into a beep via 021).

## Open decisions (recommendation)

- **`GetWindowRect` vs extended frame bounds** — Win10+ windows have invisible resize borders, so `GetWindowRect` includes a few px of shadow, making snapped windows look slightly misaligned. _Recommendation:_ read `DWMWA_EXTENDED_FRAME_BOUNDS` for measuring and compensate on `SetWindowPos` so visual edges align (Rectangle-for-Windows tools do this). If time-boxed, ship `GetWindowRect` first and note the follow-up.

## Acceptance criteria

- [ ] `platform/windows.rs` implements all `Platform` trait methods via Win32; the project builds on Windows.
- [ ] On a single-monitor Windows machine, the four halves and a couple of other actions (e.g. a corner, Maximize) snap correctly.
- [ ] Cycling (state machine) and Restore work on Windows via the shared core (no Windows-specific action code).
- [ ] Attempting to move an unmovable/elevated window fails gracefully (no crash; sets up 021/027's beep).

## Testing

- Manual on a Windows machine: grant nothing (not needed), press the halves and a corner; repeat a directional key to confirm cycling; press Restore.
- (Multi-monitor + mixed DPI correctness is verified in 026.)

## LLM prompt

```text
You are implementing issue 025 for JC Grid Manager. Read docs/idea.md §1/§5.3/§5.6/§8 and
docs/issues/025-windows-win32-core-shim.md. Issue 001 (Platform trait) is done and the macOS actions
exist to compare behavior against.

Task: Implement the Platform trait for Windows in a new platform/windows.rs using the windows-rs crate
(GetForegroundWindow, GetWindowRect / DWM extended frame bounds, SetWindowPos, MonitorFromWindow +
GetMonitorInfo rcWork, EnumDisplayMonitors, owning-process identity). Wire platform/mod.rs to select it
on Windows. Get the halves + a couple of other actions working end-to-end on a single monitor; cycling,
Restore, and config must work via the shared core with no Windows-specific action logic. Fail gracefully
on unmovable/elevated windows (no crash).

Definition of done: builds on Windows; halves/corner/Maximize snap correctly single-monitor; cycling +
Restore work; unmovable windows don't crash.

Constraints: this is the CORE shim only — per-monitor DPI awareness is issue 026 and Windows-specific
quirks (elevated beep, Ctrl+Alt conflicts) are issue 027. Do not add per-action Windows code; the point is
that the shared core drives everything.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary (note
whether you used GetWindowRect or DWM extended frame bounds); do NOT git commit; refine the Suggested
commit message; the user commits.
```

## Implementation log (fill this in)

- **Started:** _<!-- -->_
- **Finished:** _<!-- -->_
- **Duration:** _<!-- -->_

## Implementation summary (fill this in)

_<!-- ... -->_

## Suggested commit message

```
feat(windows): implement the Platform trait via Win32

Add platform/windows.rs (windows-rs) implementing focused window, frame,
set_frame, work_area, displays, and identity so the shared core drives all
actions on Windows. Single-monitor snapping, cycling, and restore work.
```
