# 028 — [Frontend] Settings window shell + navigation

| | |
|---|---|
| **Issue ID** | 028 |
| **Layer** | frontend (React + TypeScript) — own files; picked up by a frontend-focused agent |
| **Depends on** | 020 (backend config + IPC contract) |
| **Blocks** | 029 (shortcut editor lives inside this shell), 030 (onboarding is reachable from here) |
| **Default shortcut** | n/a (opened from the tray "Settings…" — 024) |
| **Source** | `docs/idea.md` §5.2, §6 |
| **Status** | ☑ Done |

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

## Implementation log

- **Started:** 2026-07-08 21:38 WIB
- **Finished:** 2026-07-08 21:50 WIB
- **Duration:** ~12m hands-on (excludes reading/design)

## Implementation summary

Replaced the Tauri+React starter with a real Settings window: state-based tab nav for **Shortcuts
/ General / About**, all backend access funneled through one typed bridge.

**What changed** (frontend only — no `src-tauri/**` touched)
- **`src/lib/tauriBridge.ts`** — typed wrappers for the whole IPC contract (020/022/023):
  `Config`, `Bind`, `Tunables`, `BindingInfo`, `IgnoreStatus`, `BindingError` types +
  `getConfig`/`getBindings`/`setBinding`/`resetBinding`/`resetAllBindings`/`setTunable`/
  `getAutostart`/`setAutostart`/`getFrontmostApp`/`toggleIgnoreCurrentApp`, an
  `onBindingsChanged` subscription, and an `errorMessage` helper. No component calls `invoke`
  directly. Field names/shapes verified against `src-tauri/src/config.rs` (e.g. `Bind` uses the
  `super` key from `BindRepr`; `Tunables` = `almost_maximize_factor`/`resize_step`/`min_size`).
- **`src/components/SettingsWindow.tsx`** — the shell: sidebar tabs, client-side routing by
  `useState`. About uses `@tauri-apps/api/app` `getVersion` + the opener plugin (`openUrl`) and
  notes the unsigned build + Accessibility requirement. Shortcuts is a labelled placeholder for
  029.
- **`src/components/GeneralSection.tsx`** — launch-at-login checkbox (`get/set_autostart`,
  optimistic with rollback) and three tunable sliders (`set_tunable`, committed on release, resynced
  from `getConfig` if the backend rejects). Slider ranges stay inside the backend's accepted bounds.
- **`src/App.tsx`** now just renders `<SettingsWindow/>`; **`src/App.css`** rewritten as a
  lightweight, native-feeling theme that follows OS light/dark via `prefers-color-scheme`;
  **`index.html`** title updated. (Greet demo + logos gone.)

**Key decisions / deviations**
- **State tabs, no router**, and **no UI kit** — per the issue's recommendations (lightweight, §2).
- Default tab is **General** (functional today) rather than the Shortcuts placeholder.
- Tunables are **sliders committed on `pointerup`/`keyup`**, not per-keystroke number inputs, so a
  half-typed value never hits `set_tunable` (which would reject and flicker).
- Kept `ignore_apps` / the ignore commands in the bridge (for 022/settings later) even though this
  screen doesn't surface them yet — the bridge is the shared module for all frontend slices.

**Verification**
- `pnpm exec tsc --noEmit` → clean (strict, `noUnusedLocals`/`noUnusedParameters`).
  `pnpm run build` (`tsc && vite build`) → built, 38 modules, `dist/` (gitignored).
- **Rendered it headlessly** (vite dev @ :1420, gstack browse): shell mounts with **no console
  errors**; the three tabs switch correctly; **About** shows name / version / unsigned-build note /
  GitHub link; **Shortcuts** shows its placeholder. **General** stays on "Loading…" in a plain
  browser only because `invoke` has no Tauri backend there — expected; it loads under `tauri dev`.
- **Not run here:** the live `tauri dev` smoke (open Settings from the tray → General reads real
  state; toggle launch-at-login → verify via 023; change a tunable → reopen and confirm it
  persists). Needs the running app; flagged for the user.

## Suggested commit message

```
feat(ui): add Settings window shell with tabbed navigation

Replace the Tauri+React starter with a Settings window: state-based Shortcuts /
General / About tabs, following the OS light/dark theme. Route every backend call
through a new typed src/lib/tauriBridge.ts (the 020/022/023 IPC contract); no raw
invoke in components. General wires launch-at-login and the sizing tunables to the
backend; About shows name/version and notes the unsigned build. Shortcuts is a
placeholder for 029. tsc + vite build clean; shell verified headlessly.
```
