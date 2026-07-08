# 011 — Maximize

| | |
|---|---|
| **Issue ID** | 011 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 (soft: 003 to maximize on the correct display) |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | `⌃⌥↩` (Control+Option+Return) |
| **Source** | `docs/idea.md` §4 (Sizing → Maximize) |
| **Status** | ☐ Not started |

## Summary

Add **Maximize**: fill the entire **work area** of the window's display. Not macOS native fullscreen — just the visible frame (respects the menu bar / taskbar).

## Geometry (fraction of work area)

- Maximize → `(0.0, 0.0, 1.0, 1.0)`

## What to build

1. Add the `Maximize` geometry (full work area) to the core action table.
2. Default bind `⌃⌥↩` already exists in 004 — make it functional.
3. Interacts well with the state machine: a fresh Maximize should capture a restore baseline like any snap (so `⌃⌥⌫` un-maximizes). Confirm this works if 005 is present.

## Acceptance criteria

- [ ] `⌃⌥↩` fills the focused window's display work area (not covering the menu bar).
- [ ] On multi-monitor (if 003 is done), it maximizes on the window's own display.
- [ ] If 005 is present, `⌃⌥⌫` afterward restores the pre-maximize frame.
- [ ] Unit test asserts the full-work-area rect.

## Testing

- Unit: fraction conversion `(0,0,1,1)`.
- Manual: `⌃⌥↩` fills the work area; `⌃⌥⌫` restores.

## LLM prompt

```text
You are implementing issue 011 for JC Grid Manager. Read docs/idea.md §4 (Maximize) and
docs/issues/011-maximize.md. Issue 004 is done; 003 and 005 may or may not be — integrate with them
if present but don't require them.

Task: Implement Maximize as fill-the-work-area (fraction (0,0,1,1)) of the window's display — NOT native
macOS fullscreen. Default bind ⌃⌥↩ already exists; make it work. Ensure it captures a restore baseline
via the state machine if that's present. Unit-test the conversion.

Definition of done: acceptance criteria pass; ⌃⌥↩ fills the work area; tests green.

Constraints: work area only (respect menu bar/taskbar), never the fullscreen space. Scope is only Maximize
(Almost Maximize is issue 012, Maximize Height is 013).

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
feat(core): add Maximize action (⌃⌥↩)

Fill the window's display work area (not native fullscreen); captures a
restore baseline so ⌃⌥⌫ un-maximizes.
```
