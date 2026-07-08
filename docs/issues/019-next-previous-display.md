# 019 — Next / Previous Display

| | |
|---|---|
| **Issue ID** | 019 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 003, 004 |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | Next `⌃⌥⌘→`, Previous `⌃⌥⌘←` (Windows: **Ctrl+Alt+Win+←/→**) |
| **Source** | `docs/idea.md` §4 (Displays), §5.4, §11 |
| **Status** | ☑ Done |

## Summary

Add **Next / Previous Display**: move the window to the adjacent display, **preserving its relative size/position**. Because geometry is expressed as fractions of a work area (§5.3), this is clean: measure the window's fractional rect within its **current** display's work area, then apply the **same fractions** to the **target** display's work area.

## Geometry (fraction transfer)

1. `src = display_for(win)` (issue 003).
2. `frac = (win relative to src work area)` → `((win.x-src.x)/src.w, (win.y-src.y)/src.h, win.w/src.w, win.h/src.h)`.
3. `dst = next/previous display` in a stable order (see decision).
4. New frame = `frac` applied to `dst` work area (clamp into `dst`).

## What to build

1. `NextDisplay` / `PreviousDisplay` handlers using the fraction-transfer above; reuse `display_for` (003) and the geometry helpers.
2. Define a **stable display ordering** (see decision) so "next/previous" is deterministic; wrap around at the ends.
3. Default binds `⌃⌥⌘→` / `⌃⌥⌘←` already exist in 004 (Command = Win on Windows) — make them functional.
4. Unit-test the fraction transfer between two synthetic displays (including different sizes and a negative-origin display).

## Open decisions (recommendation)

- **Display order** — by x-origin (left→right) vs OS enumeration order. _Recommendation:_ sort by work-area origin (x, then y); left→right is the least surprising "next." Single-display → no-op (graceful; beep after 021).
- **Windows shortcut conflict** — `Ctrl+Alt+Win+←/→` brushes Windows' own Win+Arrow snapping (§5.4/§11). _Recommendation:_ ship the default; note in the config UI (029) that it may need rebinding on Windows. Actual verification happens in issue 027.

## Acceptance criteria

- [ ] With ≥2 displays, `⌃⌥⌘→` / `⌃⌥⌘←` move the window to the adjacent display, preserving its relative size/position (a left-half window becomes a left-half window on the new display).
- [ ] Deterministic wrap-around ordering; single-display is a graceful no-op.
- [ ] Unit test asserts the fraction transfer across two differently-sized displays.

## Testing

- Unit: fraction transfer between synthetic displays.
- Manual (multi-monitor, [[dev-multi-monitor-setup]]): snap a window to a half, press `⌃⌥⌘→` → same relative half on the next display.

## LLM prompt

```text
You are implementing issue 019 for JC Grid Manager. Read docs/idea.md §4 (Displays), §5.4, §11 and
docs/issues/019-next-previous-display.md. Issues 003 (display selection) and 004 (registry) are done.

Task: Implement Next/Previous Display — move the window to the adjacent display preserving its relative
size/position by transferring its fractional rect from the source work area to the destination work area.
Reuse display_for (003). Define a deterministic left→right display order with wrap-around; single-display
is a graceful no-op. Default binds ⌃⌥⌘→/← already exist (Command→Win on Windows); make them work.
Unit-test the fraction transfer across differently-sized displays.

Definition of done: acceptance criteria pass; relative placement is preserved across displays; tests green.

Constraints: reuse the fraction-based geometry — do not hardcode pixel offsets. Windows shortcut-conflict
verification is issue 027, not here. Scope is only Next/Previous Display.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary;
do NOT git commit; refine the Suggested commit message; the user commits.
```

## Implementation log (fill this in)

- **Started:** 2026-07-08 18:53 WIB
- **Finished:** 2026-07-08 18:58 WIB
- **Duration:** ~5m

## Implementation summary

Last action slice — completes the fan-out. Added `display_move(win, src, displays, forward)`:
orders displays left→right by work-area origin (x, then y), finds the window's current display,
picks the next/previous (wrapping), and transfers the window's fractional rect from the source
work area to the destination (clamped into it). NextDisplay/PreviousDisplay arms call it;
`⌃⌥⌘→`/`⌃⌥⌘←` (from 004) now functional.

**Display order (was a documented recommendation):** used left→right by origin — the least-
surprising "next." Single-display is a graceful no-op (returns None). The Windows `Ctrl+Alt+Win`
conflict noted in §5.4 is verified in issue 027, not here.

Un-underscored `target_for`'s `displays` param and — now that every action has an arm — replaced
the `_ => None` fallback with an explicit `Restore => None` (Restore is handled by the dispatcher),
making `target_for` **exhaustive over `Action`** so a future variant can't be silently dropped.
Also updated the dispatcher's no-op log (an action that produced no move now reads "nothing to do"
rather than "not implemented"; a soft beep replaces it in 021).

Tests: relative-rect transfer to a differently-sized display, wrap-around, single-display no-op.
`cargo test` → 47 pass. **Action fan-out (007–019) complete.**

## Suggested commit message

```
feat(core): add Next/Previous Display actions (⌃⌥⌘←/→)

Move the window to the adjacent display, preserving relative size/position by
transferring its fractional rect between work areas.
```
