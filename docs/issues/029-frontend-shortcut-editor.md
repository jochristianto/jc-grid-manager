# 029 — [Frontend] Shortcut editor / recorder

| | |
|---|---|
| **Issue ID** | 029 |
| **Layer** | frontend (React + TypeScript) — own files |
| **Depends on** | 020 (validation + set/reset commands), 028 (shell it lives in) |
| **Blocks** | — |
| **Default shortcut** | n/a |
| **Source** | `docs/idea.md` §4 ("Every shortcut is rebindable"), §5.4 |
| **Status** | ☑ Done |

## Summary

Build the **Shortcuts** section: list every action with its current binding, let the user **record** a new combo, and show **warnings** when a combo is already used by another action or is reserved/refused by the OS (§4/§5.4). This is the UI over the 020 rebinding backend.

## Context (self-contained)

- Lives inside the Settings shell (028), under the Shortcuts tab, using `src/lib/tauriBridge.ts`.
- **IPC contract consumed** (from 020 — confirm names against its Implementation summary):
  - `get_bindings() -> { action, label, bind, is_default }[]`
  - `set_binding(action, bind) -> Result<void, { code, message }>` (validates: duplicate / OS-refused)
  - `reset_binding(action)`, `reset_all_bindings()`
  - event `bindings-changed` (re-fetch on receipt)
- Recording a combo: capture modifiers + key in the browser, translate to the backend `Bind` shape, and send to `set_binding`. The base modifier is ⌃⌥ (mac) / Ctrl+Alt (win); show combos in the platform's glyphs.
- Some actions have **no** default bind (Center Half, Move to Edge, Fourths, Sixths) — the user can add one here.

## What to build

1. A table/list of actions grouped like §4, each row: label, current combo (or "None"), Record button, Reset button.
2. A **recorder** control that captures a key combo and calls `set_binding`; on a validation error, show the returned message inline (duplicate → name the conflicting action; OS-refused → explain + suggest another).
3. **Reset** per-row and **Reset all**.
4. Subscribe to `bindings-changed` and re-fetch so the list stays in sync (e.g. if a rebind cascades).
5. Windows conflict hints (from 027) surfaced near the relevant rows if the backend flags them.

## Open decisions (recommendation)

- **Key capture UX** — inline "press keys now…" vs a modal. _Recommendation:_ inline row-level capture with a clear recording state + Escape to cancel; simplest and fastest.
- **Showing platform glyphs** — detect OS and render ⌃⌥ vs Ctrl+Alt. _Recommendation:_ render the running platform's glyphs (config is per-machine anyway).

## Acceptance criteria

- [ ] Every action appears with its current binding (or "None"); grouping mirrors §4.
- [ ] Recording a valid new combo updates the binding live (backend re-registers via 020) and persists.
- [ ] A duplicate combo is rejected with a message naming the conflicting action; an OS-refused combo is rejected with an explanation — neither is silently applied.
- [ ] Per-row Reset and Reset-all restore defaults.
- [ ] The list stays in sync via `bindings-changed`.

## Testing

- Manual: rebind Left Half, confirm the new key works globally and survives restart; try to set a combo already used → inline conflict message; reset one row and reset all.

## LLM prompt

```text
You are implementing issue 029 for JC Grid Manager (FRONTEND, React + TypeScript). Read docs/idea.md §4/§5.4
and docs/issues/029-frontend-shortcut-editor.md. Issues 020 (rebinding backend + validation) and 028
(Settings shell + tauriBridge) are done — confirm the exact IPC names against issue 020's Implementation
summary.

Task: Build the Shortcuts section inside the Settings shell: list every action grouped like §4 with its
current combo, a key-combo recorder that calls set_binding, inline validation errors (duplicate → name the
conflicting action; OS-refused → explain), per-row Reset and Reset-all, and a bindings-changed subscription
to stay in sync. Render the running platform's modifier glyphs. Surface Windows conflict hints (027) if the
backend flags them.

Definition of done: acceptance criteria pass; rebinding works live and validates; UI stays in sync.

Constraints: frontend files only; all backend access via tauriBridge.ts. Do not implement validation logic
in the UI — trust the backend's result and render it. Scope is the shortcut editor.

Bookkeeping (required): record START now; add FINISH + DURATION; write the Implementation summary;
do NOT git commit; refine the Suggested commit message; the user commits.
```

## Implementation log

- **Started:** 2026-07-08 21:51 WIB
- **Finished:** 2026-07-08 21:57 WIB
- **Duration:** ~6m hands-on (excludes reading/design)

## Implementation summary

Built the **Shortcuts** section inside the 028 shell: every action grouped like §4 with its current
combo, a per-row key recorder, inline backend-validation errors, and per-row + global reset — all
over 020's rebinding contract via `tauriBridge`.

**What changed** (frontend only)
- **`src/lib/format.ts`** — `formatBind(bind)` renders the running platform's glyphs (`⌃⌥⇧⌘` on
  macOS, `Ctrl+Alt+…` on Windows) with arrow/return/delete symbols and letter/digit keys, plus
  exported `IS_MAC` / `BASE_MODIFIER_LABEL`. Mirrors the Rust `Bind::hint` (024).
- **`src/components/ShortcutEditor.tsx`** — fetches `getBindings`, groups the 37 actions like §4,
  and renders a row (label · combo/"None" · Record · Reset) each. **Recording**: click Record →
  the next non-modifier `keydown` becomes the combo (`base_modifier = ctrl&&alt`, `shift`, `super =
  meta`, `code = e.code`, which is already the W3C code the backend wants); Esc cancels; auto-repeat
  ignored. It calls `set_binding` and **renders whatever the backend returns** — a rejection message
  goes inline on the row (duplicate names the conflicting action; OS-refused explains). **Reset**
  per row (disabled when already default) and **Reset all**. Subscribes to `bindings-changed` and
  re-fetches so the list stays in sync.
- **`SettingsWindow.tsx`** — renders `<ShortcutEditor/>` in the Shortcuts tab (placeholder gone) and
  defaults to that tab now that it's real. **`App.css`** — styles for the editor rows.

**Key decisions / deviations**
- **No validation logic in the UI** (per the constraint): the recorder sends the raw combo and
  trusts the backend's verdict. This also means **Windows conflict hints (027)** need no special
  code here — when 027 makes `set_binding` reject a reserved combo, that message renders inline like
  any other, near the row.
- **Inline row-level recording** with an explicit "Press keys…" state + Esc-to-cancel (the
  recommended UX), not a modal.
- `e.code` maps 1:1 to the backend `code` (both are W3C UI Events codes), so no key translation
  table is needed on the write path.

**Verification**
- `pnpm exec tsc --noEmit` → clean (strict). `pnpm run build` → built, 40 modules.
- **Rendered headlessly** (vite dev + gstack browse): the app still mounts with the Shortcuts tab as
  default — **no console errors, no white-screen**; tabs switch. The list itself shows "Loading…" in
  a plain browser because `get_bindings` (invoke) has no Tauri backend there — expected; it populates
  under `tauri dev`.
- **Not run here:** the live smoke (record a new combo for Left Half → fires globally + survives
  restart; set a duplicate → inline conflict naming the other action; Reset one + Reset all). Needs
  the running app; flagged for the user.

## Suggested commit message

```
feat(ui): add shortcut editor with key recording and conflict warnings

Build the Shortcuts section: every action grouped like §4 with its current combo,
a per-row recorder that captures a keydown and calls set_binding, and per-row +
global reset. Validation stays in the backend — a rejected combo (duplicate /
OS-refused) renders inline on the row. Render the platform's modifier glyphs and
re-fetch on bindings-changed to stay in sync.
```
