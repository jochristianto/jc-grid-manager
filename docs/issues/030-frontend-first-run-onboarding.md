# 030 — [Frontend] First-run / Accessibility onboarding (macOS)

| | |
|---|---|
| **Issue ID** | 030 |
| **Layer** | frontend (React + TypeScript) — own files |
| **Depends on** | 001/021 (a backend trust-state command), 028 (shell) |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §8 (the three states), §5.6, §11 |
| **Status** | ☐ Not started |

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

## Implementation log (fill this in)

- **Started:** _<!-- -->_
- **Finished:** _<!-- -->_
- **Duration:** _<!-- -->_

## Implementation summary (fill this in)

_<!-- ... -->_

## Suggested commit message

```
feat(ui): add macOS Accessibility onboarding (three states)

Handle never-asked / denied / stale-grant with tailored guidance (system
prompt, settings deep-link, tccutil reset), auto-advancing once trusted.
macOS-only.
```
