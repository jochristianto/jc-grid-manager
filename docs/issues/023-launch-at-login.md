# 023 — Launch at login (autostart)

| | |
|---|---|
| **Issue ID** | 023 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 (soft: 020 to persist the toggle) |
| **Blocks** | surfaced as a settings toggle (028) |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §2, §8 ("Launch at login") |
| **Status** | ☑ Done |

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

## Implementation log

- **Started:** 2026-07-08 21:05 WIB
- **Finished:** 2026-07-08 21:11 WIB
- **Duration:** ~6m hands-on (excludes reading/design)

## Implementation summary

Added optional launch-at-login (§8) via `tauri-plugin-autostart`, off by default, with the
backend commands the settings UI (028) will drive. No UI here.

**What changed**
- **`Cargo.toml`**: added `tauri-plugin-autostart = "2"` (pulls `auto-launch` + friends;
  `Cargo.lock` updated).
- **`lib.rs`**: registered `tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None)` and
  added `get_autostart` / `set_autostart` to the invoke handler.
- **`config.rs`** (kept the autostart commands next to the other config commands so they can
  reuse the private `save` + `ConfigState`):
  - `get_autostart() -> Result<bool, BindingError>` — reads the **OS-registered reality** via
    `app.autolaunch().is_enabled()` (the source of truth that survives a restart).
  - `set_autostart(enabled) -> Result<(), BindingError>` — flips the OS login item first
    (`enable`/`disable`), then mirrors the new value into `config.autostart` and persists. On an
    OS failure, config is left untouched. New error code `autostart`.
  - Updated the module's IPC-contract doc-comment with both commands + the new code.

**Key decisions / deviations**
- **OS is the source of truth**, config is a mirror (per the issue): `get_autostart` never trusts
  the file — it queries the plugin — so the toggle is correct after a restart even if the file is
  stale or hand-edited. No startup reconciliation needed: the login item is an OS artifact that
  persists on its own.
- **No `bindings-changed` emit** for autostart — that event is scoped to shortcut/tunable changes,
  and the toggle is self-driven by the one settings window, so a re-fetch signal adds nothing.
- Default disabled falls out for free: a fresh install has no login item → `is_enabled()` is
  `false` and `config.autostart` defaults to `false`.
- `MacosLauncher::LaunchAgent` (the modern LaunchAgent plist), not the legacy AppleScript path.

**Verification**
- `cargo build` OK (new plugin links), `cargo test` → 63 pass (unchanged — the wiring is thin
  plugin glue; `config.autostart` round-trip is already covered by 020's config test), `cargo
  clippy --all-targets` clean.
- **Not run here:** the manual toggle smoke (enable → macOS System Settings → General → Login
  Items shows the app; restart → `get_autostart` still true; disable → entry removed). It
  registers a real OS login item, so it's left for the user to run on the target machine.

**Follow-up:** 028 surfaces this as a Settings toggle (calls `get_autostart` on open,
`set_autostart` on change).

## Suggested commit message

```
feat(core): add optional launch-at-login via autostart plugin

Integrate tauri-plugin-autostart (LaunchAgent) and expose get_autostart /
set_autostart, off by default. get_autostart reads the OS-registered reality so
the state is correct after a restart; set_autostart flips the login item and then
mirrors it into config.autostart for the settings UI (028). Backend only.
```
