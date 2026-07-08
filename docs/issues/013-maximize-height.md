# 013 — Maximize Height

| | |
|---|---|
| **Issue ID** | 013 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | `⌃⌥⇧↑` (Control+Option+Shift+Up) |
| **Source** | `docs/idea.md` §4 (Sizing → Maximize Height) |
| **Status** | ☑ Done |

## Summary

Add **Maximize Height**: make the window **full height** while **keeping** its current **width and horizontal position**. Unlike the pure placements, this needs the window's **current frame**, not just a work-area fraction.

## Geometry (mixed — current frame + work area)

Given work area `W` and current window frame `win`:
- `x = win.x` (unchanged), `w = win.w` (unchanged)
- `y = W.y` (top of work area), `h = W.h` (full work-area height)

## What to build

1. Add a `MaximizeHeight` handler that reads the current frame (`Platform::frame`) and the display work area, then sets `y`/`h` to full height while preserving `x`/`w`.
2. Default bind `⌃⌥⇧↑` already exists in 004 — make it functional (note the extra **Shift** modifier).
3. Clamp so the window stays within the work-area height.
4. Unit-test the mixed computation with a synthetic current frame + work area.

## Acceptance criteria

- [ ] `⌃⌥⇧↑` stretches the focused window to full work-area height without changing its width or x-position.
- [ ] Works when the window starts partially offscreen vertically (result is clamped to the work area).
- [ ] Unit test: given `win` and `W`, output has `win.x`/`win.w` preserved and full-height `y`/`h`.

## Testing

- Unit: mixed frame+work-area computation.
- Manual: size a window narrow, press `⌃⌥⇧↑` → it becomes full-height, same width/position.

## LLM prompt

```text
You are implementing issue 013 for JC Grid Manager. Read docs/idea.md §4 (Maximize Height) and
docs/issues/013-maximize-height.md. Issue 004 is done.

Task: Implement Maximize Height — full work-area height while KEEPING the window's current width and
x-position. This needs the current frame (Platform::frame) plus the work area, not just a fraction.
Default bind ⌃⌥⇧↑ (note the Shift) already exists; make it work. Clamp to the work area. Unit-test the
mixed computation.

Definition of done: acceptance criteria pass; width/x preserved, height maximized; tests green.

Constraints: do not alter width or x. Scope is only Maximize Height.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary;
do NOT git commit; refine the Suggested commit message; the user commits.
```

## Implementation log (fill this in)

- **Started:** 2026-07-08 18:38 WIB
- **Finished:** 2026-07-08 18:40 WIB
- **Duration:** ~2m

## Implementation summary

First current-frame-relative action, so `target_for`'s `_current` param became `current`. Added
`MaximizeHeight => Rect(current.x, work.y, current.w, work.h)` — full work-area height, top-
aligned, width + x preserved. Because y/h are set to the work area's own values, a vertically-
offscreen window is inherently clamped fully on-screen. Bind `⌃⌥⇧↑` (with the Shift extra, from
004) now functional. `cargo test` → 36 pass.

## Suggested commit message

```
feat(core): add Maximize Height action (⌃⌥⇧↑)

Stretch the window to full work-area height while preserving its current
width and horizontal position.
```
