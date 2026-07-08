# 004 — Action registry + shortcut dispatcher + default binds

| | |
|---|---|
| **Issue ID** | 004 |
| **Layer** | core (backend, Rust) |
| **Depends on** | 001 |
| **Blocks** | 005–019 (every action slice), 020 (config), 024 (tray menu) |
| **Default shortcut** | n/a (defines the whole default scheme) |
| **Source** | `docs/idea.md` §4, §5.4, §7 |
| **Status** | ☑ Done |

## Summary

Replace the four hardcoded shortcuts in `lib.rs` with a **registry-driven dispatcher**. Define a single `Action` enum naming **every** command in §4 (halves, thirds, quarters, sixths, fourths, sizing, move-to-edge, display-move, restore, etc.), a table of **default bindings** in a platform-neutral form that resolves to Control+Option (macOS) / Ctrl+Alt (Windows), and a dispatcher that turns a fired global shortcut into an `Action` call. This is the spine every later action slice plugs into: once an action is named and bound here, implementing it elsewhere makes the key "just work."

## Context (self-contained)

- Current `lib.rs` registers `left/right/up/down` and matches them to `platform::Half`. That pattern does not scale to ~35 actions and is not rebindable.
- The base modifier is Control+Option (macOS) / Ctrl+Alt (Windows). Display-move adds Command/Win. Maximize Height adds Shift. `tauri-plugin-global-shortcut` is already a dependency.
- Full default scheme (macOS form) from §4:
  - Halves: Left `⌃⌥←`, Right `⌃⌥→`, Top `⌃⌥↑`, Bottom `⌃⌥↓`; Center Half — none.
  - Corners: TL `⌃⌥U`, TR `⌃⌥I`, BL `⌃⌥J`, BR `⌃⌥K`.
  - Thirds: First `⌃⌥D`, Center `⌃⌥F`, Last `⌃⌥G`, First-Two `⌃⌥E`, Last-Two `⌃⌥T`.
  - Sizing: Maximize `⌃⌥↩`, Almost Maximize — none, Maximize Height `⌃⌥⇧↑`, Smaller `⌃⌥-`, Larger `⌃⌥=`, Center `⌃⌥C`, Restore `⌃⌥⌫`.
  - Displays: Next `⌃⌥⌘→`, Previous `⌃⌥⌘←`.
  - Move to Edge / Fourths / Sixths: **no default shortcuts** (menu-only; user may bind later).

## What to build

1. **`Action` enum** in `core/actions.rs` naming every §4 command. Include a `label()` (for menus) and a stable string id (for config keys later).
2. **Default binding table** `default_bindings() -> Map<Action, Bind>` in a platform-neutral shape (base-modifier + key + extra modifiers), plus a resolver that turns a `Bind` into a concrete `Shortcut` for the current OS. Actions with no default map to nothing.
3. **A dispatcher** that, given a fired `Shortcut`, finds the `Action` and executes it. For this slice, wire the **halves** through the real geometry path (from 001/003). Every other action may call a **placeholder** that logs `"<action> not implemented"` (a real soft beep arrives in 021) — this keeps the whole key scheme live and testable immediately.
4. **Register all default-bound shortcuts** at startup (replacing the four-shortcut loop). A bind the OS refuses is **logged, not fatal** (matches current behavior; full validation is 020).

## Open decisions (recommendation)

- **Bind representation** — bespoke struct vs reuse the plugin's `Shortcut` directly. _Recommendation:_ a small platform-neutral `Bind { base_modifier: bool, extra: Modifiers, code: Code }` struct, resolved to `Shortcut` at registration; this is what 020 will serialize.
- **Unimplemented actions** — silent no-op vs log vs beep. _Recommendation:_ log for now; 021 upgrades the "nothing happened" case to a soft beep.

## Acceptance criteria

- [ ] A single `Action` enum enumerates every command listed in §4.
- [ ] A default-binding table exists and resolves correctly to macOS shortcuts (spot-check `⌃⌥←`, `⌃⌥↩`, `⌃⌥⇧↑`, `⌃⌥⌘→`).
- [ ] `lib.rs` no longer hardcodes four shortcuts; it registers from the table and dispatches via the registry.
- [ ] The four halves still work; pressing an unimplemented action's key logs a clear "not implemented" line.
- [ ] Unit test: every `Action` has a stable id and (where applicable) resolves to the expected macOS shortcut.

## Testing

- Unit: default-binding resolution for a representative set of actions; round-trip `Action` ↔ string id.
- Manual: press halves (work), press e.g. `⌃⌥D` (logs "First Third not implemented").

## LLM prompt

```text
You are implementing issue 004 for JC Grid Manager. Read docs/idea.md §4/§5.4/§7 and
docs/issues/004-action-registry-and-dispatcher.md. Issue 001 is done (shared core + Platform trait);
003 (display selection) may or may not be done — don't depend on it.

Task: Introduce an `Action` enum covering every command in §4, a platform-neutral default-binding
table that resolves to Control+Option (macOS)/Ctrl+Alt (Windows) with Command→Win and the Shift/Command
extras noted, and a registry-driven dispatcher. Replace the four hardcoded shortcuts in lib.rs with
registration from the table. Wire the halves through the real geometry path; every other action calls a
placeholder that logs "<action> not implemented". A bind the OS refuses is logged, not fatal.

Definition of done: acceptance criteria pass; halves still work; unimplemented keys log clearly; unit
tests for binding resolution and action ids are green.

Constraints: do NOT implement the other actions' geometry (separate issues) and do NOT build config
persistence (issue 020) or conflict validation. Keep the Bind struct serialization-friendly for 020.

Bookkeeping (required): record START in the Implementation log now; add FINISH + DURATION when done;
write the Implementation summary; do NOT git commit; refine the Suggested commit message; the user commits.
```

## Implementation log (fill this in)

- **Started:** 2026-07-08 17:12 WIB
- **Finished:** 2026-07-08 17:24 WIB
- **Duration:** ~12m

## Implementation summary

Replaced the four hardcoded shortcuts with a registry-driven scheme.

**What changed**
- `core/actions.rs`: added the `Action` enum — all 37 §4 commands — with a stable kebab-case
  `id()` (config keys), a menu `label()`, `as_half()` (maps the four directional halves to
  `Half`), an `ALL` array (iteration for registration + the tray menu), and `from_id()`.
  Pure/no-window tests cover id uniqueness, `id ↔ from_id` round-trip, and the half mapping.
- `shortcuts.rs` (new): `Bind { base_modifier, extra: Modifiers, code: Code }` +
  `to_shortcut()`; `default_bind()` holds the §4 default scheme; `default_registry()` resolves
  it to `Vec<(Shortcut, Action)>`. Tests spot-check ⌃⌥←, ⌃⌥↩, ⌃⌥⇧↑, ⌃⌥⌘→, the menu-only
  actions (no default), and that no two actions resolve to the same chord.
- `lib.rs`: builds the registry, registers every default bind at startup (a bind the OS
  refuses is logged, not fatal), and dispatches a fired shortcut → `Action` → `dispatch()`:
  the four halves run the real geometry path (`platform::snap`); every other action logs
  `"<label> not implemented"`. Removed the old `ctrl_alt` helper and the four `.clone()`s
  (which also clears the pre-existing `clone_on_copy` clippy warning).

**Key decisions**
- `Bind` lives in `shortcuts.rs`, not `core`, because it depends on the plugin's `Modifiers`
  / `Code`; this keeps `core` pure and framework-free. `Action` (pure identity) stays in core.
- ⌘/Win resolves to `Modifiers::SUPER` — verified against `global-hotkey` 0.8: `HotKey::new`
  normalizes `Meta → Super`, and the macOS backend maps `Super` to the Command flag.
- `id()`/`from_id()` are `#[allow(dead_code)]` for now (only tests use them); issue 020 wires
  them into config persistence.

**Verification:** `cargo test` → 14 pass (9 new). `cargo clippy` → no new warnings (and one
fewer than before). Runtime check (press ⌃⌥ arrows → snap; press ⌃⌥D → logs "First Third not
implemented") needs the running app + Accessibility grant — a user-side smoke test.

**Follow-ups:** 005 = cycling state machine; 003 = per-window `work_area`; 007–019 fill each
action's geometry; 020 wires `id`/`from_id` + rebinding; 024 uses `label()`/`ALL` for the tray.

## Suggested commit message

```
feat(core): add action registry, default binds, and shortcut dispatcher

Name every idea.md §4 command in an Action enum (core/actions.rs) with stable
ids + menu labels, add a platform-neutral default-binding table in a new
shortcuts.rs (base ⌃⌥ / Ctrl+Alt, ⌘/Win = Super), and dispatch fired shortcuts
through a resolved registry. Replaces the four hardcoded shortcuts in lib.rs:
the halves run the real geometry path, every other action logs a placeholder
until its slice lands. cargo test: 14 passing.
```
