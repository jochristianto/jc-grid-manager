# 010 — Corners / Quarters

| | |
|---|---|
| **Issue ID** | 010 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | TL `⌃⌥U`, TR `⌃⌥I`, BL `⌃⌥J`, BR `⌃⌥K` |
| **Source** | `docs/idea.md` §4 (Corners / quarters) |
| **Status** | ☐ Not started |

## Summary

Add the four **quarter** placements (screen corners), each half width × half height.

## Geometry (fraction of work area)

- Top Left → `(0.0, 0.0, 0.5, 0.5)`
- Top Right → `(0.5, 0.0, 0.5, 0.5)`
- Bottom Left → `(0.0, 0.5, 0.5, 0.5)`
- Bottom Right → `(0.5, 0.5, 0.5, 0.5)`

## What to build

1. Add the four fractions to the core action table for `TopLeft` / `TopRight` / `BottomLeft` / `BottomRight`.
2. Default binds (`⌃⌥U/I/J/K`) already exist in 004 — make them functional.
3. Unit-test all four conversions.

## Acceptance criteria

- [ ] `⌃⌥U/I/J/K` snap the window to the correct corner quarter on its display.
- [ ] Unit tests assert the four target rects.
- [ ] The four quarters tile the work area with no gaps/overlaps within tolerance.

## Testing

- Unit: four fraction conversions.
- Manual: press each of `⌃⌥U/I/J/K` → correct corner.

## LLM prompt

```text
You are implementing issue 010 for JC Grid Manager. Read docs/idea.md §4 (Corners) and
docs/issues/010-corners-quarters.md. Issue 004 is done.

Task: Implement the four corner quarters at (0,0,.5,.5), (.5,0,.5,.5), (0,.5,.5,.5), (.5,.5,.5,.5).
Default binds ⌃⌥U/I/J/K already exist; make them work. Unit-test the four conversions and verify they
tile the work area.

Definition of done: acceptance criteria pass; four keys snap to corners; tests green.

Constraints: scope is only the four quarters.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary;
do NOT git commit; refine the Suggested commit message; the user commits.
```

## Implementation log (fill this in)

- **Started:** _<!-- -->_
- **Finished:** _<!-- -->_
- **Duration:** _<!-- -->_

## Implementation summary (fill this in)

_<!-- ... -->_

## Suggested commit message

```
feat(core): add corner quarter actions (⌃⌥U/I/J/K)

Top-left/right and bottom-left/right quarters at half width × half height.
```
