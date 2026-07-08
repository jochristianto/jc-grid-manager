# 022 — Ignore-app list

| | |
|---|---|
| **Issue ID** | 022 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004, 020 |
| **Blocks** | tray menu label (024) |
| **Default shortcut** | n/a (tray menu toggle) |
| **Source** | `docs/idea.md` §4 ("Ignore [App]", Other menu items) |
| **Status** | ☐ Not started |

## Summary

Add per-app **ignore**: the user can stop the app from managing windows of a specific application. The tray menu shows a dynamic **"Ignore [App]"** item labeled with the frontmost app's name (e.g. "Ignore Code" when VS Code is frontmost). When an app is ignored, all snap actions skip its windows.

## Context (self-contained)

- Needs the focused window's owning-app identity — the `Platform::identity(win)` method from 001 (bundle id + display name).
- The ignore list is persisted in the config (020). Toggling adds/removes the current app.
- Enforcement point: in the dispatcher, before executing an action, resolve the focused app; if it's in the ignore list, skip (optionally a soft beep via 021, or silent — see decision).

## What to build

1. **Ignore check in dispatch**: resolve the focused app's identity; if ignored, skip the action.
2. **Toggle command** `toggle_ignore_current_app()` (invoke + tray) that adds/removes the frontmost app's identity in the config ignore list and persists (via 020).
3. **Dynamic label source**: a way for the tray (024) to ask for the current frontmost app's display name to render "Ignore [App]" and reflect checked state.
4. Unit-test the ignore-list membership logic (given identity + list → skip?).

## Open decisions (recommendation)

- **Ignored-app feedback** — silent skip vs soft beep. _Recommendation:_ silent skip (the user chose to ignore it; a beep would be noise). Log at debug.
- **Identity key** — bundle id (macOS) / exe path or AUMID (Windows) vs display name. _Recommendation:_ key on a stable id (bundle id / exe path), store the display name alongside for the label.

## Acceptance criteria

- [ ] Toggling "Ignore [App]" for the frontmost app persists and immediately causes snap actions to skip that app's windows.
- [ ] Un-toggling restores management.
- [ ] The frontmost app's display name is available for the dynamic menu label and the item reflects checked state.
- [ ] Unit test: membership check (identity in list → skip).

## Testing

- Unit: membership logic.
- Manual: focus an app, toggle ignore (via a temporary command or the tray once 024 lands), press `⌃⌥←` → nothing happens for that app; toggle off → snapping resumes.

## LLM prompt

```text
You are implementing issue 022 for JC Grid Manager. Read docs/idea.md §4 (Ignore [App]) and
docs/issues/022-ignore-app-list.md. Issues 004 (dispatch) and 020 (config) are done.

Task: Add a per-app ignore list. Use Platform::identity to resolve the focused window's owning app;
if it's in the persisted ignore list (config, 020), the dispatcher skips the action (silent). Add a
toggle command that adds/removes the frontmost app and persists it, plus a way to fetch the frontmost
app's display name + checked state for the tray "Ignore [App]" label (issue 024). Unit-test the
membership logic.

Definition of done: acceptance criteria pass; ignoring an app immediately suppresses snapping for it and
persists; label data is available for the tray.

Constraints: key the list on a stable app id (bundle id / exe path), storing the display name for the
label. Silent skip (no beep) for ignored apps. Scope is only the ignore list.

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
feat(core): add per-app ignore list

Skip snap actions for apps the user has ignored; toggle the frontmost app via a
persisted list and expose its name/checked state for the tray "Ignore [App]" item.
```
