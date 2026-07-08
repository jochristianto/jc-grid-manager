# 027 — Windows specifics: elevated-window beep + Ctrl+Alt conflicts

| | |
|---|---|
| **Issue ID** | 027 |
| **Layer** | windows (per-OS I/O shim) + config |
| **Depends on** | 025, 020, 021 |
| **Blocks** | — |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §4, §5.4 (Windows conflicts), §8, §11 |
| **Status** | ☑ Done (type-checked for Windows; hardware smoke + display-move decision pending) |

## Summary

Handle the Windows-only rough edges: (1) **admin/elevated windows can't be moved** by a non-elevated app — detect that and play the soft beep (021) rather than failing silently; (2) the default **Ctrl+Alt** binds collide with known Windows behaviors — surface guidance and make rebinding easy.

## Context (self-contained)

- **Elevated windows** (§8): the app won't run elevated by choice, so `SetWindowPos` on an admin window silently no-ops. Detect (e.g. the call reports failure / no effect) and route through the 021 beep.
- **Ctrl+Alt conflicts** (§5.4/§11):
  - **Ctrl+Alt+Arrow** rotates the screen on many **Intel-graphics** PCs → clashes with the Half shortcuts. Mitigation: user turns those hotkeys off in Intel Graphics settings; rebinding is the fallback.
  - **Ctrl+Alt+letter = AltGr** on some non-US keyboards → may type a character. Mitigation: rebind.
  - **Ctrl+Alt+Win+←/→** (Next/Prev Display) brushes Windows' own **Win+Arrow** snapping (§11) → verify; rebind if it conflicts.
- The rebinding machinery already exists (020); this slice provides **Windows-aware guidance/warnings**, not new rebinding infra.

## What to build

1. **Elevated/unmovable detection** on Windows → call the 021 beep (fill the Windows beep stub with `MessageBeep`).
2. **Conflict guidance surfaced to the user**: when a default Ctrl+Alt bind fails to register or is known-conflicting on Windows, expose a warning (via the 020 validation path / a startup notice) pointing at the Intel-rotation and AltGr mitigations, with a one-click path to rebind (frontend 029 renders it; backend provides the signal + text here).
3. **Verify the display-move conflict**: test `Ctrl+Alt+Win+←/→` vs Windows snapping; document the result and, if it conflicts, ship a Windows-specific default or a prominent rebind hint.

## Open decisions (recommendation)

- **Alternate Windows defaults vs keep-and-warn** — change some Windows default binds to dodge conflicts, or keep parity and warn. _Recommendation:_ keep cross-platform parity by default (the whole point is identical muscle memory, §2) and **warn + offer rebind**; only change a default if a conflict is unavoidable/unfixable. **Confirm with the user.**

## Acceptance criteria

- [ ] Acting on an elevated/admin window on Windows plays the soft beep (021) instead of silently doing nothing; the app never crashes on such windows.
- [ ] The Windows beep stub from 021 is implemented (`MessageBeep`).
- [ ] Known Ctrl+Alt conflicts produce a clear, actionable warning (Intel rotation / AltGr) with a rebind path.
- [ ] The `Ctrl+Alt+Win+←/→` display-move conflict is tested and the outcome documented (kept-with-warning or rebound).

## Testing

- Manual on Windows: run something as admin (e.g. an elevated Terminal), try to snap it → beep, no crash. On an Intel-graphics PC (or simulate), confirm the arrow-rotation conflict guidance appears. Try the display-move keys and note behavior.

## LLM prompt

```text
You are implementing issue 027 for JC Grid Manager. Read docs/idea.md §4, §5.4, §8, §11 and
docs/issues/027-windows-specifics-conflicts.md. Issues 025 (Windows shim), 020 (config/validation), and
021 (soft beep) are done.

Task: Handle Windows-specific rough edges. (1) Detect elevated/unmovable windows and route them through
the soft beep; implement the Windows MessageBeep stub left by 021. (2) Surface actionable guidance for the
known Ctrl+Alt conflicts (Intel screen rotation, AltGr) via the 020 validation/startup-notice path, with a
rebind hook for the frontend (029). (3) Test Ctrl+Alt+Win+←/→ vs Windows' Win+Arrow snapping and document
the result; keep parity + warn unless a conflict is unavoidable. Confirm the keep-vs-rebind default with
the user.

Definition of done: acceptance criteria pass; elevated windows beep without crashing; conflict guidance is
present; the display-move conflict outcome is documented.

Constraints: reuse 020's rebinding infra — don't build new rebinding. Default to cross-platform parity +
warnings over changing Windows defaults. Scope is Windows specifics only.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary (incl.
the display-move conflict finding and the keep-vs-rebind decision); do NOT git commit; refine the Suggested
commit message; the user commits.
```

## Implementation log

- **Started:** 2026-07-08 22:27 WIB
- **Finished:** 2026-07-08 22:33 WIB
- **Duration:** ~6m hands-on (excludes reading/design)

## Implementation summary

Handled the Windows rough edges: elevated/unmovable windows now beep instead of failing silently,
and the known Ctrl+Alt conflicts are surfaced as actionable warnings in the shortcut editor.

**What changed**
- **Windows beep (`MessageBeep`)** — filled 021's Windows stub: `platform/windows.rs::beep()` →
  `MessageBeep(MB_OK)`; `platform/mod.rs` now cfg-selects it (macOS `NSBeep` / Windows
  `MessageBeep` / silent elsewhere). `MessageBeep` lives in
  `Win32::System::Diagnostics::Debug` — added that windows feature.
- **Elevated-window handling — no new detection needed.** A non-elevated app's `SetWindowPos` on
  an admin window silently no-ops (UIPI), so `set_frame` returns `Ok` but the frame doesn't change;
  021's existing effect check (`is_effective`) already sees "no change" → `Ok(false)` → `notify_no_op`
  → now an audible `MessageBeep`. So implementing the beep *is* the elevated fix; documented in the
  `beep` doc-comment. No crash (the read still works; the write just doesn't take).
- **Ctrl+Alt conflict guidance** — new `get_platform_notices() -> PlatformNotice[]` command
  (`config.rs`): on **Windows** returns three warnings (Intel Ctrl+Alt+Arrow screen rotation; AltGr
  on non-US keyboards; the display-move-vs-Windows-snapping adjacency), each pointing at the fix +
  "rebind below"; **empty on macOS**. The frontend `ShortcutEditor` (029) fetches it and renders a
  banner above the shortcut list — so the rebind path is right there. Registered in `lib.rs`; added
  to the IPC contract; `tauriBridge.ts` + a `.notice` style.

**Key decisions / deviations — needs your confirmation**
- **Display-move conflict (`Ctrl+Alt+Win+←/→` vs Windows `Win+←/→`): kept parity + warn**, per the
  issue's recommendation and §2 (identical muscle memory). I did **not** change the Windows default.
  I could not test the actual interaction on Windows hardware, so I shipped a warning (notice #3)
  telling the user to rebind if their setup reacts oddly. **Please confirm keep-vs-rebind once you
  can try it on Windows** — if it genuinely collides, say so and I'll ship a Windows-specific default
  or a stronger nudge.

**Verification** (macOS dev box)
- `MessageBeep` API cross-checked in the isolated crate (`cargo check --target
  x86_64-pc-windows-msvc` → clean). `windows.rs` + `config.rs` type-check cleanly for the Windows
  target (via the temporary `build.rs` no-op; `build.rs` unchanged in the commit).
- macOS: `cargo test` 68 pass, `cargo clippy --all-targets` clean. Frontend: `tsc` clean,
  `vite build` OK.
- **Not run here (needs Windows):** snap an elevated Terminal → single beep, no crash; confirm the
  notices banner renders; observe the `Ctrl+Alt+Win+←/→` interaction with Windows snapping. Flagged.

## Suggested commit message

```
feat(windows): beep on elevated windows and warn on Ctrl+Alt conflicts

Fill 021's Windows beep stub with MessageBeep, so an elevated window that silently
refuses SetWindowPos (caught by 021's effect check) now beeps instead of failing
quietly. Add get_platform_notices: on Windows it returns the known Ctrl+Alt
conflicts (Intel screen rotation, AltGr, display-move vs Win+Arrow snapping) with
a rebind nudge, rendered as a banner in the shortcut editor; empty on macOS. Keeps
cross-platform default parity (§2). Cross-checked for x86_64-pc-windows-msvc; the
elevated-window + conflict smoke needs a Windows machine.
```
