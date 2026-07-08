// The Shortcuts editor (idea.md §4 "every shortcut is rebindable" / §5.4). Lists every action
// grouped like §4, records a new combo per row, and surfaces the backend's validation verbatim
// (duplicate / OS-refused) — the UI never decides validity itself. All backend access is via
// ../lib/tauriBridge (issue 020's rebinding contract).

import { useCallback, useEffect, useState } from "react";
import {
  errorMessage,
  getBindings,
  onBindingsChanged,
  resetAllBindings,
  resetBinding,
  setBinding,
  type Bind,
  type BindingInfo,
} from "../lib/tauriBridge";
import { BASE_MODIFIER_LABEL, formatBind } from "../lib/format";

// Grouping mirrors §4 (and the tray menu). Action ids are the stable Rust `Action::id` values.
const GROUPS: { title: string; actions: string[] }[] = [
  { title: "Halves", actions: ["left-half", "right-half", "top-half", "bottom-half", "center-half"] },
  { title: "Corners", actions: ["top-left", "top-right", "bottom-left", "bottom-right"] },
  {
    title: "Thirds",
    actions: ["first-third", "center-third", "last-third", "first-two-thirds", "last-two-thirds"],
  },
  {
    title: "Sizing",
    actions: ["maximize", "almost-maximize", "maximize-height", "smaller", "larger", "center", "restore"],
  },
  { title: "Displays", actions: ["next-display", "previous-display"] },
  { title: "Move to Edge", actions: ["move-left", "move-right", "move-up", "move-down"] },
  { title: "Fourths", actions: ["first-fourth", "second-fourth", "third-fourth", "last-fourth"] },
  {
    title: "Sixths",
    actions: [
      "sixth-top-left",
      "sixth-top-center",
      "sixth-top-right",
      "sixth-bottom-left",
      "sixth-bottom-center",
      "sixth-bottom-right",
    ],
  },
];

const MODIFIER_KEYS = new Set(["Control", "Alt", "Shift", "Meta"]);

export function ShortcutEditor() {
  const [byId, setById] = useState<Map<string, BindingInfo> | null>(null);
  const [recording, setRecording] = useState<string | null>(null);
  const [rowErrors, setRowErrors] = useState<Record<string, string>>({});
  const [globalError, setGlobalError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const list = await getBindings();
      setById(new Map(list.map((b) => [b.action, b])));
    } catch (err) {
      setGlobalError(errorMessage(err));
    }
  }, []);

  // Initial load + stay in sync when the backend re-registers anything (issue 020).
  useEffect(() => {
    void refresh();
    const unlisten = onBindingsChanged(() => void refresh());
    return () => {
      void unlisten.then((un) => un());
    };
  }, [refresh]);

  const clearRowError = useCallback((action: string) => {
    setRowErrors((m) => {
      if (!(action in m)) return m;
      const next = { ...m };
      delete next[action];
      return next;
    });
  }, []);

  // While recording a row, capture the next non-modifier keydown as the new combo.
  useEffect(() => {
    if (!recording) return;
    const action = recording;
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.repeat) return;
      if (e.key === "Escape") {
        setRecording(null);
        return;
      }
      if (MODIFIER_KEYS.has(e.key)) return; // wait for the actual key
      const bind: Bind = {
        base_modifier: e.ctrlKey && e.altKey,
        shift: e.shiftKey,
        super: e.metaKey,
        code: e.code,
      };
      setRecording(null);
      clearRowError(action);
      // Trust the backend's verdict: it validates duplicates + OS-refusal and re-registers live.
      setBinding(action, bind)
        .then(() => refresh())
        .catch((err) => setRowErrors((m) => ({ ...m, [action]: errorMessage(err) })));
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [recording, refresh, clearRowError]);

  async function resetOne(action: string) {
    clearRowError(action);
    try {
      await resetBinding(action);
      await refresh();
    } catch (err) {
      setRowErrors((m) => ({ ...m, [action]: errorMessage(err) }));
    }
  }

  async function resetAll() {
    setGlobalError(null);
    try {
      await resetAllBindings();
      await refresh();
    } catch (err) {
      setGlobalError(errorMessage(err));
    }
  }

  if (!byId) {
    return <p className="muted">Loading…</p>;
  }

  return (
    <section className="section shortcuts">
      <div className="shortcuts-head">
        <h2>Shortcuts</h2>
        <button className="link" onClick={() => void resetAll()}>
          Reset all
        </button>
      </div>
      <p className="hint">
        Click <b>Record</b>, then press a combo — the base modifier is {BASE_MODIFIER_LABEL}; add
        Shift or the Command/Win key as you like. Press Esc to cancel.
      </p>
      {globalError && <p className="error">{globalError}</p>}

      {GROUPS.map((group) => (
        <div className="shortcut-group" key={group.title}>
          <h3>{group.title}</h3>
          {group.actions.map((action) => {
            const info = byId.get(action);
            if (!info) return null;
            const isRecording = recording === action;
            return (
              <div className="shortcut-row" key={action}>
                <span className="s-label">{info.label}</span>
                <span className={info.bind ? "s-combo" : "s-combo none"}>
                  {isRecording ? "Press keys…" : info.bind ? formatBind(info.bind) : "None"}
                </span>
                <button
                  className={isRecording ? "s-record recording" : "s-record"}
                  onClick={() => setRecording(isRecording ? null : action)}
                >
                  {isRecording ? "Cancel" : "Record"}
                </button>
                <button className="s-reset" disabled={info.is_default} onClick={() => void resetOne(action)}>
                  Reset
                </button>
                {rowErrors[action] && <p className="s-error error">{rowErrors[action]}</p>}
              </div>
            );
          })}
        </div>
      ))}
    </section>
  );
}
