export interface DesktopPreferences {
  version: 1;
  autoReconnect: boolean;
}

const KEY = "b4s.desktop.preferences.v1";

const DEFAULT_PREFERENCES: DesktopPreferences = {
  version: 1,
  autoReconnect: false,
};

export function readDesktopPreferences(): DesktopPreferences {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(KEY) ?? "null");
    if (
      typeof value === "object" && value !== null &&
      "version" in value && value.version === 1 &&
      "autoReconnect" in value && typeof value.autoReconnect === "boolean"
    ) {
      return { version: 1, autoReconnect: value.autoReconnect };
    }
  } catch {
    // Keep startup safe when storage is unavailable or malformed.
  }
  return DEFAULT_PREFERENCES;
}

export function writeAutoReconnect(enabled: boolean): boolean {
  try {
    const current = readDesktopPreferences();
    localStorage.setItem(KEY, JSON.stringify({ ...current, autoReconnect: enabled }));
    return true;
  } catch {
    return false;
  }
}
