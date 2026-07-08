# Installing JC Grid Manager

JC Grid Manager is a **menu-bar / system-tray** window manager for macOS and Windows. There is no
dock or taskbar icon — after launch it lives in the menu bar (macOS) or the system tray
(Windows), and every action is reachable from that icon's menu or via keyboard shortcuts.

These are **unsigned personal builds** (v1), so both OSes show a one-time "unknown developer"
warning on first launch. The steps below get past it. There is **no auto-update** — to upgrade,
download the newer build and reinstall.

## Download

Grab the artifact for your OS from the release you were given:

- **macOS:** `JC Grid Manager_<version>_<arch>.dmg`
- **Windows:** `JC Grid Manager_<version>_<arch>-setup.exe` (NSIS installer)

## macOS

1. Open the `.dmg` and drag **JC Grid Manager** into **Applications**.
2. **First launch (Gatekeeper).** Double-clicking shows *"JC Grid Manager can't be opened because
   it is from an unidentified developer."* This is expected for an unsigned app. Instead:
   **right-click (or Control-click) the app → Open → Open.** You only need to do this once.
3. **Grant Accessibility.** The app needs Accessibility permission to move other apps' windows.
   On first run it opens an onboarding screen and prompts you; approve **JC Grid Manager** under
   **System Settings → Privacy & Security → Accessibility**. The onboarding screen closes itself
   once permission is granted.
4. **After an update or reinstall (unsigned-app caveat).** Because the build is unsigned, macOS can
   keep a **stale** Accessibility entry that looks enabled but no longer works (see
   [issue 002](issues/002-self-signed-dev-cert.md) for the dev-time mitigation). If snapping stops
   working after an update, either remove **JC Grid Manager** from the Accessibility list and add
   it back, or reset the grant from Terminal and relaunch:

   ```sh
   tccutil reset Accessibility com.jochristianto.jcgridmanager
   ```

## Windows

1. Run the `-setup.exe` installer.
2. **SmartScreen.** Windows shows *"Windows protected your PC."* This is expected for an unsigned
   app. Click **More info → Run anyway**, then follow the installer.
3. No special permission is required to manage normal windows. **Admin/elevated windows** can't be
   moved by a non-elevated app — acting on one plays a soft beep instead of doing nothing.
4. **Shortcut conflicts.** Some `Ctrl+Alt` combos collide with Windows/Intel-graphics behavior
   (screen rotation, AltGr). The **Shortcuts** tab shows the specifics and lets you rebind any
   action. See [issue 027](issues/027-windows-specifics-conflicts.md).

## Notes

- **Tray-only.** Closing the settings window doesn't quit the app — it keeps running in the
  menu-bar/tray. Quit from the tray menu's **Quit** item.
- **Local config, no sync.** Settings (shortcut rebinds, sizing tunables, ignore list, launch-at-
  login) are stored per machine in the OS app-config directory and are hand-editable as a fallback.
- **No auto-update / telemetry.** v1 ships no updater and no "Check for Updates…". Download and
  reinstall to upgrade.
