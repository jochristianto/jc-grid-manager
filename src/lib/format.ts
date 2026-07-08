// Render a backend `Bind` as a human chord using the running platform's glyphs (idea.md §5.4).
// Config is per-machine, so we show the local platform's symbols: ⌃⌥⇧⌘ on macOS, Ctrl+Alt+… on
// Windows. Mirrors the Rust-side `Bind::hint` used by the tray (issue 024).

import type { Bind } from "./tauriBridge";

export const IS_MAC =
  typeof navigator !== "undefined" &&
  (/Mac/i.test(navigator.platform) || /Mac OS X/i.test(navigator.userAgent));

/** The base-modifier label for the running platform (idea.md §4). */
export const BASE_MODIFIER_LABEL = IS_MAC ? "⌃⌥" : "Ctrl+Alt";

const KEY_GLYPHS: Record<string, string> = {
  ArrowLeft: "←",
  ArrowRight: "→",
  ArrowUp: "↑",
  ArrowDown: "↓",
  Enter: "↩",
  Backspace: "⌫",
  Delete: "⌦",
  Escape: "⎋",
  Tab: "⇥",
  Space: "␣",
  Minus: "−",
  Equal: "=",
};

/** A readable label for a W3C key code: glyph for known keys, else the letter/digit or raw code. */
export function keyLabel(code: string): string {
  const glyph = KEY_GLYPHS[code];
  if (glyph) return glyph;
  const letter = /^Key([A-Z])$/.exec(code);
  if (letter) return letter[1];
  const digit = /^Digit([0-9])$/.exec(code);
  if (digit) return digit[1];
  return code;
}

/** Format a chord for display, e.g. `⌃⌥←` (macOS) or `Ctrl+Alt+←` (Windows). */
export function formatBind(bind: Bind): string {
  if (IS_MAC) {
    let out = "";
    if (bind.base_modifier) out += "⌃⌥";
    if (bind.shift) out += "⇧";
    if (bind.super) out += "⌘";
    return out + keyLabel(bind.code);
  }
  const parts: string[] = [];
  if (bind.base_modifier) parts.push("Ctrl", "Alt");
  if (bind.shift) parts.push("Shift");
  if (bind.super) parts.push("Win");
  parts.push(keyLabel(bind.code));
  return parts.join("+");
}
