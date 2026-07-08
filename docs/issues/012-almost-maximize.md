# 012 — Almost Maximize

| | |
|---|---|
| **Issue ID** | 012 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 004 (soft: 011) |
| **Blocks** | surfaced in tray menu (024) |
| **Default shortcut** | none (menu-only; user may bind) |
| **Source** | `docs/idea.md` §4 (Sizing → Almost Maximize) |
| **Status** | ☐ Not started |

## Summary

Add **Almost Maximize**: fill **~90%** of the work area, **centered**. The percentage is **configurable** (default 90%).

## Geometry (fraction of work area)

- Almost Maximize (default 90%) → `(0.05, 0.05, 0.9, 0.9)`
- General form for factor `f` (0<f≤1): `((1-f)/2, (1-f)/2, f, f)`

## What to build

1. Add the `AlmostMaximize` geometry parameterized by a factor (default `0.9`).
2. Read the factor from config once 020 exists; until then use the default constant (keep it in one named place so 020 can wire it).
3. No default bind (menu-only) — exercised via tray menu (024) or user bind.
4. Unit-test the general form for a couple of factors.

## Open decisions (recommendation)

- **Default factor** — 90% per §4. _Recommendation:_ keep 90%; expose as a config key `almost_maximize_factor` for 020.

## Acceptance criteria

- [ ] Almost Maximize centers a 90%-of-work-area window on its display.
- [ ] The factor is a single named constant/config key (not a magic number scattered around).
- [ ] Unit test asserts the rect for factor 0.9 (and one other factor).

## Testing

- Unit: `((1-f)/2,(1-f)/2,f,f)` for f=0.9 and f=0.8.
- Manual: trigger via menu/bind → centered 90% window.

## LLM prompt

```text
You are implementing issue 012 for JC Grid Manager. Read docs/idea.md §4 (Almost Maximize) and
docs/issues/012-almost-maximize.md. Issue 004 is done.

Task: Implement Almost Maximize as a centered window filling a configurable fraction of the work area
(default 0.9 → rect (0.05,0.05,0.9,0.9)). Parameterize by a factor with the general form
((1-f)/2,(1-f)/2,f,f). Keep the default in one named place so issue 020 (config) can wire it. No default
shortcut. Unit-test two factors.

Definition of done: acceptance criteria pass; centered 90% placement works; tests green.

Constraints: scope is only Almost Maximize; don't build the config system (020), just leave a clean hook.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary;
do NOT git commit; refine the Suggested commit message; the user commits.
```

## Implementation log (fill this in)

- **Started:** _<!-- -->_
- **Finished:** _<!-- -->_
- **Duration:** _<!-- -->_

## Implementation summary (fill this in)

_<!-- ... -->_

## Suggested commit message

```
feat(core): add Almost Maximize action (configurable, default 90%)

Centered window filling a configurable fraction of the work area
((1-f)/2,(1-f)/2,f,f); default factor 0.9.
```
