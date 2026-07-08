# 030 — [Frontend] First-run / Accessibility onboarding (macOS)

| | |
|---|---|
| **Issue ID** | 030 |
| **Layer** | frontend (React + TypeScript) — own files |
| **Depends on** | 001/021 (a backend trust-state command), 028 (shell) |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §8 (the three states), §5.6, §11 |
| **Status** | ☑ Done |

## Summary

Build the macOS **first-run / Accessibility** onboarding that handles **three** states, not one (§8): never-asked, denied, and the unsigned-app "should-be-trusted-but-isn't" case after a reinstall/update. Explain why the permission is needed and guide the user to fix each state.

## Context (self-contained)

- macOS needs Accessibility permission to move other apps' windows. The backend already calls `AXIsProcessTrusted()` / `AXIsProcessTrustedWithOptions` in the macOS shim.
- **Three states to handle (§8):**
  1. **Never asked** → trigger the system prompt and explain why it's needed.
  2. **Denied** → deep-link to System Settings → Privacy & Security → Accessibility, and explain.
  3. **Should-be-trusted-but-isn't** (unsigned app after reinstall/update — the stale grant, §5.6) → guide to remove & re-add the entry, or run `tccutil reset Accessibility com.jochristianto.jcgridmanager` and relaunch.
- **Backend command needed** (define/confirm with the backend): `get_accessibility_state() -> "trusted" | "never_asked" | "denied"` plus actions `prompt_accessibility()` and `open_accessibility_settings()`. If these don't exist yet, this issue includes adding thin wrappers in the shim (they're a few AX / `NSWorkspace open URL` calls) — coordinate so the frontend has a stable contract.
- Windows needs no permission, so this screen is macOS-only; on Windows it should not appear.

## What to build

1. A `FirstRun` component shown on launch when the state is not `trusted` (and reachable later from Settings if it regresses).
2. Branch the UI by the three states with clear copy + the right call-to-action button per state (prompt / open settings / show reset instructions).
3. Poll or re-check on window focus so that once the user grants it, the screen advances to a success state without a manual restart where possible.
4. macOS-only gating.

## Open decisions (recommendation)

- **Distinguishing "never asked" vs "denied"** — TCC doesn't expose this cleanly. _Recommendation:_ treat first launch (no prior prompt recorded in config) as never-asked → prompt once; thereafter, if still untrusted, treat as denied/stale and show the settings + reset guidance. Record "prompted once" in config (020) to make the distinction.
- **Auto-advance** — poll interval vs focus-based re-check. _Recommendation:_ re-check on window focus + a slow poll while the screen is open; stop once trusted.

## Acceptance criteria

- [ ] On macOS without Accessibility, first launch shows onboarding that prompts and explains why.
- [ ] If denied, the screen deep-links to the Accessibility settings pane and explains.
- [ ] The stale-grant case shows remove/re-add guidance and the exact `tccutil reset` command.
- [ ] After the user grants permission, the screen advances to success without requiring a manual relaunch (where feasible).
- [ ] The screen never appears on Windows.

## Testing

- Manual (macOS): fresh state → prompt; deny → settings guidance; use `tccutil reset Accessibility com.jochristianto.jcgridmanager` to simulate the stale case → reset guidance; grant → advances to success.

## LLM prompt

```text
You are implementing issue 030 for JC Grid Manager (FRONTEND, React + TypeScript, macOS-only screen).
Read docs/idea.md §8/§5.6/§11 and docs/issues/030-frontend-first-run-onboarding.md. Issue 028 (shell +
tauriBridge) is done.

Task: Build the macOS Accessibility onboarding handling three states (never-asked / denied / stale-grant),
with correct copy and call-to-action per state (prompt, open Accessibility settings, show tccutil reset
guidance). Gate it to macOS only. Re-check on focus so it advances to success once granted. It needs a
backend trust-state contract: get_accessibility_state(), prompt_accessibility(), open_accessibility_settings()
— if missing, add thin wrappers in the macOS shim (a few AX / NSWorkspace-open-URL calls) and record a
"prompted once" flag in config (020) to distinguish never-asked from denied.

Definition of done: acceptance criteria pass; all three states are handled with the right guidance; the
screen never shows on Windows.

Constraints: frontend + only the minimal backend trust-state wrappers if they don't exist. Reuse tauriBridge.
Scope is the onboarding flow.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary (note any
backend wrappers you added); do NOT git commit; refine the Suggested commit message; the user commits.
```

## Implementation log

- **Started:** 2026-07-08 21:58 WIB
- **Finished:** 2026-07-08 22:06 WIB
- **Duration:** ~8m hands-on (excludes reading/design)

## Implementation summary

Built the macOS Accessibility onboarding, plus the thin backend trust-state contract it needed.
The app gates on the permission at launch and advances to Settings on its own once granted.

**Backend added** (the wrappers the issue anticipated)
- **`platform/macos.rs`** — split the old `ensure_trusted` into reusable `is_trusted()` (pure
  `AXIsProcessTrusted`, no prompt) and `prompt()` (the `AXIsProcessTrustedWithOptions` system
  prompt); `ensure_trusted` now composes them. Added `open_accessibility_settings()` — opens
  `x-apple.systempreferences:…Privacy_Accessibility` via `NSWorkspace` (that URL scheme is why it's
  a shim call, not the opener plugin).
- **`platform/mod.rs`** — `accessibility_trusted()` (macOS query; **always true off macOS**, so no
  onboarding on Windows), `prompt_accessibility()`, `open_accessibility_settings()` (no-ops off
  macOS).
- **`config.rs`** — new persisted `accessibility_prompted: bool`, and three commands:
  `get_accessibility_state() -> "trusted"|"never_asked"|"denied"` (trusted if the AX check passes;
  else `never_asked`/`denied` by the prompted flag), `prompt_accessibility()` (prompts + records the
  flag), `open_accessibility_settings()`. Registered in `lib.rs`; documented in the IPC contract.

**Frontend added**
- **`tauriBridge.ts`** — `AccessibilityState` type + `getAccessibilityState`/`promptAccessibility`/
  `openAccessibilitySettings`.
- **`components/FirstRun.tsx`** — the onboarding card. `never_asked` → explain + **Grant
  Accessibility Access** (`prompt_accessibility`). `denied` → explain + **Open Accessibility
  Settings** (`open_accessibility_settings`) **and** the stale-grant block (remove/re-add guidance +
  the exact `tccutil reset Accessibility com.jochristianto.jcgridmanager` with a Copy button).
- **`App.tsx`** — gates the app: `loading → trusted → SettingsWindow`, otherwise `<FirstRun>`. It
  **re-checks on window focus and slow-polls (2 s)** so granting permission in System Settings
  advances to the app without a relaunch, then stops polling. **Skips the check entirely off macOS**
  (`IS_MAC`) so the screen never appears on Windows. On a backend error it fails open to the app.
- **`App.css`** — onboarding-card styles.

**Key decisions / deviations**
- **Three §8 states via one persisted flag**: TCC can't distinguish denied from a stale grant, so
  both surface as `denied`, and that screen carries **both** the settings deep-link and the reset
  guidance — covering all three acceptance rows.
- Double macOS gate (backend returns trusted off-macOS **and** the frontend `IS_MAC` short-circuit)
  so the onboarding truly never renders on Windows.

**Verification**
- Backend: `cargo test` → 68 pass, `cargo clippy --all-targets` clean, `cargo build` links the new
  `NSWorkspace`/`NSURL` calls. Frontend: `pnpm exec tsc --noEmit` clean, `pnpm run build` OK.
- Headless (browse): the app **mounts with no console errors**; in a plain browser it renders the
  brief loading (null) state because `get_accessibility_state` (invoke) has no Tauri backend to
  answer — expected; under `tauri dev` it resolves instantly.
- **Not run here** (needs `tauri dev` on macOS): fresh state → prompt; deny → settings guidance;
  `tccutil reset Accessibility com.jochristianto.jcgridmanager` to simulate the stale case → reset
  guidance; grant → auto-advances to Settings. Flagged for the user.

## Suggested commit message

```
feat(ui): add macOS Accessibility onboarding (three states)

Gate the app on the macOS Accessibility permission (idea.md §8). Add the backend
trust-state contract — get_accessibility_state / prompt_accessibility /
open_accessibility_settings, with a persisted accessibility_prompted flag to tell
"never asked" from "denied". FirstRun handles never-asked (system prompt), denied
(settings deep-link), and the unsigned stale-grant case (tccutil reset guidance);
it re-checks on focus + a slow poll so granting advances to the app without a
relaunch. macOS-only — the screen never shows on Windows. cargo test 68; tsc/build
clean.
```
