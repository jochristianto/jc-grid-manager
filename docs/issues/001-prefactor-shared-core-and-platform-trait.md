# 001 — Prefactor: extract the shared core + `Platform` trait + test harness

| | |
|---|---|
| **Issue ID** | 001 |
| **Layer** | core (backend, Rust) |
| **Depends on** | None — can start immediately |
| **Blocks** | 003, 004, 005, and every action slice (007–019); the Windows shim (025) |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §5.3, §6, §10 |
| **Status** | ☐ Not started |

## Summary

Redraw the code boundary described in §5.3 **before** adding more features. Today all the snapping logic (the `Half` enum, the fraction math, the work-area math) lives inside `src-tauri/src/platform/macos.rs`. Pull the smart, platform-agnostic part into a shared `core` module and hide the raw OS calls behind a thin `Platform` trait. This is the "make the change easy, then make the easy change" step: after it, every new action is a small addition to a geometry table, and adding Windows is just a second trait implementation.

## Context (self-contained)

Current state (as of writing):
- `src-tauri/src/platform/macos.rs` holds: `Half` (Left/Right/Top/Bottom), `Half::fraction()`, `focused_window()`, `main_work_area()`, `set_frame()`, `ensure_trusted()`, and the `snap()` entry point. It snaps to the **main** display only.
- `src-tauri/src/platform/mod.rs` re-exports `snap` and `Half`, with a non-macOS stub. Its own doc comment already says the shared core + `Platform` trait "get factored out once more of the native surface is proven." That time is now.
- `src-tauri/src/lib.rs` maps four hardcoded shortcuts to `Half` variants and calls `platform::snap`.

Target module layout (from §6):
```
src-tauri/src/
├── core/
│   ├── mod.rs
│   ├── geometry.rs   # fraction-based target rectangles + a Rect type
│   └── (state.rs, actions.rs land in 004/005)
├── platform/
│   ├── mod.rs        # the Platform trait (~6 I/O methods) + cfg-gated selection
│   ├── macos.rs      # implements Platform (cfg macos)
│   └── windows.rs    # added in 025 (cfg windows)
```

## What to build

1. **A `Rect` type + geometry module** in `core/geometry.rs`: a plain rectangle in a top-left, fraction-friendly space, plus the function that turns a `(x, y, w, h)` fraction tuple + a work-area `Rect` into an absolute target `Rect`. Move `Half` and its `fraction()` here (or into `core/actions.rs`; either is fine for this slice as long as it is out of `macos.rs`).
2. **The `Platform` trait** in `platform/mod.rs` with the ~6 dumb I/O methods from §5.3 — no logic, just OS calls:
   - `focused_window() -> Result<Window, Error>`
   - `frame(win) -> Rect`
   - `set_frame(win, Rect) -> Result<(), Error>`
   - `work_area(win) -> Rect` (work area of the display the window is on)
   - `displays() -> Vec<Rect>` (work areas of all displays)
   - `identity(win) -> WindowIdentity` (owning app bundle id / name — used later by 022)
   Define an associated `Window` handle type (an opaque wrapper around `AXUIElementRef` on macOS).
3. **Implement `Platform` for macOS** by moving the existing AX/NSScreen code from `macos.rs` behind the trait. Keep the size→position→size `set_frame` recipe and the primary-screen-height coordinate flip exactly as they are — they are correct and hard-won.
4. **Route the existing four half-snaps** through `core` + the trait so they still work identically. **Preserve today's behavior** (still targets the main display — proper per-window display selection is issue 003). This keeps the prefactor reviewable as a no-behavior-change refactor.
5. **Stand up the test harness**: make `core` unit-testable with no real windows. Add `#[cfg(test)]` tests in `core/geometry.rs` that feed a fake work-area `Rect` + a fraction tuple and assert the resulting absolute `Rect`. `cargo test` must pass.

## Open decisions (recommendation)

- **Error type** — use a simple `String` error (matches current code) or a small `enum`. _Recommendation:_ keep `Result<_, String>` for now; a richer error enum can come with 021 (soft beep) when failure classification starts to matter.
- **Where `Half`/actions live** — `core/actions.rs` vs `core/geometry.rs`. _Recommendation:_ create `core/actions.rs` now (even if tiny) so 004/005 have a home; put the pure rectangle math in `core/geometry.rs`.

## Acceptance criteria

- [ ] `Half`, `fraction()`, and all rectangle/work-area math no longer live in `platform/macos.rs`.
- [ ] `platform/mod.rs` defines a `Platform` trait with the ~6 I/O methods; `macos.rs` implements it; the non-macOS path still compiles (stub/`todo!` is fine until 025).
- [ ] The four half-snaps (`⌃⌥` + arrows) still work on macOS, behaving exactly as before (main display).
- [ ] `cargo test` runs and passes at least 3 geometry unit tests that use no real windows.
- [ ] No clippy regressions introduced (`cargo clippy` is no noisier than before).

## Testing

- Unit: geometry conversions (fraction → absolute) with a synthetic work area, including a work area at a **negative** origin (mirrors the multi-monitor case in [[dev-multi-monitor-setup]]).
- Manual smoke: build, grant Accessibility, press each of the four arrows, confirm the focused window snaps to the correct half of the main display.

## LLM prompt

```text
You are implementing issue 001 for the JC Grid Manager project. Read docs/idea.md
(especially §5.3, §6, §10) and docs/issues/001-prefactor-shared-core-and-platform-trait.md
in full before starting.

Task: Refactor src-tauri so the snapping logic moves out of the macOS shim into a shared,
platform-agnostic `core` module, and the raw OS calls sit behind a `Platform` trait — WITHOUT
changing the behavior of the existing four half-snaps. Preserve the size→position→size set_frame
recipe and the primary-screen-height coordinate flip. Add a `cargo test` harness with pure geometry
unit tests (no real windows).

Definition of done: acceptance criteria in the issue file all pass; `cargo test` is green; the four
half-snaps still work on macOS exactly as before.

Constraints: this is a no-behavior-change refactor. Do NOT add new actions, display selection,
cycling, or Windows code — those are separate issues. Match the existing code style and comment density.

Bookkeeping (required):
- Fill the Implementation log below with your START time (date, time, timezone) now.
- On completion, add the FINISH time and DURATION.
- Write the Implementation summary (what you changed, key decisions, any deviations, follow-ups).
- Do NOT git commit. Refine the Suggested commit message to match what you actually did; the user commits.
- Stay within this issue's scope.
```

## Implementation log (fill this in)

- **Started:** _<!-- YYYY-MM-DD HH:MM TZ -->_
- **Finished:** _<!-- YYYY-MM-DD HH:MM TZ -->_
- **Duration:** _<!-- e.g. 2h15m -->_

## Implementation summary (fill this in)

_<!-- What you built, key decisions, deviations from the plan, follow-ups discovered. -->_

## Suggested commit message

```
refactor(core): extract shared geometry core and Platform trait

Move Half, fraction math, and work-area math out of the macOS shim into a
platform-agnostic `core` module, and hide raw AX/NSScreen calls behind a
`Platform` trait. Behavior of the four half-snaps is unchanged. Add a pure
geometry unit-test harness.
```
