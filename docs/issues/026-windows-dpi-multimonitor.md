# 026 — Windows per-monitor-DPI-v2 + multi-monitor

| | |
|---|---|
| **Issue ID** | 026 |
| **Layer** | windows (per-OS I/O shim) |
| **Depends on** | 025 |
| **Blocks** | — |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §5.3 (Windows note), §10, §11 |
| **Status** | ☐ Not started |

## Summary

Make Windows correct on **mixed-DPI, multi-monitor** setups. Win32 reports scaled/virtualized coordinates unless the app declares **per-monitor-DPI-v2 awareness** (§5.3). Declare it, then verify snapping lands pixel-correct across monitors with different scale factors — the highest-risk area on Windows, mirroring the macOS multi-monitor risk (§11).

## Context (self-contained)

- Without per-monitor-DPI-v2, `GetWindowRect` / `SetWindowPos` / `GetMonitorInfo` return DPI-virtualized values on secondary monitors, so windows land in the wrong place or wrong size on mixed-scale setups.
- Declare awareness via the app manifest (`dpiAwareness = PerMonitorV2`) and/or `SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2)` early at startup (before any window/monitor query). With Tauri, ensure the declaration actually takes effect for the running process (manifest embedding or an early API call).
- The shared core already works in fractions of a work area, so once the shim returns **true physical pixels** consistently, multi-monitor "just works" — this slice is about making the shim's numbers honest.

## What to build

1. Declare per-monitor-DPI-v2 awareness (manifest and/or `SetProcessDpiAwarenessContext` at startup); confirm it's active (`GetThreadDpiAwarenessContext`).
2. Audit the `work_area`/`displays`/`frame`/`set_frame` paths return consistent physical-pixel coordinates across monitors.
3. Verify snapping on a mixed-DPI, multi-monitor arrangement (e.g. a 150%-scaled laptop panel + a 100% external), including a monitor placed at negative virtual coordinates.

## Acceptance criteria

- [ ] The process runs as per-monitor-DPI-v2 aware (verified at runtime).
- [ ] Snapping on a secondary monitor with a **different** scale factor lands pixel-correct (right monitor, right size, edges flush).
- [ ] A window on a monitor at negative virtual coordinates snaps correctly.
- [ ] No regression on a single-monitor / single-DPI setup.

## Testing

- Manual on Windows with two monitors at different scales: snap halves/corners/Maximize on each; drag a window across and re-snap; place a monitor left of primary (negative coords) and verify. This is the §10 "manual smoke on real hardware, multi-monitor" surface.

## LLM prompt

```text
You are implementing issue 026 for JC Grid Manager. Read docs/idea.md §5.3 (Windows), §10, §11 and
docs/issues/026-windows-dpi-multimonitor.md. Issue 025 (Windows core shim) is done.

Task: Make the Windows shim correct on mixed-DPI multi-monitor setups by declaring per-monitor-DPI-v2
awareness (app manifest and/or SetProcessDpiAwarenessContext at startup, before any monitor/window query)
and ensuring frame/work_area/displays/set_frame return consistent physical pixels. Verify snapping across
monitors with different scale factors and a negative-coordinate monitor.

Definition of done: the process is per-monitor-DPI-v2 aware at runtime; snapping is pixel-correct across
mixed-DPI monitors and negative-coordinate monitors; no single-monitor regression.

Constraints: don't change the shared core (it's already fraction-based) — only make the shim's pixel
numbers honest. Scope is DPI + multi-monitor correctness.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary (how you
declared DPI awareness and what you tested on); do NOT git commit; refine the Suggested commit message;
the user commits.
```

## Implementation log (fill this in)

- **Started:** _<!-- -->_
- **Finished:** _<!-- -->_
- **Duration:** _<!-- -->_

## Implementation summary (fill this in)

_<!-- ... -->_

## Suggested commit message

```
fix(windows): declare per-monitor-DPI-v2 and fix mixed-DPI multi-monitor snaps

Run as per-monitor-DPI-v2 aware so Win32 returns true physical pixels; snapping
now lands correctly across monitors with different scale factors and negative
virtual coordinates.
```
