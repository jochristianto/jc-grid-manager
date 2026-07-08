# 028 — [Frontend] Settings window shell + navigation

| | |
|---|---|
| **Issue ID** | 028 |
| **Layer** | frontend (React + TypeScript) — own files; picked up by a frontend-focused agent |
| **Depends on** | 020 (backend config + IPC contract) |
| **Blocks** | 029 (shortcut editor lives inside this shell), 030 (onboarding is reachable from here) |
| **Default shortcut** | n/a (opened from the tray "Settings…" — 024) |
| **Source** | `docs/idea.md` §5.2, §6 |
| **Status** | ☐ Not started |

## Summary

Turn the default Tauri+React demo page into a real **Settings window** shell with navigation between sections: **Shortcuts** (editor lands in 029), **General** (launch-at-login toggle from 023, tunables from 012/015), and **About**. This is the container the other frontend slices plug into.

## Context (self-contained)

- Current `src/App.tsx` is the Vite/React/Tauri starter ("Welcome to Tauri + React") — replace it.
- Frontend talks to Rust via `@tauri-apps/api`'s `invoke()` and the event system (§5.2). The typed wrappers should live in `src/lib/tauriBridge.ts` (per §6).
- **IPC contract consumed here** (from issue 020 / 023 — confirm exact names against 020's implementation summary):
  - `get_config() -> Config`
  - `get_bindings() -> { action, label, bind, is_default }[]`
  - `set_tunable(key, value)` — `almost_maximize_factor`, `resize_step`, `min_size`
  - `get_autostart() -> boolean`, `set_autostart(enabled)`
  - event `bindings-changed`
- Project layout target (§6): `src/components/{SettingsWindow,ShortcutEditor,FirstRun}.tsx`, `src/lib/tauriBridge.ts`, `src/main.tsx`.

## What to build

1. Replace the demo `App.tsx` with a `SettingsWindow` shell: left/tab nav for **Shortcuts / General / About**, routed client-side.
2. `src/lib/tauriBridge.ts`: typed wrappers around the `invoke` commands + a helper to subscribe to `bindings-changed`. Keep all `invoke` calls behind this module (no raw `invoke` in components).
3. **General** section: launch-at-login toggle (`get/set_autostart`), and inputs for the tunables (`set_tunable`), with sensible ranges.
4. **About** section: app name, version, a line that it's an unsigned personal build, link out via the opener plugin.
5. **Shortcuts** section: a placeholder that 029 fills.
6. Theme: respect OS light/dark; keep it lightweight (§2).

## Open decisions (recommendation)

- **Routing** — a router lib vs simple state-based tabs. _Recommendation:_ state-based tabs (three sections; no need for a router dependency).
- **Styling** — plain CSS/CSS-modules vs a UI kit. _Recommendation:_ keep it minimal/native-feeling; no heavy component library (lightweight goal, §2).

## Acceptance criteria

- [ ] The demo page is gone; opening the window shows a Settings shell with Shortcuts / General / About navigation.
- [ ] All backend calls go through `src/lib/tauriBridge.ts` (typed).
- [ ] General: launch-at-login toggle reflects and updates real state; tunable inputs call `set_tunable` and reflect current config.
- [ ] About shows name/version and notes the unsigned build.
- [ ] Light/dark follows the OS; window is usable at its default size.

## Testing

- Manual: open Settings from the tray; switch sections; toggle launch-at-login and confirm via 023; change a tunable and confirm it persists (reopen).

## LLM prompt

```text
You are implementing issue 028 for JC Grid Manager (FRONTEND, React + TypeScript). Read docs/idea.md
§5.2/§6 and docs/issues/028-frontend-settings-window.md. The backend IPC contract from issue 020 (and
autostart from 023) is available — confirm exact command names against issue 020's Implementation summary.

Task: Replace the default Tauri+React demo (src/App.tsx) with a real Settings window: state-based tab nav
for Shortcuts / General / About. Add src/lib/tauriBridge.ts with typed wrappers around the invoke commands
and a bindings-changed subscription; route ALL backend calls through it. Build the General section
(launch-at-login toggle + tunable inputs) and About. Leave a placeholder for the Shortcuts editor (issue
029). Respect OS light/dark; keep it lightweight (no heavy UI kit).

Definition of done: acceptance criteria pass; the shell works and General reads/writes real backend state.

Constraints: same repo, but keep this to frontend files (src/**). No raw invoke() in components — only via
tauriBridge.ts. Don't build the shortcut recorder (029) or onboarding (030) beyond entry points.

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
feat(ui): add Settings window shell with tabbed navigation

Replace the starter page with Shortcuts/General/About sections and a typed
tauriBridge; General wires launch-at-login and tunables to the backend.
```
