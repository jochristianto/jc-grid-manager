# 016 — Move to Edge

| | |
|---|---|
| **Issue ID** | 016 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | none (menu-only sub-menu; user may bind) |
| **Source** | `docs/idea.md` §4 (Sub-menus → Move to Edge) |
| **Status** | ☑ Done |

## Summary

Add the **Move to Edge** sub-menu: slide the window **flush to an edge** (Left / Right / Up / Down) **without resizing**. Position-only; keeps the current size.

## Geometry (mixed — current frame + work area)

Given work area `W`, current frame `win`:
- Left → `x = W.x`, y unchanged
- Right → `x = W.x + W.w - win.w`, y unchanged
- Up → `y = W.y`, x unchanged
- Down → `y = W.y + W.h - win.h`, x unchanged

## What to build

1. Four handlers `MoveToEdge{Left,Right,Up,Down}` that reposition (no resize) using current frame + work area.
2. No default binds — surfaced via the tray sub-menu (024) or user binds (020). The `Action` variants should exist in the 004 enum; if they were not added there, add them now.
3. Unit-test the four edge computations.

## Acceptance criteria

- [ ] Each Move-to-Edge action slides the window flush against the given edge without changing its size.
- [ ] Works from any starting position; result stays within the work area.
- [ ] Unit tests assert the four resulting origins.

## Testing

- Unit: four edge computations.
- Manual: trigger via menu/bind → window butts against the chosen edge, same size.

## LLM prompt

```text
You are implementing issue 016 for JC Grid Manager. Read docs/idea.md §4 (Move to Edge) and
docs/issues/016-move-to-edge.md. Issue 004 is done.

Task: Implement Move to Edge (Left/Right/Up/Down) — slide the window flush to an edge WITHOUT resizing,
using the current frame + work area. No default shortcuts (menu-only). Ensure the four Action variants
exist (add to the enum if missing). Unit-test the four origins.

Definition of done: acceptance criteria pass; window snaps flush to each edge at constant size; tests green.

Constraints: position only, never resize. Scope is only the four Move-to-Edge actions.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary;
do NOT git commit; refine the Suggested commit message; the user commits.
```

## Implementation log (fill this in)

- **Started:** 2026-07-08 18:47 WIB
- **Finished:** 2026-07-08 18:49 WIB
- **Duration:** ~2m

## Implementation summary

Added four `target_for` arms — MoveLeft/Right/Up/Down — that slide the window flush to a work-
area edge without resizing (position-only; the perpendicular axis is unchanged). Menu-only (no
default binds; user-bindable via 020). Unit test asserts the four origins at constant size.
`cargo test` → 43 pass.

## Suggested commit message

```
feat(core): add Move to Edge actions (left/right/up/down)

Slide the window flush to an edge without resizing.
```
