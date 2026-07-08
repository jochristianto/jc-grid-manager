# 008 — Thirds (First / Center / Last)

| | |
|---|---|
| **Issue ID** | 008 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 (soft: 005 for the shared cycle table) |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | First `⌃⌥D`, Center `⌃⌥F`, Last `⌃⌥G` |
| **Source** | `docs/idea.md` §4 (Thirds) |
| **Status** | ☐ Not started |

## Summary

Add the three **direct** thirds placements — full-height columns at the left, center, and right third of the display. These are direct jumps (each key goes straight to its third); the ½→⅔→⅓ *cycling* lives on the halves keys (issue 005), not here.

## Geometry (fraction of work area)

- First Third → `(0.0, 0.0, 1/3, 1.0)`
- Center Third → `(1/3, 0.0, 1/3, 1.0)`
- Last Third → `(2/3, 0.0, 1/3, 1.0)`

## What to build

1. Add the three fractions to the core action table for `FirstThird` / `CenterThird` / `LastThird`.
2. Their default binds (`⌃⌥D/F/G`) already exist in the 004 table — this makes them functional.
3. Unit-test each conversion (use exact thirds; watch for float rounding — `1/3` etc.).

## Acceptance criteria

- [ ] `⌃⌥D` / `⌃⌥F` / `⌃⌥G` snap the focused window to the left / center / right third, full height, on its own display.
- [ ] Unit tests assert the three target rects (tolerant of float rounding).
- [ ] Columns tile with no gap/overlap (First end == Center start, etc.) within tolerance.

## Testing

- Unit: three fraction conversions.
- Manual: press `⌃⌥D`, `⌃⌥F`, `⌃⌥G` → left/center/right thirds.

## LLM prompt

```text
You are implementing issue 008 for JC Grid Manager. Read docs/idea.md §4 (Thirds) and
docs/issues/008-thirds.md. Issue 004 is done.

Task: Implement First/Center/Last Third as direct full-height column placements at fractions
(0,0,1/3,1), (1/3,0,1/3,1), (2/3,0,1/3,1). Their default binds ⌃⌥D/F/G already exist; make them work.
Unit-test the conversions (mind float rounding so columns tile cleanly).

Definition of done: acceptance criteria pass; the three keys snap to their thirds; tests green.

Constraints: these are DIRECT placements, not the halves cycle. Scope is only the three single thirds
(the two-thirds placements are issue 009).

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
feat(core): add First/Center/Last Third actions (⌃⌥D/F/G)

Direct full-height third-columns at (0,0,1/3,1), (1/3,0,1/3,1), (2/3,0,1/3,1).
```
