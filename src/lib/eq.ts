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

export function defaultCustomBands(bandCount: number): number[] {
  return Array.from({ length: bandCount }, () => 0);
}

export function loadCustomEqPresets(storageKey: string, bandCount: number, minGain: number, maxGain: number): CustomEqPreset[] {
  try {
    const raw = localStorage.getItem(`b4s.eq.custom.${storageKey}`);
    const parsed = raw ? JSON.parse(raw) : [];
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (item): item is CustomEqPreset =>
        item && typeof item.id === "string" && typeof item.label === "string" &&
        Array.isArray(item.bands) && item.bands.length === bandCount &&
        item.bands.every((gain: unknown) => typeof gain === "number" && Number.isFinite(gain) && gain >= minGain && gain <= maxGain)
    );
  } catch {
    return [];
  }
}

export function saveCustomEqPresets(storageKey: string, presets: CustomEqPreset[]): void {
  localStorage.setItem(`b4s.eq.custom.${storageKey}`, JSON.stringify(presets));
}
