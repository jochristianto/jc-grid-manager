# 031 — Packaging: unsigned `.dmg` / `.msi` / `.exe`

| | |
|---|---|
| **Issue ID** | 031 |
| **Layer** | packaging |
| **Depends on** | broad — do this **last**, once macOS (001–024) and Windows (025–027) are working |
| **Blocks** | — |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §2, §3, §5.6, §9 |
| **Status** | ☐ Not started |

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

## Implementation log (fill this in)

- **Started:** _<!-- -->_
- **Finished:** _<!-- -->_
- **Duration:** _<!-- -->_

## Implementation summary (fill this in)

_<!-- ... -->_

## Suggested commit message

```
build: package unsigned .dmg and Windows installer, tray-only

Configure the Tauri bundler for a .dmg and an NSIS .exe (unsigned, v1), verify
tray-only operation with no dock/taskbar icon, and document the Gatekeeper /
SmartScreen bypass and macOS Accessibility caveat. No OTA.
```
