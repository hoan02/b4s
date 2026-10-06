export interface DesktopPreferences {
  version: 2;
  autoReconnect: boolean;
  experimentalMode: boolean;
}

const KEY = "b4s.desktop.preferences.v2";
const LEGACY_KEY = "b4s.desktop.preferences.v1";

const DEFAULT_PREFERENCES: DesktopPreferences = {
  version: 2,
  autoReconnect: false,
  experimentalMode: false,
};

export function readDesktopPreferences(): DesktopPreferences {
  try {
    const value: unknown = JSON.parse(
      localStorage.getItem(KEY) ?? localStorage.getItem(LEGACY_KEY) ?? "null",
    );
    if (
      typeof value === "object" && value !== null &&
      "version" in value && (value.version === 1 || value.version === 2) &&
      "autoReconnect" in value && typeof value.autoReconnect === "boolean"
    ) {
      return {
        version: 2,
        autoReconnect: value.autoReconnect,
        experimentalMode:
          value.version === 2 && "experimentalMode" in value &&
          typeof value.experimentalMode === "boolean"
            ? value.experimentalMode
            : false,
      };
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

export function writeExperimentalMode(enabled: boolean): boolean {
  try {
    const current = readDesktopPreferences();
    localStorage.setItem(KEY, JSON.stringify({ ...current, experimentalMode: enabled }));
    return true;
  } catch {
    return false;
  }
}
