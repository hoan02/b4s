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

function decodeCurrentPreferences(raw: string): DesktopPreferences | null {
  try {
    const value: unknown = JSON.parse(raw);
    if (
      typeof value === "object" && value !== null &&
      "version" in value && value.version === 2 &&
      "autoReconnect" in value && typeof value.autoReconnect === "boolean" &&
      "experimentalMode" in value && typeof value.experimentalMode === "boolean"
    ) {
      return {
        version: 2,
        autoReconnect: value.autoReconnect,
        experimentalMode: value.experimentalMode,
      };
    }
  } catch {
    // Invalid persisted values are recovered to the safe defaults.
  }
  return null;
}

function migrateLegacyPreferences(raw: string): DesktopPreferences | null {
  try {
    const value: unknown = JSON.parse(raw);
    if (
      typeof value === "object" && value !== null &&
      "version" in value && value.version === 1 &&
      "autoReconnect" in value && typeof value.autoReconnect === "boolean"
    ) {
      return {
        version: 2,
        autoReconnect: value.autoReconnect,
        experimentalMode: false,
      };
    }
  } catch {
    // Invalid legacy values are discarded during recovery.
  }
  return null;
}

export function readDesktopPreferences(): DesktopPreferences {
  try {
    const current = localStorage.getItem(KEY);
    if (current !== null) {
      return decodeCurrentPreferences(current) ?? DEFAULT_PREFERENCES;
    }

    const legacy = localStorage.getItem(LEGACY_KEY);
    if (legacy === null) return DEFAULT_PREFERENCES;

    const migrated = migrateLegacyPreferences(legacy);
    if (!migrated) {
      try { localStorage.removeItem(LEGACY_KEY); } catch { /* Storage recovery is best effort. */ }
      return DEFAULT_PREFERENCES;
    }

    try {
      localStorage.setItem(KEY, JSON.stringify(migrated));
      localStorage.removeItem(LEGACY_KEY);
    } catch {
      // Keep the migrated in-memory settings if storage is temporarily read-only.
    }
    return migrated;
  } catch {
    // Keep startup safe when storage is unavailable.
    return DEFAULT_PREFERENCES;
  }
}

function persistPreferences(preferences: DesktopPreferences): boolean {
  try {
    localStorage.setItem(KEY, JSON.stringify(preferences));
  } catch {
    return false;
  }
  try { localStorage.removeItem(LEGACY_KEY); } catch { /* Legacy cleanup is best effort. */ }
  return true;
}

export function writeAutoReconnect(enabled: boolean): boolean {
  return persistPreferences({ ...readDesktopPreferences(), autoReconnect: enabled });
}

export function writeExperimentalMode(enabled: boolean): boolean {
  return persistPreferences({ ...readDesktopPreferences(), experimentalMode: enabled });
}
