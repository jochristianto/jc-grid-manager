# 005 — Snap state machine (cycling + restore baseline)

| | |
|---|---|
| **Issue ID** | 005 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 (soft: 003) |
| **Blocks** | 006 (Restore), and the "cycling" behavior of directional actions (008/009 benefit) |
| **Default shortcut** | exercised via `⌃⌥←` / `⌃⌥→` (repeat to cycle) |
| **Source** | `docs/idea.md` §4 ("Cycling"), §7 (the whole state machine), §10 |
| **Status** | ☐ Not started |

## Summary

Build the part that makes it *feel* like Rectangle (§7): repeating a directional shortcut **cycles** through sizes (Left → ½ → ⅔ → ⅓), and the app remembers a **restore baseline** so a later Restore (issue 006) can undo. This is pure logic — a small remembered record + fresh-grab-vs-continue rules — unit-testable with no real windows.

## Context (self-contained)

- The remembered record (§7): last action, cycle step, the **frame we actually set**, and the **restore baseline** (the frame before the first snap of a run). Start with a single record; a small MRU list is a later upgrade.
- **Continue the cycle** only if all hold: same window as last time **and** its current frame still ≈ the frame we last set **and** the action matches **and** it's on the same display. Otherwise it's a **fresh grab**.
- **Fresh grab** → capture the current frame as the restore baseline, cycle step = 0. **Continue** → advance the step, keep the baseline.
- After `set_frame`, **re-read the actual resulting frame** and store *that* (terminals/min-size windows that don't land exactly must still cycle correctly).
- Invalidation is automatic: if the user drags the window, its frame no longer matches what we set, so the next press starts fresh. No OS move/resize listeners.
- "≈" needs a tolerance (a few px) because `set_frame` results are not always exact.
- Cycle tables (fractions of work area):
  - Left: `½ (0,0,.5,1)` → `⅔ (0,0,2/3,1)` → `⅓ (0,0,1/3,1)`
  - Right: `½ (.5,0,.5,1)` → `⅔ (1/3,0,2/3,1)` → `⅓ (2/3,0,1/3,1)`

## What to build

1. **The state record + transition function** in `core/state.rs`: given `(action, current_frame, work_area, display_id)`, decide fresh-grab vs continue, produce the next cycle step, and return the target fraction. Pure; no I/O.
2. **Frame-match tolerance** helper (`≈` within N px, default a few px; make it a small constant).
3. **Wire directional halves to cycle**: Left/Right now cycle `½ → ⅔ → ⅓` on repeat; re-read actual frame after set and store it.
4. **Expose the baseline** so issue 006 (Restore) can read and clear it.

## Open decisions (recommendation)

- **Do Top/Bottom cycle too?** §4 only annotates Left/Right with the ½→⅔→⅓ cycle; Top/Bottom are blank. _Recommendation:_ make the state machine **general** (a cycle list per action) and give Top/Bottom the vertical analogue (`½ → ⅔ → ⅓` heights), matching Rectangle. Ship it behind the same mechanism so it's a one-line table change if you'd rather leave them single-step. **Question for the user:** cycle all four halves (recommended) or only Left/Right?
- **Match tolerance** — 1px vs a few px. _Recommendation:_ ~5px; document why (set_frame rounding + min-size windows).
- **State scope** — single record vs MRU list. _Recommendation:_ single record now (§7 says upgradeable later).

## Acceptance criteria

- [ ] `core/state.rs` holds a pure transition function with **no window I/O**, fully unit-tested.
- [ ] Unit tests cover: fresh grab (baseline captured, step 0); continue (same window/action/display, frame ≈ last set → advance); invalidation (frame moved by "user" → fresh); action change → fresh; display change → fresh.
- [ ] Pressing `⌃⌥←` repeatedly cycles the focused window ½ → ⅔ → ⅓ → ½ …; same for `⌃⌥→`.
- [ ] After each snap, the stored "last set frame" equals the **re-read** actual frame.
- [ ] The restore baseline is queryable/clearable for issue 006.

## Testing

- Unit: scripted `(action, current_frame)` sequences against a fake work area asserting cycle step, target rect, and baseline (this is the §10 core-test pattern).
- Manual: repeat `⌃⌥←` and watch ½→⅔→⅓; drag the window, press again → starts fresh at ½.

## LLM prompt

```text
You are implementing issue 005 for JC Grid Manager. Read docs/idea.md §4 (Cycling), §7 (state machine),
§10 and docs/issues/005-snap-state-machine-cycling.md. Issue 004 (action registry/dispatcher) is done.

Task: Implement the snap state machine described in §7 as PURE logic in core/state.rs: a remembered
record (last action, cycle step, last-set frame, restore baseline), fresh-grab-vs-continue rules, and a
per-action cycle table. Wire the directional half actions to cycle ½ → ⅔ → ⅓ on repeat. After set_frame,
re-read the actual frame and store that. Expose the baseline for the Restore action (issue 006).

Before you start, confirm with the user (or note your default in the Implementation summary): should all
four halves cycle (recommended) or only Left/Right? Implement it as a general per-action cycle table
either way.

Definition of done: acceptance criteria pass; the state transition function is unit-tested with no real
windows; repeating ⌃⌥← / ⌃⌥→ cycles sizes; manual drag invalidates the cycle.

Constraints: no OS move/resize listeners — invalidation is by frame comparison only. Do not implement the
Restore *action* here (issue 006), just expose the baseline. Keep all decision logic in core (testable).

Bookkeeping (required): record START in the Implementation log now; add FINISH + DURATION; write the
Implementation summary (including your answer to the cycle-all-four question); do NOT git commit; refine
the Suggested commit message; the user commits.
```

## Implementation log (fill this in)

- **Started:** _<!-- -->_
- **Finished:** _<!-- -->_
- **Duration:** _<!-- -->_

## Implementation summary (fill this in)

_<!-- ... -->_

## Suggested commit message

```
feat(core): add snap state machine with size cycling and restore baseline

Repeating a directional shortcut cycles ½ → ⅔ → ⅓; a per-window baseline is
captured on fresh grab and re-read after each set_frame. Pure, unit-tested;
invalidation is automatic via frame comparison (no OS listeners).
```
