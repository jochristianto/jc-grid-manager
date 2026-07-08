# 021 — Soft beep + stubborn-window handling

| | |
|---|---|
| **Issue ID** | 021 |
| **Layer** | core (backend, Rust) + per-OS beep |
| **Depends on** | 004 |
| **Blocks** | referenced by 006 (Restore no-op), 019 (single-display no-op), 027 (elevated windows) |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §4 ("Stubborn windows"), §8, §11 |
| **Status** | ☐ Not started |

## Summary

Make the app **best-effort and honest**: if a window can't be moved/resized (fixed-size, minimum-size, fullscreen, or an admin/elevated window on Windows), do what's possible and play a **soft beep** when **nothing** could be done. This turns silent failures into a clear signal.

## Context (self-contained)

- Today `set_frame` on macOS returns an error string when the AX set fails; those are logged. This slice classifies "nothing happened" and beeps.
- "Nothing could be done" means: the target action produced **no effective change** to the window frame (compare requested vs re-read actual, within tolerance) **or** the shim reported the window is unmanageable.
- The beep is the standard system alert sound: macOS `NSBeep()`; Windows `MessageBeep` (added properly in 027, but define the cross-OS hook now).
- Restore (006) and Next/Prev Display (019) already have "no-op" cases that should route through this beep.

## What to build

1. A small `notify_no_op()` (name your choice) that plays the system beep, with a per-OS implementation behind the `Platform` boundary (macOS now; Windows stub that 027 fills).
2. **Effect detection**: after an action's `set_frame`, re-read the actual frame; if it did not change meaningfully **and** the action intended a change, call the beep. (The state machine already re-reads the frame — reuse that read.)
3. Route existing no-op cases (Restore-with-no-baseline, single-display display-move) through the beep.
4. Optional: a debounce so a rapid key-repeat doesn't machine-gun the beep.

## Open decisions (recommendation)

- **What counts as "nothing done"** — strict (frame byte-identical) vs tolerant (within N px). _Recommendation:_ tolerant, reusing the state-machine tolerance, so a window that clamps to its min-size but *did* move isn't falsely flagged.
- **Beep on every partial failure vs only total failure** — §4 says beep when **nothing** could be done. _Recommendation:_ beep only on total no-op; log partials.

## Acceptance criteria

- [ ] Acting on a genuinely immovable/fixed-size window (that doesn't change at all) plays one soft beep.
- [ ] A window that partially complies (e.g. clamps to min size but moves) does **not** beep.
- [ ] Restore with no baseline and display-move with a single display beep instead of silently doing nothing.
- [ ] Rapid repeats don't spam the beep (debounced).
- [ ] The beep is behind the `Platform` boundary (macOS implemented; Windows stub present for 027).

## Testing

- Manual (macOS): try to snap a fixed-size window / a native-fullscreen window → soft beep; snap a normal window → no beep. Press `⌃⌥⌫` with nothing snapped → beep.

## LLM prompt

```text
You are implementing issue 021 for JC Grid Manager. Read docs/idea.md §4 (Stubborn windows), §8, §11 and
docs/issues/021-soft-beep-stubborn-windows.md. Issue 004 is done; 005/006/019 may be present.

Task: Add best-effort handling with a soft system beep when an action changes nothing. Implement effect
detection by comparing the requested frame with the re-read actual frame (reuse the state machine's
re-read, tolerant within N px). Play the system beep (macOS NSBeep now; add a Windows stub behind the
Platform boundary for issue 027). Route Restore-no-baseline and single-display display-move through the
beep. Debounce so key-repeat doesn't spam it.

Definition of done: acceptance criteria pass; immovable windows beep once, partial-comply windows don't,
no-op commands beep, repeats are debounced.

Constraints: beep only on TOTAL no-op (log partials). Keep the beep behind the Platform boundary. Scope is
only the beep + effect detection wiring.

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
feat(core): soft beep when an action can do nothing

Detect no-op actions by comparing requested vs re-read frame and play the
system beep (behind the Platform boundary), with debounce. Route Restore and
single-display display-move no-ops through it.
```
