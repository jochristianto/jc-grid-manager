# 017 — Fourths

| | |
|---|---|
| **Issue ID** | 017 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | none (menu-only sub-menu; user may bind) |
| **Source** | `docs/idea.md` §4 (Sub-menus → Fourths) |
| **Status** | ☐ Not started |

## Summary

Add the **Fourths** sub-menu: four **full-height columns**, each ¼ of the width (First / Second / Third / Last Fourth).

## Geometry (fraction of work area)

- First Fourth → `(0.0, 0.0, 0.25, 1.0)`
- Second Fourth → `(0.25, 0.0, 0.25, 1.0)`
- Third Fourth → `(0.5, 0.0, 0.25, 1.0)`
- Last Fourth → `(0.75, 0.0, 0.25, 1.0)`

## What to build

1. Add the four fractions to the core action table for the fourths.
2. No default binds — surfaced via the tray sub-menu (024). Add the four `Action` variants if not already present from 004.
3. Unit-test the four conversions and that they tile the width.

## Acceptance criteria

- [ ] The four fourths snap to their quarter-width full-height columns on the window's display.
- [ ] Columns tile the width with no gaps/overlaps within tolerance.
- [ ] Unit tests assert the four target rects.

## Testing

- Unit: four fraction conversions.
- Manual: trigger each via menu/bind → correct quarter-width column.

## LLM prompt

```text
You are implementing issue 017 for JC Grid Manager. Read docs/idea.md §4 (Fourths) and
docs/issues/017-fourths.md. Issue 004 is done.

Task: Implement the four Fourths as full-height quarter-width columns at (0,0,.25,1), (.25,0,.25,1),
(.5,0,.25,1), (.75,0,.25,1). No default shortcuts (menu-only). Add the Action variants if missing.
Unit-test the conversions and column tiling.

Definition of done: acceptance criteria pass; tests green.

Constraints: scope is only the four Fourths.

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
feat(core): add Fourths actions

Four full-height quarter-width columns (first/second/third/last).
```
