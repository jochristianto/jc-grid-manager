# 007 — Center Half

| | |
|---|---|
| **Issue ID** | 007 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | none (menu-only; user may bind) |
| **Source** | `docs/idea.md` §4 (Halves → Center Half) |
| **Status** | ☑ Done |

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

- **Started:** 2026-07-08 18:10 WIB
- **Finished:** 2026-07-08 18:22 WIB
- **Duration:** ~12m

## Implementation summary

Center Half is trivial on its own; the substance here is the **shared geometry table** it
triggered (per issue 001's "every new action is a small addition to a geometry table"). 008–019
are now one-line arms.

**What changed**
- `core/geometry.rs`: added `target_for(action, step, current, work, displays) -> Option<Rect>`
  — the single place an action's target geometry lives. `None` = not implemented yet (dispatcher
  logs it). Implemented arms so far: the four halves (cycle via `step`) and **Center Half**
  `(0.25, 0, 0.5, 1)`. (`current`/`displays` params are `_`-prefixed until the computed/cross-
  display slices use them.)
- `core/state.rs`: generalized `next_target` — it now takes `cycle_len` + a target-computing
  closure and returns `Option<Rect>` (geometry stays in the table; the state machine stays pure
  and just decides step + baseline). A `None` target leaves the record untouched.
- `core/actions.rs`: added `Action::cycle_len()` (halves 3, else 1).
- `platform/mod.rs`: replaced the halves-only `snap_cycling` with `perform(action, state)` that
  drives **any** action through the table + state machine (`Ok(false)` = not implemented).
- `lib.rs`: `dispatch` now routes every non-Restore action through `perform` (one main-thread
  lock, Restore branch kept). **Net effect: Restore now undoes any action**, since they all
  capture a baseline.

**Verification:** `cargo test` → 31 pass (halves still cycle; Center Half fraction; unimplemented
→ None; the state transitions unchanged). `cargo clippy` → clean. Runtime (bind Center Half or
trigger via the tray in 024) is a user-side check.

**Follow-ups:** 008–018 add their `target_for` arms; 019 uses the `displays` param.

## Suggested commit message

```
feat(core): add geometry table + Center Half action

Introduce core::geometry::target_for — the single table mapping an action +
cycle step to its target rect — and drive every action through the §7 state
machine via platform::perform (generalizing the halves-only path), so Restore
now works on all actions. Add Center Half: a centered half-width, full-height
column at fraction (0.25, 0, 0.5, 1). cargo test: 31 passing.
```
