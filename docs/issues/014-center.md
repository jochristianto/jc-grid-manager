# 014 — Center

| | |
|---|---|
| **Issue ID** | 014 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | `⌃⌥C` (Control+Option+C) |
| **Source** | `docs/idea.md` §4 (Sizing → Center) |
| **Status** | ☑ Done |

## Summary

Add **Center**: center the window **at its current size** within its display's work area. Keeps size, changes only position.

## Geometry (mixed — current size + work area)

Given work area `W` and current window frame `win`:
- `w = win.w`, `h = win.h` (unchanged)
- `x = W.x + (W.w - win.w) / 2`
- `y = W.y + (W.h - win.h) / 2`
- If the window is larger than the work area in a dimension, clamp origin to the work-area edge (don't push it off-screen).

## What to build

1. Add a `Center` handler that reads the current frame + work area and repositions without resizing.
2. Default bind `⌃⌥C` already exists in 004 — make it functional.
3. Unit-test the centering math, including the larger-than-work-area clamp.

## Acceptance criteria

- [ ] `⌃⌥C` centers the focused window on its display without changing its size.
- [ ] A window wider/taller than the work area clamps to the edge rather than going off-screen.
- [ ] Unit test asserts centered origin for a normal case and the clamped case.

## Testing

- Unit: centering + clamp.
- Manual: move a window to a corner, press `⌃⌥C` → it centers at the same size.

## LLM prompt

```text
You are implementing issue 014 for JC Grid Manager. Read docs/idea.md §4 (Center) and
docs/issues/014-center.md. Issue 004 is done.

Task: Implement Center — reposition the window to the center of its display work area at its CURRENT
size (no resize). Use the current frame + work area. Clamp origin if the window is larger than the work
area. Default bind ⌃⌥C already exists; make it work. Unit-test the math incl. the clamp case.

Definition of done: acceptance criteria pass; window centers without resizing; tests green.

Constraints: never change the window size. Scope is only Center.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary;
do NOT git commit; refine the Suggested commit message; the user commits.
```

## Implementation log (fill this in)

- **Started:** 2026-07-08 18:40 WIB
- **Finished:** 2026-07-08 18:42 WIB
- **Duration:** ~2m

## Implementation summary

Added `Center` — keeps the current size and centers it in the work area:
`x = work.x + max((work.w - current.w)/2, 0)`, likewise for y. The `max(…, 0)` clamps a window
larger than the work area to the top/left edge instead of pushing it off-screen. Bind `⌃⌥C` (from
004) now functional. Tests cover the normal centering and the oversized clamp. `cargo test` → 38 pass.

## Suggested commit message

```
feat(core): add Center action (⌃⌥C)

Center the window at its current size within the display work area, clamping
origin when the window exceeds the work area.
```
