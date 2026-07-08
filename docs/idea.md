# JC Grid Manager — Project Document

> _Updated 2026-07-08 after a design-review ("grilling") session. Decisions from that review are folded in below and marked **[review]** where they changed the original plan; the reasoning is kept inline so it isn't lost._

## 0. App Name

**JC Grid Manager**

> **"Grid" here means _directional_ snapping** — halves, thirds, quarters, corners (like Rectangle) — **not** draggable zones (like FancyZones). Worth flagging, because the name implies zones but the model is directional.
>
> Name is **not** cleared for public release: "Grid" is common in this space (an existing macOS app is called "Grid," plus GridMove, WindowGrid). Since v1 is a personal tool shipped by direct download, this only matters if/when it's released publicly — see §11.

## 1. Overview

**JC Grid Manager** is a personal, native menu-bar / system-tray utility for macOS and Windows that snaps, resizes, and repositions windows using global keyboard shortcuts.

**Why it exists [review]:** the author works on both macOS and Windows daily. On macOS, [Rectangle](https://rectangleapp.com/) already solves this perfectly. On Windows, [PowerToys FancyZones](https://learn.microsoft.com/en-us/windows/powertoys/fancyzones) does not — it is built around _dragging windows into predefined zones_, whereas the author wants Rectangle's _stateless directional commands_ (press Left → left half; press again → cycle to two-thirds, then one-third), with the same keys and the same muscle memory on both machines. So this is a deliberate **Rectangle-parity tool brought to Windows**, not a novel product.

**Core principle [review]:** one shared TypeScript UI layer + one shared Rust core that holds **all** the snapping logic, sitting on top of a **thin per-OS shim** that only does raw window I/O. (This boundary was redrawn during review — see §5.3.)

---

## 2. Goals

- One codebase, one feature set, behaving identically on macOS and Windows.
- Match **Rectangle's** feature set and default shortcut scheme (§4), so muscle memory transfers between the author's two machines.
- Global shortcuts identical across OSes, with only the modifier differing: **Control + Option on macOS ↔ Control + Alt on Windows** _(updated in review — the plan previously said "Command," but Rectangle's defaults use Control + Option)_.
- Lives in the menu bar (macOS) / system tray near the clock (Windows); no dock/taskbar icon.
- Optional launch at login.
- Lightweight: fast startup, low idle memory/CPU.
- Distributed by direct download (`.dmg` / `.exe`/`.msi`) — no App Store / Microsoft Store.

## 3. Non-Goals (v1)

- No App Store / Microsoft Store distribution (avoids sandbox limits that block cross-app window control).
- **No paid code signing / notarization [review]** — the app ships unsigned; consequences and mitigations in §8 and §5.6.
- **No auto-update in the baseline [review]** — manual reinstall is the default; OTA is an optional later add (§9).
- No Linux support (the platform shim keeps the door open for later).
- **No cloud sync** — config is local **per machine**, so custom shortcuts do not sync between the author's Mac and PC (a possible future add).
- No visual drag-to-define zone editor (possible v2).

---

## 4. Features & Default Shortcuts (v1)

The full Rectangle menu, on both platforms. **Modifier mapping:** `⌥` (Option) → **Alt** and `⌘` (Command) → **Windows key** on Windows. Example: `⌃⌥←` on macOS = **Ctrl + Alt + ←** on Windows.

Repeating a **directional** shortcut **cycles** through sizes (e.g. Left → ½ → ⅔ → ⅓). See the state machine in §7.

**Halves**

| Action | Shortcut (macOS) | Notes |
| --- | --- | --- |
| Left Half | `⌃⌥←` | cycles ½ → ⅔ → ⅓ |
| Right Half | `⌃⌥→` | cycles ½ → ⅔ → ⅓ |
| Top Half | `⌃⌥↑` | |
| Bottom Half | `⌃⌥↓` | |
| Center Half | — | centered column, half the screen width, full height |

**Corners (quarters)**

| Action | Shortcut | Action | Shortcut |
| --- | --- | --- | --- |
| Top Left | `⌃⌥U` | Top Right | `⌃⌥I` |
| Bottom Left | `⌃⌥J` | Bottom Right | `⌃⌥K` |

**Thirds**

| Action | Shortcut | Action | Shortcut |
| --- | --- | --- | --- |
| First Third | `⌃⌥D` | First Two Thirds | `⌃⌥E` |
| Center Third | `⌃⌥F` | Last Two Thirds | `⌃⌥T` |
| Last Third | `⌃⌥G` | | |

**Sizing**

| Action | Shortcut | Behavior |
| --- | --- | --- |
| Maximize | `⌃⌥↩` | fill the work area |
| Almost Maximize | — | fills **~90%** of the work area, centered _(configurable)_ |
| Maximize Height | `⌃⌥⇧↑` | full height; **keeps** current width & horizontal position |
| Smaller | `⌃⌥-` | shrink by **~5%** of the screen, around the window's center; floors at a minimum _(configurable)_ |
| Larger | `⌃⌥=` | grow by **~5%** of the screen, around the center; caps at full screen _(configurable)_ |
| Center | `⌃⌥C` | center the window at its current size |
| Restore | `⌃⌥⌫` | return the window to its pre-snap size/position |

**Displays**

| Action | Shortcut (macOS) | Notes |
| --- | --- | --- |
| Next Display | `⌃⌥⌘→` | preserve relative size/position |
| Previous Display | `⌃⌥⌘←` | |

> On Windows these become **Ctrl + Alt + Win + ←/→**, which brushes against Windows' own **Win + Arrow** snapping — verify and rebind if it conflicts (see §5.4).

**Sub-menus** _(menu-only — **no default shortcuts**; the user can add their own)_

- **Move to Edge:** slide the window flush to an edge (Left / Right / Up / Down) **without resizing**.
- **Fourths:** four full-height columns, each ¼ of the width (First / Second / Third / Last Fourth).
- **Sixths [review]:** a **3-across × 2-down grid** = six cells (each ⅓ wide, ½ tall).

**Other menu items**

- **Ignore "[App]":** stop managing windows for the currently focused app — the label shows the current app's name (e.g. "Ignore Code" when VS Code is frontmost).
- **Settings…**, **About**, **Quit**.
- _No "Check for Updates…" in v1_ — added only if/when OTA lands (§9).

**Behavior notes [review]**

- **Cycling:** repeating a directional shortcut cycles its sizes; the cycle resets when you act on a different window, or move/resize the current one by hand (see §7).
- **Stubborn windows:** if a window can't be moved/resized (fixed-size, minimum-size, fullscreen, or an admin-elevated window on Windows), the app does its best and plays a **soft beep** if nothing could be done.
- **Every shortcut is rebindable**; the editor **warns** if a combo is already used by another action or is reserved/refused by the OS.

---

## 5. Technology Stack

### 5.1 Framework: Tauri v2

TypeScript UI (settings window, tray menu) + a small, fast Rust core for the window-control logic. Keeps the binary small and avoids shipping a full Chromium runtime (unlike Electron), while building both OSes from one project.

### 5.2 Frontend (TypeScript)

- **React + TypeScript.**
- **Scope:** settings window, shortcut editor/recorder, tray menu labels, first-run/permission screens.
- **Talks to the Rust core via** Tauri's `invoke()` bridge and event system.

### 5.3 Core Logic (Rust) — boundary redrawn [review]

The original plan put actions (`snap_left_half`, …) in the per-OS layer, which would force the geometry + cycling + restore logic to be written **twice**, once per OS. Redrawn so the smart part is shared and the per-OS part is a dumb I/O shim:

**Shared, platform-agnostic core** (all the logic; unit-testable with **no real windows** — see §10):

- **Geometry:** computes target rectangles as **fractions of a display's work area** (0–1), so screen size, DPI, and coordinate systems never enter the math.
- **Snap state machine** (§7): cycling, restore baseline, "did the user move it?" detection.
- **Display selection:** picks the display a window is on by **largest overlap**, not by which display a corner lands in.

**Thin per-OS shim** (`#[cfg(target_os = …)]`) — ~5 methods, no logic:
`focused_window()`, `frame(win)`, `set_frame(win, rect)`, `work_area(win)`, `displays()`, `identity(win)`.

- **macOS:** Accessibility API (`AXUIElement`) via `accessibility-sys` / `cocoa` / `core-graphics`. The shim owns the coordinate conversion: AX positions are top-left origin in **points**, while the work area (`NSScreen.visibleFrame`) is bottom-left origin — the shim converts both into one top-left, fraction-friendly space. Because everything is in **points**, mixing Retina and non-Retina displays needs no special handling.
- **Windows:** Win32 (`GetForegroundWindow`, `GetWindowRect`, `SetWindowPos`, `MonitorFromWindow`, `GetMonitorInfo` → `rcWork`). Must declare **per-monitor-DPI-v2 awareness**, or Win32 reports scaled/virtualized coordinates on mixed-DPI setups (Win32 works in **physical pixels**).

Requires macOS **Accessibility permission** (§8). Windows needs no special permission for user-level windows, but **cannot move elevated/admin windows** unless the app itself runs elevated — which it won't by choice (§8).

### 5.4 Global Shortcuts

- **Plugin:** `tauri-plugin-global-shortcut`.
- Stored in a platform-neutral form; the base modifier resolves to **Control + Option (macOS)** / **Ctrl + Alt (Windows)**.
- Defaults match across platforms; **config is per-machine** (no sync), so a rebind on one machine does not propagate to the other.
- **Known Windows conflicts with Ctrl + Alt [review]:**
  - **Ctrl + Alt + Arrow** rotates the screen on many **Intel-graphics** PCs → clashes with the Half shortcuts. Mitigation: turn those hotkeys off in Intel Graphics settings; rebinding is the fallback.
  - **Ctrl + Alt + letter** equals **AltGr** on some non-US keyboards → may type a character instead. Mitigation: rebind.
- The shortcut editor **validates** bindings: rejects combos already used by another action, and detects combos the OS refuses to register.

### 5.5 Tray / Menu Bar Icon

Built into Tauri (`tauri::tray::TrayIconBuilder`). Renders in the macOS menu bar and the Windows system tray near the clock. The tray menu mirrors §4.

### 5.6 Packaging & Distribution — signing decision [review]

- **Build:** Tauri bundler → `.dmg` (macOS), `.msi` (WiX) and/or `.exe` (NSIS) (Windows), configured in `tauri.conf.json`.
- **No paid code signing in v1** — the author is the only user and isn't buying certificates yet. Consequences:
  - **macOS:** Gatekeeper shows an "unidentified developer" warning on first launch (right-click → **Open** to bypass). More importantly, Accessibility permission is tied to the app's identity, so it **resets on each reinstall/update** — see §8.
  - **Windows:** SmartScreen shows "Windows protected your PC" on first run (**More info → Run anyway**).
  - **Dev workaround:** sign local builds with a **stable self-signed certificate** (free) so macOS keeps recognizing rebuilds and doesn't re-prompt for Accessibility on every build.

---

## 6. Project Structure

```text
jc-grid-manager/
├── src/                          # TypeScript / React frontend
│   ├── components/
│   │   ├── TrayMenu.tsx
│   │   ├── ShortcutEditor.tsx
│   │   ├── SettingsWindow.tsx
│   │   └── FirstRun.tsx          # macOS Accessibility onboarding
│   ├── lib/tauriBridge.ts        # typed wrappers around invoke()
│   └── main.tsx
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs                # command dispatch
│   │   ├── core/                 # shared, platform-agnostic (pure, testable)
│   │   │   ├── geometry.rs       # fraction-based target rectangles
│   │   │   ├── state.rs          # snap state machine (cycle + restore)
│   │   │   └── actions.rs        # action → geometry mapping
│   │   ├── platform/
│   │   │   ├── mod.rs            # Platform trait (the ~5 I/O methods)
│   │   │   ├── macos.rs          # AXUIElement shim   (cfg-gated)
│   │   │   └── windows.rs        # Win32 shim         (cfg-gated)
│   │   ├── shortcuts.rs          # registration + modifier resolution + conflict checks
│   │   └── config.rs            # local per-machine config (JSON/TOML)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── icons/
├── package.json
└── tsconfig.json
```

---

## 7. Architecture Flow & the Snap State Machine

**Flow**

1. **Launch** → Rust core starts, tray icon appears, shortcuts register from saved config (or defaults on first run).
2. **Shortcut pressed** → `tauri-plugin-global-shortcut` fires → resolves to a named action.
3. **Dispatch** → the core reads the focused window + its display's work area (via the shim), runs the state machine, computes the target rect (as fractions), and calls `set_frame`.
4. **Settings change** → frontend `invoke()` → core updates + persists local config → re-registers affected shortcuts.

**The snap state machine** — the part that makes it _feel_ like Rectangle. State is a single remembered record (upgradeable later to a small most-recently-used list): last action, cycle step, the frame we actually set, and a restore baseline.

On a directional shortcut:

- Read the focused window's current frame + its display's work area.
- **Continue the cycle** only if: same window as last time **and** its current frame still ≈ the frame we last set **and** the action matches **and** it's on the same display. Otherwise it's a **fresh grab**.
- **Fresh grab** → capture the current frame as the **restore baseline**, start the cycle at step 0. **Continuing** → advance the cycle, keep the baseline.
- Compute target (fractions of work area) → `set_frame` → **re-read the actual resulting frame** and store _that_ (so terminals / min-size windows that don't land exactly still cycle correctly).

**Restore** returns the window to its baseline and clears the record.

Invalidation is therefore **automatic** — no OS move/resize listeners. If the user drags a window, its frame no longer matches what we set, so the next press simply starts fresh.

---

## 8. Permissions & First-Run

**macOS Accessibility permission** (needed to move other apps' windows):

- Detect with `AXIsProcessTrusted()`. The first-run flow must handle **three** states, not one:
  1. **Never asked** → prompt (`AXIsProcessTrustedWithOptions`) and explain why it's needed.
  2. **Denied** → deep-link to System Settings → Privacy & Security → Accessibility, and explain.
  3. **Should-be-trusted-but-isn't** (the unsigned-app case after a reinstall/update, where the old grant went stale) → guide the user to remove & re-add the entry, or run `tccutil reset Accessibility <bundle-id>` and relaunch.
- Because the app is unsigned, expect to **re-grant after reinstalls/updates** (mitigated in dev by the stable self-signed cert, §5.6).

**Windows:** no special permission for user-level windows. **Admin/elevated windows can't be moved** (the app won't run elevated by choice); they simply won't respond — a **soft beep** signals it.

**Stubborn windows generally** (fixed-size, minimum-size, fullscreen, elevated): best-effort; **soft beep** when nothing could be done.

**Launch at login:** Tauri autostart plugin (`tauri-plugin-autostart`), toggleable in settings.

---

## 9. Distribution & Updates [review]

- Direct download from GitHub Releases / a project site.
- **Updates: manual reinstall from a new installer is the default.** No "Check for Updates…" in the v1 menu.
- **OTA auto-update — optional, later.** The architecture accommodates `tauri-plugin-updater`; the updater's own signing key is **free** (separate from OS code signing).
  - **macOS caveat:** an _unsigned_ auto-update makes the app forget Accessibility permission right after updating, so on macOS the **manual-reinstall default stays** until/unless a Developer ID certificate is obtained. OTA is smoother on Windows.

---

## 10. Testing Approach [review]

- The shared core (geometry + state machine) is **pure logic** — unit-tested with **no real windows**: feed a fake work area + a scripted sequence of `(action, current frame)` and assert the resulting rectangles, cycle steps, and restore behavior.
- The per-OS shim (the ~5 I/O methods) needs **manual smoke testing** on real machines, including **multi-monitor** setups (the highest-risk area — see §11).

---

## 11. Open Questions / Future

- **macOS multi-monitor coordinate math** — the known-hard part; confined to the mac shim, but where bugs will cluster (displays arranged above/left give negative coordinates; mixed sizes/scales).
- **Accessibility re-grant on reinstall** (unsigned) — acceptable for now; revisit if a Developer ID cert is purchased.
- **Windows Ctrl + Alt conflicts** — Intel screen-rotation and AltGr; mitigated by disabling those hotkeys + rebinding.
- **Display-move shortcuts on Windows** — `⌃⌥⌘←/→` maps to Ctrl + Alt + Win, which brushes Windows' own Win + Arrow snapping; verify and rebind if needed.
- **Cross-machine shortcut sync** — out of scope now (config is local); a future cloud-sync could make custom binds match across machines.
- **Visual zone editor, per-app rules, Linux support** — possible v2+ (the shim keeps Linux open).
- **Name clearance** — only if publicly released (§0).

---

## 12. Summary

| Layer | Technology |
| --- | --- |
| Language (shared) | TypeScript (UI) + Rust (core) |
| Framework | Tauri v2 |
| UI | React + TypeScript |
| Core logic | Rust — **shared pure core** (geometry + state machine) over a **thin per-OS I/O shim** |
| macOS window control | Accessibility API (`AXUIElement`) via `cocoa` / `core-graphics` / `accessibility-sys` |
| Windows window control | Win32 via `windows-rs` (per-monitor-DPI-v2 aware) |
| Global shortcuts | `tauri-plugin-global-shortcut` — **⌃⌥ on macOS / Ctrl+Alt on Windows**, fully rebindable |
| Tray icon | Tauri built-in tray API |
| Packaging | Tauri bundler → `.dmg` (macOS), `.msi`/`.exe` (Windows) — **unsigned in v1** |
| Distribution | Direct download; **manual reinstall** default, **optional OTA** later |
