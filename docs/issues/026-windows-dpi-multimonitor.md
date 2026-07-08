# 026 — Windows per-monitor-DPI-v2 + multi-monitor

| | |
|---|---|
| **Issue ID** | 026 |
| **Layer** | windows (per-OS I/O shim) |
| **Depends on** | 025 |
| **Blocks** | — |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §5.3 (Windows note), §10, §11 |
| **Status** | ☑ Done (type-checked for Windows; mixed-DPI hardware smoke pending) |

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

## Implementation log

- **Started:** 2026-07-08 22:22 WIB
- **Finished:** 2026-07-08 22:26 WIB
- **Duration:** ~4m hands-on (excludes reading/design)

## Implementation summary

Made the Windows shim honest on mixed-DPI multi-monitor setups by declaring
**per-monitor-DPI-v2** awareness at startup. No core changes — it's fraction-based already; this
just makes the shim's pixel numbers true across displays.

**What changed**
- **`platform/windows.rs`** — `ensure_dpi_awareness()`:
  `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)`, then reads back
  `GetThreadDpiAwarenessContext` + `AreDpiAwarenessContextsEqual` and logs whether the process
  ended up v2-aware (the runtime-verifiable check the acceptance asks for). Best-effort +
  idempotent: if awareness was already set (Tauri/tao may set a context during init), the `Set`
  fails harmlessly and we keep the active context.
- **`platform/mod.rs`** — `ensure_dpi_awareness()` boundary fn (Windows → the above; **no-op off
  Windows** — macOS points are DPI-independent).
- **`lib.rs`** — calls `platform::ensure_dpi_awareness()` as the **first line of `run()`**, before
  `tauri::Builder`, i.e. before any window or monitor query.
- **`Cargo.toml`** — added the `Win32_UI_HiDpi` feature (no `Cargo.lock` change; the sub-crates were
  already resolved via other `windows` features).
- The `frame`/`set_frame`/`work_area`/`displays` paths need no edits: under v2 awareness they
  already return consistent physical pixels (`GetWindowRect`/`SetWindowPos`/`GetMonitorInfoW`), and
  a monitor at negative virtual coords is carried through `Rect`'s origin unchanged — the same way
  macOS handles a display left of the primary.

**Key decisions / deviations**
- **Runtime `SetProcessDpiAwarenessContext`, not a manifest.** The API call is toolchain-free
  (embedding a `dpiAware` manifest needs the resource compiler this dev box lacks) and, called as
  the very first thing in `run()`, wins before tao would set a context. A manifest declaration
  remains a valid alternative for packaging if ever needed.
- Kept it best-effort: even if a context is already established, we log the effective state rather
  than fail, so the user can confirm "per-monitor-DPI-v2 aware: true" in the console.

**Verification** (macOS dev box — no Windows hardware)
- Cross-checked the HiDpi API (`SetProcessDpiAwarenessContext` / `GetThreadDpiAwarenessContext` /
  `AreDpiAwarenessContextsEqual` / `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2`) in the isolated
  crate: `cargo check --target x86_64-pc-windows-msvc` → clean.
- `windows.rs` + the `lib.rs` call type-check cleanly for `x86_64-pc-windows-msvc` (via the
  temporary `build.rs` no-op; `build.rs` unchanged in the commit).
- macOS unaffected: `cargo test` 68 pass, `cargo clippy --all-targets` clean.
- **Not run here (needs Windows + two monitors at different scales):** confirm the log prints
  v2-aware; snap halves/corners/Maximize on a 150% panel + a 100% external and check edges are
  flush; verify a monitor at negative virtual coords; confirm no single-monitor regression. Flagged
  for the user (the §10 real-hardware smoke).

## Suggested commit message

```
fix(windows): declare per-monitor-DPI-v2 for mixed-DPI multi-monitor snaps

Call SetProcessDpiAwarenessContext(PER_MONITOR_AWARE_V2) first thing in run(),
before any window/monitor query, so Win32 returns true physical pixels across
displays with different scale factors (and negative virtual coordinates). Log the
effective awareness so it's verifiable at runtime. Best-effort/idempotent; no core
changes (it's fraction-based). Cross-checked for x86_64-pc-windows-msvc; the
mixed-DPI hardware smoke needs a Windows machine.
```
