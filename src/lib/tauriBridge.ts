// Typed bridge to the Rust backend (idea.md §5.2/§6). Every `invoke` and event subscription the
// UI needs lives here — components never call `invoke` directly. Command + event names and payload
// shapes mirror the IPC contract shipped by issue 020 (config/rebinding), 022 (ignore list), and
// 023 (autostart); see src-tauri/src/config.rs.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** A platform-neutral chord, serialized exactly as the Rust `Bind` (issue 020). */
export interface Bind {
  base_modifier: boolean;
  shift: boolean;
  super: boolean;
  /** W3C UI Events key code, e.g. `"ArrowLeft"`, `"KeyH"`. */
  code: string;
}

/** Sizing tunables (issues 012/015) — fractions of the display work area. */
export interface Tunables {
  almost_maximize_factor: number;
  resize_step: number;
  min_size: number;
}

/** The persisted, per-machine config (issue 020). */
export interface Config {
  version: number;
  bindings: Record<string, Bind>;
  tunables: Tunables;
  ignore_apps: string[];
  autostart: boolean;
}

/** One row of the shortcut table, for the editor (issues 020/029). */
export interface BindingInfo {
  action: string;
  label: string;
  bind: Bind | null;
  is_default: boolean;
}

/** Frontmost app + ignore state (issue 022). */
export interface IgnoreStatus {
  id: string | null;
  name: string;
  ignored: boolean;
}

/** Structured command failure — the value an `invoke` promise rejects with (issue 020). */
export interface BindingError {
  code: string;
  message: string;
}

/** A settable sizing tunable. */
export type TunableKey = keyof Tunables;

/** macOS Accessibility permission state for onboarding (issue 030). */
export type AccessibilityState = "trusted" | "never_asked" | "denied";

/** A platform-specific shortcut warning (Windows Ctrl+Alt conflicts, issue 027). */
export interface PlatformNotice {
  title: string;
  body: string;
}

/** Emitted by the backend after any successful config change. */
export const EVENT_BINDINGS_CHANGED = "bindings-changed";

export function getConfig(): Promise<Config> {
  return invoke("get_config");
}

export function getBindings(): Promise<BindingInfo[]> {
  return invoke("get_bindings");
}

export function setBinding(action: string, bind: Bind): Promise<void> {
  return invoke("set_binding", { action, bind });
}

export function resetBinding(action: string): Promise<void> {
  return invoke("reset_binding", { action });
}

export function resetAllBindings(): Promise<void> {
  return invoke("reset_all_bindings");
}

export function setTunable(key: TunableKey, value: number): Promise<void> {
  return invoke("set_tunable", { key, value });
}

export function getAutostart(): Promise<boolean> {
  return invoke("get_autostart");
}

export function setAutostart(enabled: boolean): Promise<void> {
  return invoke("set_autostart", { enabled });
}

export function getFrontmostApp(): Promise<IgnoreStatus> {
  return invoke("get_frontmost_app");
}

export function toggleIgnoreCurrentApp(): Promise<IgnoreStatus> {
  return invoke("toggle_ignore_current_app");
}

/** Subscribe to backend `bindings-changed`; resolves to the unlisten fn. */
export function onBindingsChanged(handler: () => void): Promise<UnlistenFn> {
  return listen(EVENT_BINDINGS_CHANGED, () => handler());
}

/** macOS Accessibility permission state (issue 030). Always `"trusted"` off macOS. */
export function getAccessibilityState(): Promise<AccessibilityState> {
  return invoke("get_accessibility_state");
}

/** Pop the macOS Accessibility system prompt (records that we've asked). */
export function promptAccessibility(): Promise<void> {
  return invoke("prompt_accessibility");
}

/** Open System Settings → Privacy & Security → Accessibility. */
export function openAccessibilitySettings(): Promise<void> {
  return invoke("open_accessibility_settings");
}

/** Platform-specific shortcut warnings (Windows Ctrl+Alt conflicts, issue 027). Empty off Windows. */
export function getPlatformNotices(): Promise<PlatformNotice[]> {
  return invoke("get_platform_notices");
}

/** Narrow an unknown `invoke` rejection to a human message. */
export function errorMessage(err: unknown): string {
  if (err && typeof err === "object" && "message" in err) {
    return String((err as BindingError).message);
  }
  return String(err);
}
