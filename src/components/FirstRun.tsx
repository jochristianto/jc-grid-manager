// macOS Accessibility onboarding (idea.md §8/§5.6). Handles the two untrusted states the backend
// reports — "never_asked" (prompt) and "denied" (settings deep-link + stale-grant reset guidance).
// Rendered by App only when the state is not "trusted" and only on macOS; the app polls / rechecks
// on focus, so this screen closes itself once the user grants permission.

import { openAccessibilitySettings, promptAccessibility } from "../lib/tauriBridge";

const BUNDLE_ID = "com.jochristianto.jcgridmanager";
const RESET_CMD = `tccutil reset Accessibility ${BUNDLE_ID}`;

export function FirstRun({ state }: { state: "never_asked" | "denied" }) {
  return (
    <div className="firstrun">
      <div className="firstrun-card">
        <h1>Enable window management</h1>
        <p className="firstrun-lead">
          JC Grid Manager moves and resizes your windows, so macOS needs to grant it{" "}
          <b>Accessibility</b> permission. Nothing works until this is on.
        </p>

        {state === "never_asked" ? (
          <>
            <button className="firstrun-cta" onClick={() => void promptAccessibility()}>
              Grant Accessibility Access
            </button>
            <p className="hint">A system dialog will ask you to allow JC Grid Manager.</p>
          </>
        ) : (
          <>
            <button className="firstrun-cta" onClick={() => void openAccessibilitySettings()}>
              Open Accessibility Settings
            </button>
            <p className="hint">
              Turn on <b>JC Grid Manager</b> under Privacy &amp; Security → Accessibility.
            </p>

            <div className="firstrun-stale">
              <h3>Reinstalled or updated recently?</h3>
              <p>
                Because this is an unsigned build, macOS can keep a <b>stale</b> permission entry
                that looks enabled but isn't. Remove JC Grid Manager from the Accessibility list and
                add it back — or reset it from a terminal and relaunch:
              </p>
              <div className="firstrun-cmd">
                <code>{RESET_CMD}</code>
                <button
                  className="link"
                  onClick={() => void navigator.clipboard?.writeText(RESET_CMD)}
                >
                  Copy
                </button>
              </div>
            </div>
          </>
        )}

        <p className="firstrun-foot">This screen closes itself once permission is granted.</p>
      </div>
    </div>
  );
}
