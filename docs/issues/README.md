# JC Grid Manager — Implementation Issues

Vertical-slice issues derived from [`docs/idea.md`](../idea.md). Each file is **self-contained**: an LLM agent can pick up a single issue, implement just that slice, and stop.

## How an agent should use these

1. Pick an issue whose **Depends on** items are all done.
2. Follow its **LLM prompt**. Implement only that slice — do not expand scope into other issues.
3. Fill in the **Implementation log** (start time / finish time / duration) and the **Implementation summary** inside the file.
4. **Do not `git commit`.** Refine the **Suggested commit message** in the file to match what you actually did. The user commits each issue individually.

## Conventions

- **Layer** — `core` = shared, platform-agnostic Rust logic (pure, unit-testable); `macos` / `windows` = per-OS I/O shim; `frontend` = React/TypeScript UI (own files, could move to a separate repo); `dev` / `packaging` = tooling.
- **Fractions** — window geometry is expressed as `(x, y, w, h)` fractions of a display's **work area**, each in `0.0..=1.0`, per §5.3. Screen size / DPI / origin never enter the math.
- **Base modifier** — Control+Option on macOS ↔ Ctrl+Alt on Windows. `⌘`/Command maps to the **Windows key**. Shortcuts below are written in macOS form.

## Recommended order

Foundation **001–006** first. Then the action slices **007–019** in any order (they only need the foundation). Then system integration **020–024**, then Windows **025–027**, then frontend **028–030**, then packaging **031**. Do **002 (self-signed dev cert)** early — it stops the macOS Accessibility grant from resetting on every rebuild.

## Issue index

| ID | Title | Layer | Default shortcut | Depends on |
|----|-------|-------|------------------|-----------|
| [001](001-prefactor-shared-core-and-platform-trait.md) | Prefactor: shared core + `Platform` trait + test harness | core | — | None |
| [002](002-self-signed-dev-cert.md) | Self-signed dev cert (stable macOS identity) | dev | — | None |
| [003](003-display-selection-largest-overlap.md) | Display selection by largest overlap | core/macos | — | 001 |
| [004](004-action-registry-and-dispatcher.md) | Action registry + shortcut dispatcher + default binds | core | — | 001 |
| [005](005-snap-state-machine-cycling.md) | Snap state machine (cycling + restore baseline) | core | ⌃⌥←/→ cycle | 004 |
| [006](006-restore-action.md) | Restore action | core | ⌃⌥⌫ | 005 |
| [007](007-center-half.md) | Center Half | core | menu-only | 004 |
| [008](008-thirds.md) | Thirds — First / Center / Last | core | ⌃⌥D/F/G | 004 |
| [009](009-two-thirds.md) | Two-Thirds — First / Last | core | ⌃⌥E/T | 004 |
| [010](010-corners-quarters.md) | Corners / Quarters | core | ⌃⌥U/I/J/K | 004 |
| [011](011-maximize.md) | Maximize | core | ⌃⌥↩ | 004 |
| [012](012-almost-maximize.md) | Almost Maximize | core | menu-only | 004 |
| [013](013-maximize-height.md) | Maximize Height | core | ⌃⌥⇧↑ | 004 |
| [014](014-center.md) | Center | core | ⌃⌥C | 004 |
| [015](015-smaller-larger.md) | Smaller / Larger | core | ⌃⌥- / ⌃⌥= | 004 |
| [016](016-move-to-edge.md) | Move to Edge | core | menu-only | 004 |
| [017](017-fourths.md) | Fourths | core | menu-only | 004 |
| [018](018-sixths.md) | Sixths (3×2 grid) | core | menu-only | 004 |
| [019](019-next-previous-display.md) | Next / Previous Display | core | ⌃⌥⌘←/→ | 003, 004 |
| [020](020-config-persistence-and-rebinding.md) | Config persistence + rebinding + conflict validation | core | — | 004 |
| [021](021-soft-beep-stubborn-windows.md) | Soft beep + stubborn-window handling | core | — | 004 |
| [022](022-ignore-app-list.md) | Ignore-app list | core | — | 004, 020 |
| [023](023-launch-at-login.md) | Launch at login (autostart) | core | — | 004 |
| [024](024-tray-menu.md) | Tray menu mirroring §4 | core | — | 005–019, 022 |
| [025](025-windows-win32-core-shim.md) | Windows Win32 core shim | windows | — | 001 |
| [026](026-windows-dpi-multimonitor.md) | Windows per-monitor-DPI-v2 + multi-monitor | windows | — | 025 |
| [027](027-windows-specifics-conflicts.md) | Windows specifics (elevated beep + Ctrl+Alt conflicts) | windows | — | 025, 020, 021 |
| [028](028-frontend-settings-window.md) | [Frontend] Settings window shell + navigation | frontend | — | 020 |
| [029](029-frontend-shortcut-editor.md) | [Frontend] Shortcut editor / recorder | frontend | — | 020, 028 |
| [030](030-frontend-first-run-onboarding.md) | [Frontend] First-run / Accessibility onboarding | frontend | — | 001, 028 |
| [031](031-packaging-unsigned-bundles.md) | Packaging: unsigned `.dmg` / `.msi` / `.exe` | packaging | — | broad (last) |

## Status board

Move an issue's box as it progresses. `☐` not started · `◐` in progress · `☑` done.

- Foundation: ☑ 001 · ☑ 002 · ☑ 003 · ☑ 004 · ☑ 005 · ☑ 006
- Actions: ☑ 007 · ☑ 008 · ☑ 009 · ☑ 010 · ☐ 011 · ☐ 012 · ☐ 013 · ☐ 014 · ☐ 015 · ☐ 016 · ☐ 017 · ☐ 018 · ☐ 019
- System: ☐ 020 · ☐ 021 · ☐ 022 · ☐ 023 · ☐ 024
- Windows: ☐ 025 · ☐ 026 · ☐ 027
- Frontend: ☐ 028 · ☐ 029 · ☐ 030
- Packaging: ☐ 031
