# 022 — Ignore-app list

| | |
|---|---|
| **Issue ID** | 022 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004, 020 |
| **Blocks** | tray menu label (024) |
| **Default shortcut** | n/a (tray menu toggle) |
| **Source** | `docs/idea.md` §4 ("Ignore [App]", Other menu items) |
| **Status** | ☑ Done |

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

## Implementation log

- **Started:** 2026-07-08 21:12 WIB
- **Finished:** 2026-07-08 21:22 WIB
- **Duration:** ~10m hands-on (excludes reading/design)

## Implementation summary

Added the per-app ignore list (§4): the user can stop the app from managing a given application's
windows. Snap actions silently skip an ignored app; the tray (024) gets the data it needs for the
dynamic "Ignore [App]" item. Membership logic is pure + unit-tested.

**What changed**
- **`platform/macos.rs`**: implemented the real `Platform::identity(win)` (was a stub) — reads the
  window's process id (`AXUIElementGetPid`), looks up its `NSRunningApplication`, and returns
  `{ bundle_id, localizedName }`. Added an `NSString → String` helper. (Uses `AXUIElementGetPid`
  from `accessibility-sys` and `NSRunningApplication` via `msg_send`; no new crates.)
- **`platform/mod.rs`**:
  - `WindowIdentity` gained `key()` (stable list key: bundle id, else name), `is_ignored(list)`
    (pure membership), and `display_label()` (name → bundle id → "this app"). Dropped its (and the
    `Platform` trait's) `#[allow(dead_code)]` now that identity is wired for real.
  - `focused_app_identity()` — resolves the frontmost window's owning app through the same path as
    `perform` (main-thread, needs Accessibility). Shared by the dispatcher and the commands.
- **`config.rs`**:
  - `IgnoreStatus { id, name, ignored }` — the frontmost app + checked state for the tray.
  - `get_frontmost_app()` and `toggle_ignore_current_app()` commands (invoke + tray). Both call
    AppKit directly: **synchronous** Tauri commands run on the main thread (confirmed by Tauri's
    own "deadlocks in a synchronous command" note on `run_on_main_thread`), so no marshaling is
    needed. Toggle adds/removes the app's `key` in `config.ignore_apps` and persists via 020's
    `save`. Pure `toggle_membership` helper is unit-tested. New error code `identity`.
- **`lib.rs`**:
  - Enforcement in `dispatch` (§4's named enforcement point): the config snapshot now also grabs
    `ignore_apps`; inside the main-thread closure, before acting, if the list is non-empty and the
    frontmost app is ignored, it logs and returns — **silent skip**, no beep.
  - Registered `get_frontmost_app` + `toggle_ignore_current_app`.

**Key decisions / deviations**
- **Silent skip, no beep** for ignored apps (the recommendation): the user opted in, so a beep
  would be noise. Logged instead.
- **Key on a stable id** (bundle id, falling back to name); the tray label uses the **live** name
  from `focused_app_identity`, so the config only needs to store keys — `ignore_apps: Vec<String>`
  from 020 is kept as-is (no schema change). Storing display names alongside wasn't necessary.
- **Empty-list fast path**: identity is only resolved when something is actually ignored, so the
  common case adds no Accessibility round-trip (or prompt) per keypress.
- **No `bindings-changed` emit** for ignore toggles (same call as autostart) — the tray/settings
  caller updates from the command's returned `IgnoreStatus`; the event stays scoped to bindings.

**Verification**
- `cargo test` → 67 pass (4 new: `key` precedence, `is_ignored` membership incl. anonymous +
  name-fallback, `display_label` fallbacks, `toggle_membership` add/remove). `cargo clippy
  --all-targets` clean. `cargo build` OK (new AX/AppKit FFI links).
- **Not run here:** the manual smoke (focus an app, `toggle_ignore_current_app`, press `⌃⌥←` →
  nothing happens for that app; toggle off → snapping resumes) — needs the running app + a focused
  window + Accessibility grant. Flagged for the user, as with 001/020/021/023.

**Follow-up:** 024 renders the tray "Ignore [App]" item from `get_frontmost_app` (label + checked)
and calls `toggle_ignore_current_app` on click.

## Suggested commit message

```
feat(core): add per-app ignore list

Let the user stop managing a specific app's windows (§4). Implement the real
Platform::identity (window pid -> NSRunningApplication bundle id + name), and skip
snap actions in the dispatcher when the frontmost app is in the persisted ignore
list — silently, since the user opted in.

Add get_frontmost_app and toggle_ignore_current_app commands (frontmost app name +
checked state, and a persisted add/remove toggle) so the tray can render a dynamic
"Ignore [App]" item (024). New tests cover key selection, membership, and the
toggle (cargo test: 67).
```
