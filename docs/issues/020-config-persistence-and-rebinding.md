# 020 — Config persistence + rebinding + conflict validation

| | |
|---|---|
| **Issue ID** | 020 |
| **Layer** | core (backend, Rust) — defines the IPC contract for the frontend |
| **Depends on** | 004 |
| **Blocks** | 022 (ignore list persistence), 028/029 (frontend settings + shortcut editor) |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §2, §4 ("Every shortcut is rebindable"), §5.4, §5.6, §6, §7 |
| **Status** | ☐ Not started |

## Summary

Persist a **local, per-machine** config (§3 — no cloud sync) and make every shortcut **rebindable** with **validation**. This is the backend the frontend settings UI (028/029) talks to. It layers user overrides on top of the default binds from 004, validates new binds (reject duplicates and OS-refused combos), re-registers affected shortcuts live, and exposes a small set of `invoke` commands + events as the **IPC contract**.

## Context (self-contained)

- Config is local per machine; a rebind on one machine does not sync to the other (§2/§3).
- Config holds at least: shortcut overrides (`Action` → `Bind`), tunables (`almost_maximize_factor`, `resize_step`, `min_size` — from 012/015), the ignore list (used by 022), and the autostart flag (used by 023).
- Validation (§4/§5.4): reject a combo already used by another action; detect combos the OS refuses to register (try-register/rollback). Warn, don't silently drop.
- On change: persist, then **re-register** only the affected shortcuts.

## What to build

1. **Config module** `config.rs` — a serde struct with sane defaults, loaded from / saved to the OS app-config dir (path via Tauri). Missing file → defaults; forward-compatible parsing (unknown keys tolerated).
2. **Override resolution** — effective bind = user override if present, else 004 default. The dispatcher (004) reads effective binds.
3. **Validation** — `validate_bind(action, bind) -> Result<(), Conflict>`: duplicate detection across all effective binds + an OS-registration probe (register, and if it fails, report; roll back cleanly).
4. **Live re-registration** — unregister the old shortcut, register the new one, update the map; atomic-ish (roll back on failure).
5. **IPC contract** (`invoke` commands + events) — document these exact names in the file so the frontend (028/029) can build against them:
   - `get_config() -> Config`
   - `get_bindings() -> Vec<{ action, label, bind, is_default }>`
   - `set_binding(action, bind) -> Result<(), { code, message }>` (validates + persists + re-registers)
   - `reset_binding(action)` / `reset_all_bindings()`
   - `set_tunable(key, value)` (almost-maximize factor, resize step, min size)
   - event `bindings-changed` emitted after any change (so open UI refreshes)
6. Unit-test override resolution + duplicate detection with a synthetic config.

## Open decisions (recommendation)

- **Format & storage** — hand-rolled `serde_json` file vs `tauri-plugin-store` vs TOML. _Recommendation:_ `serde_json` to a single file in the Tauri app-config dir (simple, dependency-light, human-editable per §9's "hand-edit" fallback). Note TOML as an alternative if human-editing ergonomics matter more.
- **OS-refusal probe timing** — validate-on-set vs validate-on-open. _Recommendation:_ validate on `set_binding` (probe-register + rollback) so the user gets immediate feedback.

## Acceptance criteria

- [ ] Config persists across restarts in the OS app-config dir; a missing/partial file falls back to defaults without crashing.
- [ ] Effective bind = override-or-default; the dispatcher honors overrides.
- [ ] `set_binding` rejects a duplicate (used by another action) and an OS-refused combo, with a clear error; a valid bind persists and takes effect **without restart**.
- [ ] `reset_binding` / `reset_all_bindings` restore defaults.
- [ ] The `invoke` command + event names above exist and are documented in this file (the frontend contract).
- [ ] Unit tests: override resolution + duplicate detection.

## Testing

- Unit: override resolution, duplicate detection.
- Manual: rebind Left Half to a new combo via an `invoke` call (or a temporary test button), confirm it works live and survives restart; try to set a duplicate → rejected.

## LLM prompt

```text
You are implementing issue 020 for JC Grid Manager. Read docs/idea.md §2/§4/§5.4/§5.6/§6/§7 and
docs/issues/020-config-persistence-and-rebinding.md. Issue 004 (action registry + default binds) is done.

Task: Build local per-machine config persistence and live rebinding with validation. Layer user
overrides on top of the 004 default binds; validate new binds (reject duplicates and OS-refused combos
via a register/rollback probe); persist to the OS app-config dir; re-register affected shortcuts live.
Expose the documented invoke commands + `bindings-changed` event as the IPC contract for the frontend
(issues 028/029) — keep the names exactly as in the issue file. Unit-test override resolution and
duplicate detection.

Definition of done: acceptance criteria pass; rebinding works live and persists; invalid binds are
rejected with clear errors; the IPC contract is implemented and documented.

Constraints: config is LOCAL per machine (no sync). Do not build the frontend (028/029) — only the
backend + contract. Recommended storage: serde_json file in the app-config dir.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary
(list the final IPC command/event signatures you shipped); do NOT git commit; refine the Suggested
commit message; the user commits.
```

## Implementation log (fill this in)

- **Started:** _<!-- -->_
- **Finished:** _<!-- -->_
- **Duration:** _<!-- -->_

## Implementation summary (fill this in)

_<!-- ... -->_

## Suggested commit message

```
feat(core): persist local config and support validated live rebinding

Layer user shortcut overrides + tunables over defaults, persist to the app
config dir, validate new binds (duplicate + OS-refusal), and re-register live.
Expose get/set/reset binding invoke commands and a bindings-changed event.
```
