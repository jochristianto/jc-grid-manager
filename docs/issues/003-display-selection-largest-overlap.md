# 003 — Display selection by largest overlap

| | |
|---|---|
| **Issue ID** | 003 |
| **Layer** | core + macos shim |
| **Depends on** | 001 |
| **Blocks** | 019 (Next/Previous Display); improves correctness of every action on multi-monitor |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §5.3 ("Display selection"), §7, §11 |
| **Status** | ☑ Done |

## Summary

Today snapping always targets the **main** display (`main_work_area()`), so a window on a secondary monitor snaps to the wrong screen. Replace that with the §5.3 rule: pick the display a window is on by **largest overlap** of the window's frame with each display's work area — not by which display a single corner lands in. This is pure logic in `core`, fed by the shim's `displays()` and `frame(win)`.

## Context (self-contained)

- After 001, the `Platform` trait exposes `frame(win)` and `displays()` (Vec of work-area `Rect`s), and `work_area(win)`.
- The dev machine has a secondary display to the **left** of the primary at **negative** coordinates (see [[dev-multi-monitor-setup]]); the coordinate flip in the macOS shim already accounts for the primary screen height. Do not regress that.
- "Largest overlap" = for each display work area, compute the area of its intersection with the window frame; choose the max. Ties: prefer the display whose work area contains the window's center, else the first.

## What to build

1. **A pure `core` function** `display_for(window_frame: Rect, displays: &[Rect]) -> Rect` (or an index) implementing largest-overlap selection, with tie-breaking as above. Zero-overlap (offscreen) windows fall back to the display containing the window center, else the primary/first.
2. **Wire the snap path** to use it: read the focused window's `frame`, choose its display work area via `display_for`, and compute the target rectangle against **that** work area instead of the main display.
3. **Keep the macOS shim honest**: `displays()` returns each screen's `visibleFrame` converted into the same top-left, primary-height-flipped space the existing code uses.

## Open decisions (recommendation)

- **Overlap tie-breaking** — center-containment vs first-listed. _Recommendation:_ center-containment first, then first-listed. Document it in a code comment; it rarely matters but should be deterministic.

## Acceptance criteria

- [ ] `display_for` is a pure function with unit tests covering: window fully on one display; window straddling two (majority wins); window offscreen (center fallback); a display at negative origin.
- [ ] A window on the secondary (left, negative-origin) display snaps to a half **of that display**, not the primary.
- [ ] The primary-screen-height coordinate flip is unchanged; no regression on single-monitor.

## Testing

- Unit: `display_for` with synthetic display arrays (including negative origins and mixed sizes).
- Manual (multi-monitor, per [[dev-multi-monitor-setup]]): focus a window on the left secondary display, press `⌃⌥←` → snaps to the left half of the secondary display.

## LLM prompt

```text
You are implementing issue 003 for JC Grid Manager. Read docs/idea.md §5.3/§7/§11 and
docs/issues/003-display-selection-largest-overlap.md. Issue 001 (shared core + Platform trait)
is already done.

Task: Add a pure `core` function that picks the display a window is on by largest overlap of the
window frame with each display's work area (tie-break: display containing the window center, else
first). Wire the snap path to compute target rectangles against the chosen display's work area
instead of always using the main display. Keep the macOS coordinate flip intact.

Definition of done: unit tests for the overlap function pass (including negative-origin displays and
straddling windows); a window on a secondary, negative-origin display snaps within that display.

Constraints: pure logic goes in `core`; only display enumeration/frame reads touch the shim. Do not
change the size→position→size recipe. This does NOT include the Next/Previous Display command (issue 019).

Bookkeeping (required): record START now in the Implementation log; add FINISH + DURATION when done;
write the Implementation summary; do NOT git commit; refine the Suggested commit message; the user commits.
```

## Implementation log (fill this in)

- **Started:** 2026-07-08 17:27 WIB
- **Finished:** 2026-07-08 17:38 WIB
- **Duration:** ~11m

## Implementation summary

Replaced always-main snapping with per-window display selection.

**What changed**
- `core/geometry.rs`: added the pure `display_for(window, displays) -> Rect` — the display
  whose work area has the largest overlap with the window; ties (and offscreen windows) break
  toward the display containing the window's center, then the first display; empty list returns
  the window unchanged. Plus `overlap_area` and `Rect::center`/`contains` helpers. 5 unit tests:
  fully-on-one, straddle-majority, equal-overlap-center-tiebreak, offscreen→first, and a
  negative-origin secondary (the dev multi-monitor setup).
- `platform/macos.rs`: gave the shim real `frame()` (reads `kAXPosition`/`kAXSize` via
  `AXValueGetValue`) and `displays()` (every `NSScreen.visibleFrame`, flipped into the shared
  top-left space), and made `work_area(win) = display_for(frame(win), displays())`.
- The snap path in `platform/mod.rs` is **unchanged** — it already calls `work_area(win)`, so
  simply upgrading that impl makes every snap target the window's actual display. Single-monitor
  is unaffected (one display → `display_for` returns it), and the primary-height flip is intact.

**Key decisions**
- Display selection lives in `core` (pure, tested); the shim only supplies raw `frame` +
  `displays` and calls the selector. This also lets the Windows shim (025) use its native
  `MonitorFromWindow` instead, without duplicating the algorithm.
- `work_area` delegating to `display_for` keeps the trait stable and the snap orchestration
  untouched.

**Incidental hygiene:** the new `msg_send!` call sites grew the `objc` 0.2 macro's
`cargo-clippy` cfg-lint noise (8→12). Declared that cfg value known via a `[lints.rust]`
`check-cfg` in `Cargo.toml`, so `cargo clippy` is now down to a single unfixable transitive-dep
note (`block v0.1.6`) and stays meaningful for future macOS slices.

**Verification:** `cargo test` → 19 pass (5 new). `cargo clippy` → clean (one dependency note).
The multi-monitor runtime check (focus a window on the left/negative-origin secondary, press
`⌃⌥←` → snaps to *that* display's left half) is a user-side manual smoke test.

**Follow-ups:** 019 (Next/Previous Display) builds on `displays()`; 005's state machine uses
`frame()` and the "same display" continue-cycle check.

## Suggested commit message

```
feat(core): select target display by largest overlap

Snap against the work area of the display the focused window actually
occupies (largest frame overlap, center-containment tie-break) instead of
always using the main display — fixes wrong-screen snaps on multi-monitor.
Adds a pure core::geometry::display_for with unit tests (incl. negative-origin
displays), gives the macOS shim real frame()/displays(), and routes work_area
through the core selector. Also silences the objc-macro cargo-clippy cfg lint
noise via a Cargo.toml check-cfg so clippy stays meaningful.
```
