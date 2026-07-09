# JC Grid Manager

A personal, native **menu-bar / system-tray** window manager for **macOS and Windows** that snaps,
resizes, and repositions windows with global keyboard shortcuts.

It brings [Rectangle](https://rectangleapp.com/)'s _stateless directional snapping_ — press **Left**
for the left half, press again to cycle to two-thirds, then one-third — to Windows, with the same
keys and the same muscle memory on both machines. This is deliberately a **Rectangle-parity tool**,
not a drag-into-zones tool like PowerToys FancyZones.

> "Grid" here means **directional** snapping (halves, thirds, quarters, corners), **not** draggable
> zones. See [`docs/idea.md`](docs/idea.md) for the full design document and the reasoning behind
> each decision.

## Highlights

- One codebase, one feature set — behaves identically on macOS and Windows.
- Full Rectangle menu: halves, thirds, quarters, fourths, sixths, sizing (maximize / grow / shrink /
  center / restore), move-to-edge, and multi-display moves.
- **Cycling shortcuts** — repeating a directional key cycles its sizes (½ → ⅔ → ⅓).
- Lives in the menu bar / tray with **no dock or taskbar icon**; optional launch at login.
- Every shortcut is **rebindable**, with conflict/OS-reserved validation.
- Lightweight: fast startup, low idle memory/CPU — no bundled Chromium (Tauri, not Electron).

## Default shortcuts

Global shortcuts are identical across platforms; only the base modifier differs:
**Control + Option on macOS ↔ Ctrl + Alt on Windows**. `⌘` (Command) maps to the **Windows key**.

| Action | macOS | Windows |
| --- | --- | --- |
| Left / Right / Top / Bottom Half | `⌃⌥` + arrow | `Ctrl+Alt` + arrow |
| Corners (quarters) | `⌃⌥` U / I / J / K | `Ctrl+Alt` U / I / J / K |
| Thirds | `⌃⌥` D / F / G (+ E / T) | `Ctrl+Alt` D / F / G (+ E / T) |
| Maximize | `⌃⌥↩` | `Ctrl+Alt+Enter` |
| Center / Restore | `⌃⌥C` / `⌃⌥⌫` | `Ctrl+Alt+C` / `Ctrl+Alt+Backspace` |
| Next / Previous Display | `⌃⌥⌘←/→` | `Ctrl+Alt+Win+←/→` |

The full table (including fourths, sixths, move-to-edge, and sizing tunables) lives in
[`docs/idea.md` §4](docs/idea.md). Rebind anything from the **Shortcuts** tab in Settings.

## Tech stack

| Layer | Technology |
| --- | --- |
| Framework | [Tauri v2](https://tauri.app/) |
| UI | React + TypeScript (Vite) |
| Core logic | Rust — a shared, pure, unit-testable core (geometry + snap state machine) over a thin per-OS I/O shim |
| macOS window control | Accessibility API (`AXUIElement`) via `accessibility-sys` / `cocoa` / `core-graphics` |
| Windows window control | Win32 via `windows-rs` (per-monitor-DPI-v2 aware) |
| Global shortcuts | `tauri-plugin-global-shortcut` |
| Packaging | Tauri bundler → `.dmg` (macOS), `.msi` / WiX (Windows) — **unsigned in v1** |

## Project structure

```text
src/                       # React + TypeScript frontend (settings, shortcut editor, first-run)
  components/              # SettingsWindow, ShortcutEditor, GeneralSection, FirstRun
  lib/                     # typed invoke() wrappers + formatting helpers
src-tauri/
  src/
    core/                  # shared, platform-agnostic logic (geometry, state machine, actions)
    platform/              # thin per-OS I/O shim (macos.rs / windows.rs, cfg-gated)
    shortcuts.rs           # registration + modifier resolution + conflict checks
    config.rs              # local per-machine config
    tray.rs                # tray / menu-bar icon + menu
  tauri.conf.json          # bundle + window config
docs/                      # idea.md (design), install.md, dev-signing.md, issues/
```

## Prerequisites

- **[Node.js](https://nodejs.org/)** 18+ and **[pnpm](https://pnpm.io/)** (`corepack enable` gives
  you pnpm) — the repo uses `pnpm` and ships a `pnpm-lock.yaml`.
- **[Rust](https://www.rust-lang.org/tools/install)** (stable) via `rustup`.
- **Tauri v2 system dependencies** for your OS — see
  [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/):
  - **macOS:** Xcode Command Line Tools (`xcode-select --install`).
  - **Windows:** Microsoft C++ Build Tools + the WebView2 runtime (preinstalled on Windows 11).

Install JavaScript dependencies once:

```sh
pnpm install
```

## Run the dev app

Launches Vite for the UI and the Rust core together, with hot-reload on the frontend:

```sh
pnpm tauri dev
```

The window opens for development; in normal use the app is tray-only (see below). On **macOS** you'll
be prompted for **Accessibility permission** on first run — grant it under **System Settings →
Privacy & Security → Accessibility** so the app can move other apps' windows.

> **macOS dev signing (recommended).** Because the app is unsigned, macOS otherwise forgets the
> Accessibility grant on every rebuild. Sign local builds with a stable **self-signed** identity so
> the grant survives. One-time setup and the caveats are in
> [`docs/dev-signing.md`](docs/dev-signing.md); it reads `APPLE_SIGNING_IDENTITY` from a local `.env`
> (copy [`.env.example`](.env.example)).

### Frontend only

To iterate on the UI in a browser without the Rust core, run Vite directly:

```sh
pnpm dev            # http://localhost:1420
```

## Build the app

Produces optimized installers via the Tauri bundler. The `targets` array in
[`tauri.conf.json`](src-tauri/tauri.conf.json) is `["app", "dmg", "msi"]`; Tauri applies it on every
platform but skips targets that don't fit the host, so you get a **`.dmg` on macOS** and an
**`.msi` on Windows** from the same config:

```sh
pnpm tauri build
```

Artifacts land under `src-tauri/target/release/bundle/`:

- **macOS:** `dmg/JC Grid Manager_<version>_<arch>.dmg` (and the `.app` under `macos/`).
- **Windows:** `msi/JC Grid Manager_<version>_<arch>_<lang>.msi`.

Build for a single platform by running the command on that OS (there is no cross-compilation here).
The Windows `.msi` uses the [WiX Toolset](https://wixtoolset.org/), which the Tauri CLI downloads
automatically on first build. On macOS, set `APPLE_SIGNING_IDENTITY` (see above) to sign the build
with your dev certificate.

> v1 ships **unsigned** — no paid Apple Developer ID / notarization and no Windows Authenticode. Both
> OSes show a one-time "unknown developer" warning on first launch; there is no auto-updater.

## Releasing (manual)

There's no CI — releases are cut by hand. The automated GitHub Actions build was removed because
signing needs a paid developer account this project doesn't use yet, so a release is just a local
build you publish yourself:

1. **Bump the version** in lockstep across [`package.json`](package.json),
   [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json), and
   [`src-tauri/Cargo.toml`](src-tauri/Cargo.toml), and summarize the changes in
   [`CHANGELOG.md`](CHANGELOG.md). (`src-tauri/Cargo.lock` isn't touched by the bump; `cargo` refreshes
   its version field on the next build.)
2. **Build the installers** on each OS with `pnpm tauri build` (see [Build the app](#build-the-app)):
   a `.dmg` on macOS and an `.msi` on Windows.
3. **Publish** by creating a GitHub Release + `vX.Y.Z` tag and attaching those artifacts yourself.

## Install & first run

End-user install steps (getting past Gatekeeper / SmartScreen, granting Accessibility, the
unsigned-app re-grant caveat, and Windows shortcut conflicts) are in
[`docs/install.md`](docs/install.md).

## Documentation

- [`docs/idea.md`](docs/idea.md) — full design document (goals, features, architecture, state machine).
- [`docs/install.md`](docs/install.md) — install & first-run guide for end users.
- [`docs/dev-signing.md`](docs/dev-signing.md) — stable self-signed dev certificate for macOS.
- [`docs/issues/`](docs/issues/) — fine-grained implementation issues.

## Recommended IDE setup

[VS Code](https://code.visualstudio.com/), the
[Tauri extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode), and
[rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
