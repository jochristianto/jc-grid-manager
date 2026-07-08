# 031 — Packaging: unsigned `.dmg` / `.msi` / `.exe`

| | |
|---|---|
| **Issue ID** | 031 |
| **Layer** | packaging |
| **Depends on** | broad — do this **last**, once macOS (001–024) and Windows (025–027) are working |
| **Blocks** | — |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §2, §3, §5.6, §9 |
| **Status** | ☑ Done |

## Summary

Produce installable artifacts via the Tauri bundler: **`.dmg`** (macOS) and **`.msi`** (WiX) and/or **`.exe`** (NSIS) (Windows), **unsigned** in v1 (§3/§5.6). Document the expected Gatekeeper/SmartScreen first-run prompts and how to get past them. **No auto-update** in the baseline (§3/§9).

## Context (self-contained)

- Bundler config lives in `src-tauri/tauri.conf.json` (`bundle` block; currently `targets: "all"`).
- **Unsigned** (§5.6): macOS shows an "unidentified developer" warning (right-click → **Open** to bypass); Windows SmartScreen shows "Windows protected your PC" (**More info → Run anyway**).
- The macOS **Accessibility-grant-resets-on-reinstall** issue is inherent to unsigned apps; the dev-time mitigation (stable self-signed cert) is issue 002 — reference it, but release artifacts stay unsigned in v1.
- The app is a **menu-bar/tray utility with no dock/taskbar icon** (§2) — ensure the bundle/activation policy reflects that (e.g. macOS `LSUIElement`/accessory activation, no taskbar window on Windows) if not already handled elsewhere.
- **No "Check for Updates…"** and no `tauri-plugin-updater` in v1 (§9); leave the door open but don't wire OTA.

## What to build

1. Configure `bundle` targets explicitly: `.dmg` on macOS; `.msi` and/or `.exe` on Windows (pick per §5.6 — see decision). Set product name, identifier (already `com.jochristianto.jcgridmanager`), version, and icons (already present).
2. Verify the built app runs as a tray-only utility (no dock/taskbar entry) from a clean install.
3. Write install docs (`docs/install.md` or README section): where to download, the Gatekeeper and SmartScreen bypass steps, and the macOS Accessibility grant + re-grant-after-update caveat.
4. Confirm **no** updater/OTA is bundled.

## Open decisions (recommendation)

- **Windows: `.msi` vs `.exe` vs both** — WiX MSI is more "corporate"; NSIS EXE is lighter and common for direct-download tools. _Recommendation:_ ship the **NSIS `.exe`** for a personal direct-download tool (simpler), and optionally the MSI too. **Confirm with the user.**
- **macOS activation policy** — ensure accessory/agent mode so there's no dock icon; may already be set. _Recommendation:_ verify and set `LSUIElement`/accessory if a dock icon still appears.

## Acceptance criteria

- [ ] `tauri build` produces a `.dmg` on macOS and a Windows installer (`.exe` and/or `.msi`) that install and launch the app.
- [ ] The installed app runs tray-only (no dock/taskbar icon) and its shortcuts work after install.
- [ ] Install docs cover the Gatekeeper + SmartScreen bypass and the macOS Accessibility caveat.
- [ ] No auto-update / "Check for Updates…" is present.

## Testing

- Manual: build on each OS; install from the artifact on a clean machine/user; confirm tray-only operation and that a couple of shortcuts work; walk the documented first-run bypass steps.

## LLM prompt

```text
You are implementing issue 031 for JC Grid Manager (PACKAGING). Read docs/idea.md §2/§3/§5.6/§9 and
docs/issues/031-packaging-unsigned-bundles.md. The macOS and Windows features are already working.

Task: Configure the Tauri bundler to produce unsigned installable artifacts — .dmg on macOS and a Windows
installer (.exe via NSIS and/or .msi via WiX; recommend NSIS .exe for a direct-download tool — confirm with
the user). Verify the installed app runs tray-only with no dock/taskbar icon (set macOS accessory/LSUIElement
if needed). Write install docs covering the Gatekeeper ("unidentified developer" → right-click Open) and
SmartScreen ("More info → Run anyway") bypasses and the macOS Accessibility re-grant-after-update caveat.
Confirm NO updater/OTA is bundled (out of v1).

Definition of done: acceptance criteria pass; artifacts build, install, and run tray-only; docs written; no OTA.

Constraints: UNSIGNED in v1 — do not add paid Developer ID/notarization. Do not wire tauri-plugin-updater.
Scope is packaging + install docs.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary (which
Windows target(s) you shipped and any activation-policy changes); do NOT git commit; refine the Suggested
commit message; the user commits.
```

## Implementation log

- **Started:** 2026-07-08 22:34 WIB
- **Finished:** 2026-07-08 22:44 WIB
- **Duration:** ~10m hands-on (incl. a full macOS release bundle build)

## Implementation summary

Configured the Tauri bundler to produce unsigned installers, made the app a true tray-only utility
(no dock/taskbar icon, close-to-hide), wrote install docs, and confirmed no OTA.

**What changed**
- **`tauri.conf.json`**:
  - `productName` → **"JC Grid Manager"** (proper display name; identifier unchanged, so the
    `tccutil` command in 030/install docs still matches `com.jochristianto.jcgridmanager`).
  - `bundle.targets` → explicit **`["app", "dmg", "nsis"]`** (macOS `.dmg`; Windows **NSIS `.exe`**
    per the recommendation — not WiX MSI). Icons/version already present.
  - Main window: title "JC Grid Manager" + **`skipTaskbar: true`** (no Windows taskbar button).
- **`lib.rs`** (tray-only behavior, issue 031):
  - macOS **accessory activation policy** (`set_activation_policy(ActivationPolicy::Accessory)`,
    macOS-gated) → **no dock icon**.
  - **Close-to-hide**: a `CloseRequested` handler on the main window calls `prevent_close()` +
    `hide()`, so closing the settings window leaves the app running in the tray (Quit lives in the
    tray menu) instead of terminating.
- **`docs/install.md`** — download, the macOS Gatekeeper ("unidentified developer" → right-click →
  Open) and Windows SmartScreen ("More info → Run anyway") bypasses, the Accessibility grant +
  unsigned-app **stale-grant / `tccutil reset`** caveat, tray-only notes, and "no auto-update".
- **No updater**: confirmed `tauri-plugin-updater` is absent and no updater config is present.

**Key decisions / deviations — please confirm**
- **Windows target = NSIS `.exe` only** (not MSI), per the issue's recommendation for a
  direct-download personal tool. If you want the corporate-friendly `.msi` too, add `"msi"` to
  `bundle.targets`. **Confirm the NSIS-vs-MSI choice.**
- Kept the window `visible` at launch (so 030's onboarding can show on first run). A "hide until
  the tray/onboarding needs it" refinement is a possible follow-up spanning 024/028/030; out of
  scope for packaging.

**Verification**
- macOS backend compiles + `cargo clippy` clean with the activation-policy / close-to-hide code.
- **Ran a full `pnpm tauri build` on macOS** → success (release build 21.6s + bundle): produced
  `JC Grid Manager.app` and **`JC Grid Manager_0.1.0_aarch64.dmg`** (~3.4 MB), unsigned, with no
  signing/notarization prompts. Confirms the bundle config (targets, product name, icons) is valid.
- **Not run here (needs a Windows machine):** the NSIS `.exe` build + install, SmartScreen bypass,
  and tray-only behavior on Windows. And on both OSes, a clean-machine install + launch smoke.
  Flagged for the user.

## Suggested commit message

```
build: package unsigned .dmg + NSIS installer, tray-only

Set explicit bundle targets (app/dmg/nsis), a proper productName, and skipTaskbar.
Make the app tray-only: macOS accessory activation (no dock icon) and close-to-hide
so closing the settings window keeps it running in the tray. Add docs/install.md
covering the Gatekeeper / SmartScreen bypass and the macOS Accessibility (stale
grant / tccutil reset) caveat. No updater/OTA. macOS .dmg build verified.
```
