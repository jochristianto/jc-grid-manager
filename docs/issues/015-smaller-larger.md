# 015 — Smaller / Larger

| | |
|---|---|
| **Issue ID** | 015 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | Smaller `⌃⌥-`, Larger `⌃⌥=` |
| **Source** | `docs/idea.md` §4 (Sizing → Smaller / Larger) |
| **Status** | ☑ Done |

## Summary

Add **Smaller** and **Larger**: shrink/grow the window by **~5% of the screen** around the window's **center**. Smaller **floors** at a minimum; Larger **caps** at full screen. The step and minimum are **configurable**. These are inverse operations sharing one clamp routine, and they mutate the **current** frame.

## Geometry (mixed — current frame + work area)

Given work area `W`, current frame `win`, step `s` (default 5% of work area):
- `Larger`: grow width by `s·W.w` and height by `s·W.h`, re-center around `win`'s center, then clamp so the window fits within `W` (cap at full work area).
- `Smaller`: shrink by the same amounts, re-center, floor at the minimum size (configurable; default e.g. 20% of work area or a fixed min like 400×300 — see decision).
- Re-centering: keep the window's center fixed, adjust origin so the resized window stays centered on that point (then clamp into `W`).

## What to build

1. A shared `resize_around_center(win, W, delta) -> Rect` helper with cap (full work area) and floor (minimum).
2. `Larger` / `Smaller` handlers calling it with `+step` / `-step`.
3. Config hooks for `resize_step` (default 0.05) and `min_size` (leave a named default for 020 to wire).
4. Default binds `⌃⌥-` / `⌃⌥=` already exist in 004 — make them functional.
5. Unit-test grow/shrink, the cap, the floor, and repeated presses.

## Open decisions (recommendation)

- **Step basis** — 5% of the work area (both axes) vs 5% of each axis independently. _Recommendation:_ 5% of the corresponding work-area dimension per axis (so it feels even on wide monitors).
- **Minimum size** — fixed px vs fraction of work area. _Recommendation:_ a fraction (e.g. floor at ~20% width/height) so it scales with display; expose `min_size` for 020. **Confirm the default with the user.**

## Acceptance criteria

- [ ] `⌃⌥=` grows and `⌃⌥-` shrinks the window ~5% each press, keeping it centered on its prior center.
- [ ] Larger never exceeds the work area; Smaller never goes below the configured minimum.
- [ ] Repeated presses are stable (no drift off-center, no runaway).
- [ ] Unit tests cover grow, shrink, cap, floor, and a repeat sequence.

## Testing

- Unit: grow/shrink/cap/floor + repeats.
- Manual: hold-tap `⌃⌥=` / `⌃⌥-` and watch the window scale around its center within bounds.

## LLM prompt

```text
You are implementing issue 015 for JC Grid Manager. Read docs/idea.md §4 (Smaller/Larger) and
docs/issues/015-smaller-larger.md. Issue 004 is done.

Task: Implement Smaller (⌃⌥-) and Larger (⌃⌥=) as inverse operations that resize the window by a
configurable step (default 5% of the work area per axis) around the window's center, capping at the
work area and flooring at a configurable minimum. Share one resize_around_center helper. Leave named
config hooks (resize_step, min_size) for issue 020. Default binds already exist; make them work.
Unit-test grow/shrink/cap/floor and repeated presses. Confirm the minimum-size default with the user.

Definition of done: acceptance criteria pass; both keys resize around center within bounds; tests green.

Constraints: keep the center fixed; clamp into the work area. Scope is only Smaller/Larger.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary
(note your chosen minimum-size default); do NOT git commit; refine the Suggested commit message; the
user commits.
```

## Implementation log (fill this in)

- **Started:** 2026-07-08 18:42 WIB
- **Finished:** 2026-07-08 18:47 WIB
- **Duration:** ~5m

## Implementation summary

Added Smaller/Larger via a shared `resize_around_center(win, work, dw, dh, min_w, min_h)` helper:
resize by ±`RESIZE_STEP` (5% of the work area, per axis) keeping the window's center fixed, then
clamp — size floored at `MIN_SIZE_FRACTION` and capped at the work area, origin kept on-screen.

**Minimum-size default (was flagged "confirm with user"):** took the recommended default — a
**fraction, 20% of the work area** (`MIN_SIZE_FRACTION = 0.2`), so it scales with display size.
Both `RESIZE_STEP` and `MIN_SIZE_FRACTION` are named `pub const`s in `core/geometry.rs` (config
hooks for 020). Change the 0.2 if you'd prefer a different floor.

Binds `⌃⌥-` / `⌃⌥=` (from 004) now functional. Tests: grow-around-center, cap at the work area,
floor at the minimum, and a 40-press repeat that converges to the cap with no runaway.
`cargo test` → 42 pass.

## Suggested commit message

```
feat(core): add Smaller/Larger actions (⌃⌥- / ⌃⌥=)

Resize the window by a configurable step around its center, capped at the
work area and floored at a configurable minimum.
```
