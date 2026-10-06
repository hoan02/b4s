import type { EqPresetId } from "./device";

export type { EqPresetId };

export interface EqPresetMeta {
  id: EqPresetId;
  label: string;
  sub: string;
  dictSort: number;
  curve: number[];
}

export interface CustomEqPreset {
  id: string;
  label: string;
  bands: number[];
}

interface StoredCustomEqPresets {
  version: 1;
  presets: CustomEqPreset[];
}

const MAX_CUSTOM_PRESETS = 2;
const MAX_PRESET_LABEL_LENGTH = 60;
const MAX_PRESET_ID_LENGTH = 128;

export function defaultCustomBands(bandCount: number): number[] {
  return Array.from({ length: bandCount }, () => 0);
}

export function loadCustomEqPresets(storageKey: string, bandCount: number, minGain: number, maxGain: number): CustomEqPreset[] {
  try {
    const key = `b4s.eq.custom.${storageKey}`;
    const raw = localStorage.getItem(key);
    const parsed = raw ? JSON.parse(raw) : [];
    const isLegacy = Array.isArray(parsed);
    const stored = isLegacy
      ? parsed
      : parsed && parsed.version === 1 && Array.isArray(parsed.presets)
        ? parsed.presets
        : [];
    const presets = stored.filter(
      (item): item is CustomEqPreset =>
        item && typeof item.id === "string" && item.id.length > 0 && item.id.length <= MAX_PRESET_ID_LENGTH &&
        typeof item.label === "string" && item.label.trim().length > 0 && item.label.length <= MAX_PRESET_LABEL_LENGTH &&
        Array.isArray(item.bands) && item.bands.length === bandCount &&
        item.bands.every((gain: unknown) => typeof gain === "number" && Number.isFinite(gain) && gain >= minGain && gain <= maxGain)
    ).slice(0, MAX_CUSTOM_PRESETS);
    if (isLegacy) {
      try {
        localStorage.setItem(key, JSON.stringify({ version: 1, presets } satisfies StoredCustomEqPresets));
      } catch {
        // Keep valid legacy data usable when storage is read-only or unavailable.
      }
    }
    return presets;
  } catch {
    return [];
  }
}

export function saveCustomEqPresets(storageKey: string, presets: CustomEqPreset[]): void {
  const value: StoredCustomEqPresets = { version: 1, presets: presets.slice(0, MAX_CUSTOM_PRESETS) };
  localStorage.setItem(`b4s.eq.custom.${storageKey}`, JSON.stringify(value));
}
