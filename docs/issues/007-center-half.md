# 007 — Center Half

| | |
|---|---|
| **Issue ID** | 007 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | none (menu-only; user may bind) |
| **Source** | `docs/idea.md` §4 (Halves → Center Half) |
| **Status** | ☐ Not started |

## Summary

Add the **Center Half** action: a centered column, **half the screen width, full height**. Static placement — no cycling.

## Geometry (fraction of work area)

- Center Half → `(0.25, 0.0, 0.5, 1.0)`

## What to build

1. Add the `CenterHalf` geometry to the core action table (fraction above).
2. Ensure the dispatcher runs it (the `Action` variant exists from 004; it has no default bind, so it is exercised via the tray menu (024) or a user bind (020)).
3. Unit test the fraction → absolute conversion.

## Acceptance criteria

- [ ] `CenterHalf` produces `(0.25, 0, 0.5, 1)` of the focused window's display work area.
- [ ] Unit test asserts the target rect for a synthetic work area (including a negative-origin display).
- [ ] Invoking the action (via a temporary bind or the tray menu) centers a half-width, full-height window.

## Testing

- Unit: fraction conversion.
- Manual: bind temporarily or trigger via menu → window becomes a centered half-width, full-height column.

## LLM prompt

```text
You are implementing issue 007 for JC Grid Manager. Read docs/idea.md §4 and
docs/issues/007-center-half.md. Issue 004 (action registry) is done.

Task: Implement the Center Half action — a centered column at fraction (0.25, 0, 0.5, 1) of the
window's display work area. Add it to the core geometry/action table and unit-test the conversion.
It has no default shortcut; it is triggered via the tray menu (issue 024) or a user bind.

Definition of done: acceptance criteria pass; unit test green.

Constraints: static placement, no cycling. Scope is only Center Half.

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
feat(core): add Center Half action

Centered half-width, full-height column at fraction (0.25, 0, 0.5, 1).
```
