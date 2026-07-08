# 009 — Two-Thirds (First / Last)

| | |
|---|---|
| **Issue ID** | 009 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 (soft: 008) |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | First Two Thirds `⌃⌥E`, Last Two Thirds `⌃⌥T` |
| **Source** | `docs/idea.md` §4 (Thirds) |
| **Status** | ☐ Not started |

## Summary

Add the two **two-thirds** placements — full-height columns two-thirds of the display width, anchored left or right.

## Geometry (fraction of work area)

- First Two Thirds → `(0.0, 0.0, 2/3, 1.0)`
- Last Two Thirds → `(1/3, 0.0, 2/3, 1.0)`

## What to build

1. Add the two fractions to the core action table for `FirstTwoThirds` / `LastTwoThirds`.
2. Their default binds (`⌃⌥E/T`) already exist in the 004 table — make them functional.
3. Unit-test both conversions.

## Acceptance criteria

- [ ] `⌃⌥E` / `⌃⌥T` snap the window to the left / right two-thirds column, full height, on its display.
- [ ] `First Two Thirds` shares its right edge with `Last Third` (from 008), and `Last Two Thirds` shares its left edge with `First Third`, within tolerance.
- [ ] Unit tests assert both target rects.

## Testing

- Unit: two fraction conversions.
- Manual: press `⌃⌥E`, `⌃⌥T` → left/right two-thirds columns.

## LLM prompt

```text
You are implementing issue 009 for JC Grid Manager. Read docs/idea.md §4 (Thirds) and
docs/issues/009-two-thirds.md. Issue 004 is done.

Task: Implement First/Last Two Thirds at fractions (0,0,2/3,1) and (1/3,0,2/3,1). Their default binds
⌃⌥E/T already exist; make them work. Unit-test both conversions and verify edges tile with the single
thirds (issue 008) within tolerance.

Definition of done: acceptance criteria pass; both keys snap correctly; tests green.

Constraints: scope is only the two two-thirds placements.

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
feat(core): add First/Last Two Thirds actions (⌃⌥E/T)

Full-height two-thirds columns at (0,0,2/3,1) and (1/3,0,2/3,1).
```
