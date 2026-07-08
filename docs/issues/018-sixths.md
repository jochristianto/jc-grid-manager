# 018 — Sixths (3×2 grid)

| | |
|---|---|
| **Issue ID** | 018 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | none (menu-only sub-menu; user may bind) |
| **Source** | `docs/idea.md` §4 (Sub-menus → Sixths [review]) |
| **Status** | ☐ Not started |

## Summary

Add the **Sixths** sub-menu: a **3-across × 2-down grid** = six cells, each **⅓ wide, ½ tall**. (The design review fixed this shape explicitly — six cells in a 3×2 grid, not vertical sixths.)

## Geometry (fraction of work area)

Top row (`y=0`, `h=0.5`), bottom row (`y=0.5`, `h=0.5`); columns at `x ∈ {0, 1/3, 2/3}`, `w=1/3`:
- Top-Left `(0, 0, 1/3, 0.5)` · Top-Center `(1/3, 0, 1/3, 0.5)` · Top-Right `(2/3, 0, 1/3, 0.5)`
- Bottom-Left `(0, 0.5, 1/3, 0.5)` · Bottom-Center `(1/3, 0.5, 1/3, 0.5)` · Bottom-Right `(2/3, 0.5, 1/3, 0.5)`

## What to build

1. Add the six fractions to the core action table for the sixths cells.
2. No default binds — surfaced via the tray sub-menu (024). Add the six `Action` variants if not already present from 004.
3. Unit-test the six conversions and that they tile the work area.

## Open decisions (recommendation)

- **Naming** — top/bottom + left/center/right vs numbered 1–6. _Recommendation:_ positional names (Top-Left … Bottom-Right); clearer in the menu than numbers.

## Acceptance criteria

- [ ] The six sixths snap to their ⅓-wide, ½-tall cells on the window's display.
- [ ] The six cells tile the work area with no gaps/overlaps within tolerance.
- [ ] Unit tests assert the six target rects.

## Testing

- Unit: six fraction conversions + tiling.
- Manual: trigger each via menu/bind → correct grid cell.

## LLM prompt

```text
You are implementing issue 018 for JC Grid Manager. Read docs/idea.md §4 (Sixths) and
docs/issues/018-sixths.md. Issue 004 is done.

Task: Implement Sixths as a 3×2 grid of six cells, each ⅓ wide and ½ tall (fractions listed in the issue).
No default shortcuts (menu-only). Use positional names (Top-Left … Bottom-Right). Add the Action variants
if missing. Unit-test the six conversions and full tiling.

Definition of done: acceptance criteria pass; tests green.

Constraints: it is a 3-across × 2-down grid (per the design review), NOT vertical sixths. Scope is only Sixths.

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
feat(core): add Sixths actions (3×2 grid)

Six cells, each ⅓ wide and ½ tall (top/bottom × left/center/right).
```
