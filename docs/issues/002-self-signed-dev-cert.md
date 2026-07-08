# 002 — Self-signed dev cert for a stable macOS identity

| | |
|---|---|
| **Issue ID** | 002 |
| **Layer** | dev (tooling) |
| **Depends on** | None — can start immediately (do this **early**) |
| **Blocks** | Nothing hard, but removes a recurring dev-time pain for 001 and all macOS action work |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §5.6, §8, §11 |
| **Status** | ☐ Not started |

## Summary

The app ships **unsigned** in v1 (no paid Apple Developer ID — §5.6). macOS ties the Accessibility grant to the app's code identity, so an *ad-hoc*/unsigned rebuild looks like a different app each time and **the Accessibility permission resets on every rebuild**. During development that means re-granting Accessibility constantly. Fix it cheaply: sign local builds with a **stable self-signed certificate** so macOS keeps recognizing rebuilds as the same app.

## Context (self-contained)

- This is a **dev-ergonomics** task, not release signing. It is free and local.
- The goal: every `pnpm tauri dev` / `pnpm tauri build` produces a binary with the **same** code-signing identity, so the entry under System Settings → Privacy & Security → Accessibility stays valid across rebuilds.
- Bundle identifier is `com.jochristianto.jcgridmanager` (`src-tauri/tauri.conf.json`).

## What to build

1. **Create a stable self-signed code-signing certificate** in the login keychain (e.g. via Keychain Access → Certificate Assistant → *Create a Certificate…*, type *Code Signing*, or a documented `security`/`codesign` flow). Document the exact steps in a short `docs/dev-signing.md` (or a section in `README.md`) so it is reproducible on a fresh machine.
2. **Wire Tauri to sign dev/debug builds with that identity.** Configure the macOS signing identity for the bundler (e.g. `tauri.conf.json` → `bundle.macOS.signingIdentity`, or an env var such as `APPLE_SIGNING_IDENTITY` used by the dev script). Ensure it applies to the artifact that actually runs during `tauri dev`.
3. **Verify the grant survives a rebuild**: grant Accessibility once, rebuild, confirm the app is still trusted without re-granting.
4. **Document the reset escape hatch** for when identity does drift: `tccutil reset Accessibility com.jochristianto.jcgridmanager` then relaunch (this is referenced by issue 030's onboarding flow too).

## Open decisions (recommendation)

- **Certificate creation method** — Keychain Access GUI vs a scripted `security create-keychain` + `openssl`/`codesign` flow. _Recommendation:_ document the Keychain Access GUI steps (simplest, one-time), and note the CLI equivalent for CI-less reproducibility.
- **Where signing config lives** — committed in `tauri.conf.json` vs a local, git-ignored env. _Recommendation:_ keep the identity **name** referenced via an env var (e.g. in a git-ignored `.env` the dev script reads) so the repo does not hardcode one machine's cert name; document the expected name.

## Acceptance criteria

- [ ] A reproducible doc exists for creating the self-signed code-signing cert.
- [ ] Local macOS builds are signed with that stable identity (`codesign -dv` shows a consistent identity across rebuilds).
- [ ] After granting Accessibility once, at least two subsequent rebuilds run **without** re-granting.
- [ ] The `tccutil reset` recovery command is documented.

## Testing

- Manual: `codesign -dvv path/to/JC\ Grid\ Manager.app` before and after a rebuild → same identity. Grant → rebuild → still trusted.

## LLM prompt

```text
You are implementing issue 002 for JC Grid Manager. Read docs/idea.md §5.6, §8, §11 and the issue
file docs/issues/002-self-signed-dev-cert.md.

Task: Set up a stable, free, self-signed macOS code-signing identity so local rebuilds keep the same
code identity and the Accessibility permission grant survives rebuilds. Document the steps and the
tccutil reset recovery command. Wire Tauri's macOS bundler to use the identity for dev/debug builds.

Definition of done: builds are signed with a consistent identity across rebuilds, the Accessibility
grant persists across at least two rebuilds, and the process is documented reproducibly.

Constraints: this is DEV signing only — do NOT attempt paid Developer ID / notarization (out of scope,
§3). Do not hardcode a single machine's certificate name into committed config; reference it via env
and document the expected name.

Bookkeeping (required):
- Record START time in the Implementation log now (date, time, timezone).
- Add FINISH time and DURATION when done.
- Write the Implementation summary (what you set up, decisions, gotchas).
- Do NOT git commit. Refine the Suggested commit message; the user commits.
```

## Implementation log (fill this in)

- **Started:** _<!-- YYYY-MM-DD HH:MM TZ -->_
- **Finished:** _<!-- YYYY-MM-DD HH:MM TZ -->_
- **Duration:** _<!-- -->_

## Implementation summary (fill this in)

_<!-- ... -->_

## Suggested commit message

```
build(macos): sign local builds with a stable self-signed identity

Keep the code identity constant across rebuilds so the Accessibility grant
no longer resets every build. Document cert creation and the tccutil reset
recovery step.
```
