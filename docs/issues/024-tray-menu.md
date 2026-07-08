# 024 — Tray menu mirroring §4

| | |
|---|---|
| **Issue ID** | 024 |
| **Layer** | core (backend, Rust) — this is **not** frontend; the tray menu is built in Rust |
| **Depends on** | 005–019 (the actions it invokes), 022 (ignore label); soft-dep 020 |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §4 (all menus + "Other menu items"), §5.5 |
| **Status** | ☐ Not started |

## Summary

Replace the current Quit-only tray menu with the full menu from §4, so every action is reachable without a shortcut — including the menu-only ones (Center Half, Move to Edge, Fourths, Sixths). Add the dynamic **"Ignore [App]"** item, plus **Settings…**, **About**, **Quit**. Built with Tauri's tray API in Rust (§5.5), not React.

## Context (self-contained)

- Current menu is built in `lib.rs` via `TrayIconBuilder` with a single Quit item.
- Every menu item invokes an `Action` through the dispatcher (004) — the same path as the shortcuts, so items work even for actions with no default bind.
- Menu structure (mirror §4): **Halves** (+ Center Half), **Corners**, **Thirds**, **Sizing** (Maximize, Almost Maximize, Maximize Height, Smaller, Larger, Center, Restore), **Displays** (Next/Previous), and sub-menus **Move to Edge**, **Fourths**, **Sixths**. Then **Ignore "[App]"** (dynamic label + checkmark from 022), **Settings…** (opens the window — 028), **About**, **Quit**. **No "Check for Updates…"** in v1 (§4/§9).
- The "Ignore [App]" label must update to the frontmost app's name each time the menu opens (use the tray/menu open event or rebuild-on-open).

## What to build

1. Build the full nested menu mirroring §4; each leaf dispatches its `Action`.
2. Where an action has a default shortcut, show it as the item's accelerator/hint text (read from effective binds, 020, if present; else the 004 defaults).
3. Dynamic **Ignore "[App]"**: on menu-about-to-open, fetch the frontmost app name + ignored state (022) and set the label + checkmark.
4. **Settings…** opens the settings window (created in 028); until 028 lands, it can show/focus the existing main window.
5. **About** (simple dialog/window) and **Quit** (existing).

## Open decisions (recommendation)

- **Rebuild-on-open vs static menu** — needed for the dynamic Ignore label + live accelerators. _Recommendation:_ rebuild (or update) the menu on the tray's menu-open event so the app name and any rebinds are current.
- **Showing accelerators** — some platforms render menu accelerators differently in tray menus. _Recommendation:_ include them as trailing hint text for discoverability; don't rely on them being actionable from the menu.

## Acceptance criteria

- [ ] The tray menu lists all §4 actions in the grouped structure, including the menu-only ones, each invoking the correct action.
- [ ] Menu-only actions (Center Half, Move to Edge, Fourths, Sixths) are triggerable from the menu.
- [ ] "Ignore '[App]'" shows the current frontmost app's name and reflects its ignored state; toggling it works (022).
- [ ] Settings…, About, and Quit are present and functional (Settings opens the window or focuses the main window pre-028).
- [ ] No "Check for Updates…" item.

## Testing

- Manual: open the tray menu, trigger a representative action from each group and each sub-menu; switch frontmost app and reopen → "Ignore [App]" label updates; toggle it and confirm 022 behavior.

## LLM prompt

```text
You are implementing issue 024 for JC Grid Manager. Read docs/idea.md §4, §5.5, §9 and
docs/issues/024-tray-menu.md. The action slices 005–019 and the ignore list (022) are done; config (020)
may be present.

Task: Replace the Quit-only tray menu with the full §4 menu built via Tauri's tray API in Rust. Every
leaf dispatches its Action through the existing dispatcher (004), so menu-only actions work too. Add the
dynamic "Ignore '[App]'" item (name + checkmark from 022, refreshed on menu-open), Settings… (opens the
settings window from 028, or focuses the main window until then), About, and Quit. Show default/effective
shortcuts as hint text. Do NOT add a "Check for Updates…" item (out of v1).

Definition of done: acceptance criteria pass; all groups + sub-menus present and functional; Ignore label
is dynamic.

Constraints: this is Rust (tray API), not React. Reuse the dispatcher — don't reimplement action logic.
Scope is only the tray menu (the Settings/About windows' contents are frontend issues, but wiring the menu
items to open them is here).

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
feat(tray): full menu mirroring the Rectangle feature set

Replace the Quit-only menu with grouped Halves/Corners/Thirds/Sizing/Displays
plus Move-to-Edge/Fourths/Sixths sub-menus, a dynamic Ignore "[App]" item, and
Settings/About/Quit. Every item dispatches through the shared action registry.
```
