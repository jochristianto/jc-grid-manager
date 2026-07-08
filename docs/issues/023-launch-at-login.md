# 023 — Launch at login (autostart)

| | |
|---|---|
| **Issue ID** | 023 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 (soft: 020 to persist the toggle) |
| **Blocks** | surfaced as a settings toggle (028) |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §2, §8 ("Launch at login") |
| **Status** | ☐ Not started |

## Summary

Add **optional launch at login** via the Tauri autostart plugin, toggleable from settings. Off by default; the user opts in.

## Context (self-contained)

- Use `tauri-plugin-autostart` (add to `Cargo.toml` + register the plugin in `lib.rs`).
- The toggle state should reflect the OS-registered reality (query the plugin), and also be mirrored in config (020) for the settings UI.
- Frontend toggle lives in the Settings window (028); this issue provides the backend command + wiring.

## What to build

1. Add and initialize `tauri-plugin-autostart`.
2. `invoke` commands: `get_autostart() -> bool`, `set_autostart(enabled: bool)`.
3. Keep config (020) in sync if present; otherwise rely on the plugin's own state.
4. Default: disabled.

## Acceptance criteria

- [ ] Enabling launch-at-login registers the app to start on login (verify the OS login-items/registry entry appears); disabling removes it.
- [ ] `get_autostart` reflects the actual registered state after a restart.
- [ ] Default is disabled on a fresh install.

## Testing

- Manual: toggle on → confirm the OS login item exists (macOS: System Settings → General → Login Items; Windows: Task Manager → Startup); reboot/login → app starts; toggle off → entry removed.

## LLM prompt

```text
You are implementing issue 023 for JC Grid Manager. Read docs/idea.md §2, §8 and
docs/issues/023-launch-at-login.md. Issue 004 is done; 020 (config) may be present.

Task: Integrate tauri-plugin-autostart and expose get_autostart/set_autostart invoke commands, defaulting
to disabled. Mirror the state in config (020) if present. The settings UI toggle is issue 028 — only
provide the backend here.

Definition of done: acceptance criteria pass; toggling registers/unregisters the OS login item and the
state is queryable after restart.

Constraints: default disabled. Scope is only autostart wiring (no settings UI).

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
feat(core): add optional launch-at-login via autostart plugin

Integrate tauri-plugin-autostart with get/set_autostart commands (default off),
mirrored in config for the settings UI.
```
