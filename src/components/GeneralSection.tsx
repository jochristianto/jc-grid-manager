// General settings (idea.md §6): launch-at-login (issue 023) and the sizing tunables (012/015).
// All backend access goes through ../lib/tauriBridge.

import { useEffect, useState } from "react";
import {
  errorMessage,
  getAutostart,
  getConfig,
  setAutostart,
  setTunable,
  type Tunables,
  type TunableKey,
} from "../lib/tauriBridge";

interface TunableSpec {
  key: TunableKey;
  label: string;
  hint: string;
  min: number;
  max: number;
  step: number;
}

// Ranges stay inside the backend's accepted bounds ((0,1] for factors, (0,0.5] for the step),
// so the slider can never produce a value set_tunable rejects.
const TUNABLES: TunableSpec[] = [
  {
    key: "almost_maximize_factor",
    label: "Almost Maximize size",
    hint: "Fraction of the screen Almost Maximize fills, centered.",
    min: 0.5,
    max: 1,
    step: 0.05,
  },
  {
    key: "resize_step",
    label: "Resize step",
    hint: "How much Smaller / Larger change the window per press.",
    min: 0.05,
    max: 0.5,
    step: 0.05,
  },
  {
    key: "min_size",
    label: "Minimum size",
    hint: "Smaller won't shrink a window below this.",
    min: 0.05,
    max: 1,
    step: 0.05,
  },
];

export function GeneralSection() {
  const [autostart, setAutostartState] = useState<boolean | null>(null);
  const [tunables, setTunables] = useState<Tunables | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    Promise.all([getConfig(), getAutostart()])
      .then(([config, auto]) => {
        if (!active) return;
        setTunables(config.tunables);
        setAutostartState(auto);
      })
      .catch((err) => {
        if (active) setError(errorMessage(err));
      });
    return () => {
      active = false;
    };
  }, []);

  async function toggleAutostart(next: boolean) {
    setError(null);
    const previous = autostart;
    setAutostartState(next); // optimistic
    try {
      await setAutostart(next);
    } catch (err) {
      setAutostartState(previous); // roll back on failure
      setError(errorMessage(err));
    }
  }

  async function commitTunable(key: TunableKey, value: number) {
    setError(null);
    try {
      await setTunable(key, value);
    } catch (err) {
      setError(errorMessage(err));
      // Resync so the UI drops a value the backend rejected.
      try {
        const config = await getConfig();
        setTunables(config.tunables);
      } catch {
        /* keep the last known values */
      }
    }
  }

  if (autostart === null || tunables === null) {
    return <p className="muted">Loading…</p>;
  }

  return (
    <section className="section">
      <h2>General</h2>

      <label className="toggle">
        <input
          type="checkbox"
          checked={autostart}
          onChange={(e) => toggleAutostart(e.currentTarget.checked)}
        />
        <span>Launch at login</span>
      </label>

      <h3>Sizing</h3>
      {TUNABLES.map((spec) => (
        <div className="field" key={spec.key}>
          <div className="field-head">
            <label htmlFor={spec.key}>{spec.label}</label>
            <span className="value">{Math.round(tunables[spec.key] * 100)}%</span>
          </div>
          <input
            id={spec.key}
            type="range"
            min={spec.min}
            max={spec.max}
            step={spec.step}
            value={tunables[spec.key]}
            // Update the readout while dragging, commit to the backend on release.
            onChange={(e) =>
              setTunables((t) => (t ? { ...t, [spec.key]: e.currentTarget.valueAsNumber } : t))
            }
            onPointerUp={(e) => commitTunable(spec.key, e.currentTarget.valueAsNumber)}
            onKeyUp={(e) => commitTunable(spec.key, e.currentTarget.valueAsNumber)}
          />
          <p className="hint">{spec.hint}</p>
        </div>
      ))}

      {error && <p className="error">{error}</p>}
    </section>
  );
}
