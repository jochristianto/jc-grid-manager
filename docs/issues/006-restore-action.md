# 006 — Restore action

| | |
|---|---|
| **Issue ID** | 006 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 005 |
| **Blocks** | — (exposed in the tray menu, 024) |
| **Default shortcut** | `⌃⌥⌫` (Control+Option+Delete) |
| **Source** | `docs/idea.md` §4 (Sizing → Restore), §7 (Restore) |
| **Status** | ☐ Not started |

## Summary

Add the **Restore** command: return the focused window to the **baseline** frame captured before its first snap in the current run, then **clear** the remembered record (§7). Small slice on top of the state machine from 005.

## Context (self-contained)

- The restore baseline is captured on a *fresh grab* by the state machine (005) and represents the window's frame before snapping started.
- If there is no baseline for the focused window (nothing snapped, or the record was invalidated), Restore is a no-op. It should signal "nothing to do" gracefully (a soft beep once 021 lands; until then, log).
- Restore uses the same `set_frame` shim path; after applying, **clear** the record so a subsequent directional press starts a fresh cycle.

## What to build

1. Implement the `Restore` action handler: read the current window's baseline from the state record; if present, `set_frame` back to it and clear the record.
2. Register/confirm the `⌃⌥⌫` default bind (the entry already exists in the 004 table; this makes it functional).
3. Handle the no-baseline case gracefully (log now; beep after 021).

## Acceptance criteria

- [ ] After snapping a window (e.g. `⌃⌥←`, maybe cycling), pressing `⌃⌥⌫` returns it to its pre-snap size **and** position.
- [ ] After a successful restore, the record is cleared (next `⌃⌥←` starts fresh at ½).
- [ ] Restore with no baseline is a graceful no-op (no crash; logs, and will beep after 021).
- [ ] Unit test: given a state record with a baseline, the Restore transition returns the baseline frame and empties the record.

## Testing

- Unit: baseline present → returns baseline + clears; baseline absent → no-op signal.
- Manual: snap a window, cycle it, press `⌃⌥⌫` → it returns to where/what it was before the first snap.

## LLM prompt

```text
You are implementing issue 006 for JC Grid Manager. Read docs/idea.md §4 (Restore), §7 and
docs/issues/006-restore-action.md. Issue 005 (state machine + baseline) is done.

Task: Implement the Restore action (⌃⌥⌫): move the focused window back to the baseline frame the state
machine captured before its first snap, then clear the record. If there is no baseline, do nothing
gracefully (log for now; a soft beep arrives in issue 021). Add a unit test for the restore transition.

Definition of done: acceptance criteria pass; restoring returns the window to its exact pre-snap frame
and resets the cycle; no-baseline case is a safe no-op.

Constraints: reuse the baseline exposed by issue 005 — do not add new tracking. Keep the decision logic
in core (unit-testable).

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
feat(core): add Restore action (⌃⌥⌫)

Return the focused window to its pre-snap baseline and clear the snap record.
No-op with a graceful signal when there is no baseline.
```
