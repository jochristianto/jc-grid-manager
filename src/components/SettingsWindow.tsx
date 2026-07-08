// The Settings window shell (idea.md §5.2/§6): tabbed navigation between Shortcuts, General, and
// About. State-based routing — three sections don't warrant a router dependency. The Shortcuts
// editor arrives in issue 029; onboarding (030) is reachable from here later.

import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { GeneralSection } from "./GeneralSection";
import { ShortcutEditor } from "./ShortcutEditor";

type Tab = "shortcuts" | "general" | "about";

const TABS: { id: Tab; label: string }[] = [
  { id: "shortcuts", label: "Shortcuts" },
  { id: "general", label: "General" },
  { id: "about", label: "About" },
];

const PROJECT_URL = "https://github.com/jochristianto/jc-grid-manager";

export function SettingsWindow() {
  const [tab, setTab] = useState<Tab>("shortcuts");

  return (
    <div className="settings">
      <nav className="tabs">
        <div className="brand">JC Grid Manager</div>
        {TABS.map((t) => (
          <button
            key={t.id}
            className={t.id === tab ? "tab active" : "tab"}
            onClick={() => setTab(t.id)}
          >
            {t.label}
          </button>
        ))}
      </nav>
      <main className="content">
        {tab === "shortcuts" && <ShortcutEditor />}
        {tab === "general" && <GeneralSection />}
        {tab === "about" && <AboutSection />}
      </main>
    </div>
  );
}

function AboutSection() {
  const [version, setVersion] = useState<string>("");

  useEffect(() => {
    let active = true;
    getVersion()
      .then((v) => active && setVersion(v))
      .catch(() => active && setVersion("unknown"));
    return () => {
      active = false;
    };
  }, []);

  return (
    <section className="section">
      <h2>About</h2>
      <p className="app-name">JC Grid Manager</p>
      <p className="muted">Version {version || "…"}</p>
      <p className="note">
        An unsigned personal build — macOS may warn on first launch (right-click the app → Open),
        and it needs Accessibility permission to move windows.
      </p>
      <button className="link" onClick={() => openUrl(PROJECT_URL).catch(() => {})}>
        View project on GitHub
      </button>
    </section>
  );
}
