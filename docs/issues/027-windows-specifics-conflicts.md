# 027 — Windows specifics: elevated-window beep + Ctrl+Alt conflicts

| | |
|---|---|
| **Issue ID** | 027 |
| **Layer** | windows (per-OS I/O shim) + config |
| **Depends on** | 025, 020, 021 |
| **Blocks** | — |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §4, §5.4 (Windows conflicts), §8, §11 |
| **Status** | ☐ Not started |

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

## Implementation log (fill this in)

- **Started:** _<!-- -->_
- **Finished:** _<!-- -->_
- **Duration:** _<!-- -->_

## Implementation summary (fill this in)

_<!-- ... -->_

## Suggested commit message

```
feat(windows): beep on elevated windows and warn on Ctrl+Alt conflicts

Detect unmovable/elevated windows and route them through the soft beep
(MessageBeep); surface actionable guidance for Intel screen-rotation and AltGr
conflicts with a rebind path; document the display-move shortcut conflict.
```
