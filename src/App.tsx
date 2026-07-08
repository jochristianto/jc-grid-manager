import { useEffect, useState } from "react";
import "./App.css";
import { SettingsWindow } from "./components/SettingsWindow";
import { FirstRun } from "./components/FirstRun";
import { getAccessibilityState, type AccessibilityState } from "./lib/tauriBridge";
import { IS_MAC } from "./lib/format";

type Gate = AccessibilityState | "loading";

// Gate the app on the macOS Accessibility permission (issue 030). On Windows/Linux there is no
// such permission, so we skip straight to the Settings window and this never shows.
export default function App() {
  const [gate, setGate] = useState<Gate>(IS_MAC ? "loading" : "trusted");

  useEffect(() => {
    if (!IS_MAC) return;
    let active = true;
    let timer: number | undefined;

    const check = async () => {
      try {
        const state = await getAccessibilityState();
        if (!active) return;
        setGate(state);
        // Stop polling once granted; a later regression is re-caught on window focus.
        if (state === "trusted" && timer !== undefined) {
          window.clearInterval(timer);
          timer = undefined;
        }
      } catch {
        // If the backend can't be reached, don't trap the user on the gate.
        if (active) setGate("trusted");
      }
    };

    void check();
    const onFocus = () => void check();
    window.addEventListener("focus", onFocus);
    // Slow poll so granting permission in System Settings advances us without a relaunch.
    timer = window.setInterval(() => void check(), 2000);

    return () => {
      active = false;
      window.removeEventListener("focus", onFocus);
      if (timer !== undefined) window.clearInterval(timer);
    };
  }, []);

  if (gate === "loading") return null;
  if (gate === "trusted") return <SettingsWindow />;
  return <FirstRun state={gate} />;
}
