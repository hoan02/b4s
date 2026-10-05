/** Theme preference: system, light, or dark. */

export type ThemeMode = "system" | "light" | "dark";

const KEY = "b4s-theme";
let systemMedia: MediaQueryList | undefined;
let systemListener: (() => void) | undefined;

function resolvedTheme(mode: ThemeMode): "light" | "dark" {
  if (mode !== "system") return mode;
  return window.matchMedia?.("(prefers-color-scheme: light)").matches
    ? "light"
    : "dark";
}

export function getStoredTheme(): ThemeMode {
  try {
    const v = localStorage.getItem(KEY);
    if (v === "system" || v === "light" || v === "dark") return v;
  } catch {
    /* */
  }
  return "system";
}

export function applyTheme(mode: ThemeMode) {
  document.documentElement.setAttribute("data-theme", resolvedTheme(mode));
  try {
    localStorage.setItem(KEY, mode);
  } catch {
    /* */
  }

  if (systemMedia && systemListener) {
    systemMedia.removeEventListener("change", systemListener);
  }
  systemMedia = undefined;
  systemListener = undefined;

  if (mode === "system" && window.matchMedia) {
    systemMedia = window.matchMedia("(prefers-color-scheme: light)");
    systemListener = () => {
      document.documentElement.setAttribute("data-theme", resolvedTheme("system"));
    };
    systemMedia.addEventListener("change", systemListener);
  }
}
