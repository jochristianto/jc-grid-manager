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

- **Started:** 2026-07-08 17:06 WIB (+0700)
- **Finished:** 2026-07-08 17:09 WIB (+0700)
- **Duration:** ~3 minutes of wall-clock agent time (most of the effort was source-diving the Tauri v2 CLI to confirm exactly what `APPLE_SIGNING_IDENTITY` does and doesn't cover; excludes the user's own one-time manual cert-creation step, which is not yet done)

## Implementation summary

Documentation-only change (per the issue's own recommendation: env var, not
committed config). No files under `src-tauri/` or `src/` were touched — that
tree had concurrent, unrelated Rust changes in flight at the time.

**What was set up:**

- `docs/dev-signing.md` (new) — the reproducible guide:
  - Cert creation via Keychain Access GUI (Certificate Assistant → Create a
    Certificate…, type **Code Signing**, name **`JC Grid Manager Dev`**) and
    an equivalent scripted `openssl` + `security import` + `security
    add-trusted-cert` CLI flow, plus a `security find-identity -v -p
    codesigning` / `codesign -dvv` verification step.
  - How `APPLE_SIGNING_IDENTITY` is read by the Tauri CLI at build time and
    overrides `tauri.conf.json > bundle > macOS > signingIdentity` (so no
    config file needs to change), and how to load it from `.env` via `set -a;
    source .env; set +a` before `pnpm tauri build`.
  - The `tccutil reset Accessibility com.jochristianto.jcgridmanager`
    recovery escape hatch (cross-referenced from issue 030's onboarding
    flow).
- `.env.example` (new) — documents `APPLE_SIGNING_IDENTITY="JC Grid Manager
  Dev"` as the expected shape; no machine-specific value is committed.
- `.gitignore` — already ignored `.env`/`.env.*` with a `!.env.example`
  carve-out before this change; verified with `git check-ignore` and left
  untouched (no edit needed).
- `README.md` — added a one-line pointer to `docs/dev-signing.md` under a new
  "macOS dev signing" heading.

**Decisions:**

- Env var over config, per the issue's own recommendation — `tauri.conf.json`
  was not touched (and per the task constraints, would not have been even if
  it seemed necessary; it didn't).
- Documented both the GUI and CLI cert-creation paths in full, as requested,
  rather than picking one.

**Gotcha found during research (important for whoever verifies this):**
`APPLE_SIGNING_IDENTITY` is only consumed by Tauri's bundler code path
(`tauri build` / `tauri build --debug` / `tauri bundle`) — confirmed by
reading the Tauri v2 CLI source (`crates/tauri-cli/src/interface/rust.rs`
reads the env var inside bundle-settings construction; `crates/tauri-cli/
src/interface/rust/desktop.rs`'s `run_dev`/`cargo_command` shells out to
`cargo run` directly and never touches the `tauri-macos-sign`/bundler code).
**`pnpm tauri dev` does not go through that path at all**, so setting the env
var alone does not sign whatever `tauri dev` launches. The doc recommends
verifying Accessibility-dependent behavior via `pnpm tauri build --debug` +
launching the resulting `.app` (path documented, matches the real
`productName`, `jc-grid-manager`, not the issue's illustrative "JC Grid
Manager.app") instead of `pnpm tauri dev`, and notes a possible future fix
(a Cargo `runner` wrapper in `src-tauri/.cargo/config.toml`) without
implementing it, since that would touch `src-tauri/` and is out of scope
here. This is flagged clearly in `docs/dev-signing.md` so it isn't lost.

**Not done (deliberately):** did not create the certificate (user's one-time
manual step), did not run any `tauri`/`cargo` commands, did not edit
`tauri.conf.json` or anything under `src-tauri/`/`src/`, did not check off
acceptance criteria above (they depend on the user's manual verification) or
touch `docs/issues/README.md`.

## Suggested commit message

```
docs(macos): document a stable self-signed dev code-signing identity

Add docs/dev-signing.md covering certificate creation (Keychain Access GUI
and an equivalent openssl/security CLI flow), wiring the identity via the
APPLE_SIGNING_IDENTITY env var (.env.example + existing .gitignore rule,
nothing hardcoded/committed), and the tccutil reset recovery escape hatch.
Notes that `pnpm tauri dev` doesn't consume the env var today (only
`tauri build`/`bundle` do) and recommends `pnpm tauri build --debug` for
verifying Accessibility-gated behavior in the meantime. No src-tauri/ or
tauri.conf.json changes.
```
