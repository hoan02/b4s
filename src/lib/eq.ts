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

/** APK 2.14.1 uses eight configurable bands for self-defined EQ. */
export const EQ_BANDS = [
  { id: "100", frequency: 100, label: "100", unit: "Hz" },
  { id: "200", frequency: 200, label: "200", unit: "Hz" },
  { id: "400", frequency: 400, label: "400", unit: "Hz" },
  { id: "800", frequency: 800, label: "800", unit: "Hz" },
  { id: "1k", frequency: 1000, label: "1K", unit: "Hz" },
  { id: "3k", frequency: 3000, label: "3K", unit: "Hz" },
  { id: "6k", frequency: 6000, label: "6K", unit: "Hz" },
  { id: "10k", frequency: 10000, label: "10K", unit: "Hz" },
] as const;

/** Local fallback for the APK's model/server-provided eq_sound_mode list. */
export const EQ_PRESETS: EqPresetMeta[] = [
  { id: "classic", label: "Classic", sub: "Cân bằng", dictSort: 0, curve: [0, 0, 0, 0, 0, 0, 0, 0] },
  { id: "bass", label: "Super Bass", sub: "Tăng trầm", dictSort: 1, curve: [4, 3, 2, 1, 0, -1, -2, -2] },
  { id: "cinema", label: "Cinema", sub: "Âm thanh phim", dictSort: 2, curve: [3, 1, 1, 2, 1, 2, 1, -1] },
  { id: "hifi", label: "Hi-Fi Live", sub: "Sân khấu", dictSort: 3, curve: [0, -2, 1, 2, 1, 2, 1, -1] },
  { id: "voice", label: "Clear Voice", sub: "Rõ giọng", dictSort: 4, curve: [2, -2, 1, 3, 3, 2, 0, -1] },
  { id: "dj", label: "DJ", sub: "DJ", dictSort: 5, curve: [3, 0, 3, 2, 1, 2, 1, -1] },
  { id: "pop", label: "Pop", sub: "Nhạc pop", dictSort: 6, curve: [2, -2, 3, 2, 1, 2, 1, -1] },
  { id: "jazz", label: "Jazz", sub: "Jazz", dictSort: 7, curve: [3, -1, 2, 1, 2, 2, 1, -1] },
  { id: "classical", label: "Classical", sub: "Cổ điển", dictSort: 8, curve: [-2, -3, 1, 2, 1, 2, 1, -1] },
  { id: "clear", label: "Treble Boost", sub: "Tăng cao", dictSort: 9, curve: [-2, -2, 1, 2, 3, 4, 3, 1] },
  { id: "original", label: "Original", sub: "Nguyên bản", dictSort: 10, curve: [0, 0, 0, 0, 0, 0, 0, 0] },
  { id: "rock", label: "Rock Classic", sub: "Rock", dictSort: 11, curve: [3, 1, 2, 3, 1, 2, 1, -1] },
];

export const EQ_LABEL: Record<string, string> = Object.fromEntries(
  EQ_PRESETS.map((p) => [p.id, p.label])
);

export function curveForPreset(id: EqPresetId): number[] {
  return EQ_PRESETS.find((p) => p.id === id)?.curve.slice() ?? defaultCustomBands();
}

export function presetSort(id: EqPresetId): number {
  return EQ_PRESETS.find((p) => p.id === id)?.dictSort ?? 0;
}

export function clampBand(v: number): number {
  return Math.max(-12, Math.min(12, Math.round(v)));
}

export function defaultCustomBands(): number[] {
  return Array.from({ length: EQ_BANDS.length }, () => 0);
}

export function loadCustomEqPresets(storageKey: string): CustomEqPreset[] {
  try {
    const raw = localStorage.getItem(`b4s.eq.custom.${storageKey}`);
    const parsed = raw ? JSON.parse(raw) : [];
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (item): item is CustomEqPreset =>
        item && typeof item.id === "string" && typeof item.label === "string" &&
        Array.isArray(item.bands) && item.bands.length === EQ_BANDS.length
    );
  } catch {
    return [];
  }
}

export function saveCustomEqPresets(storageKey: string, presets: CustomEqPreset[]): void {
  localStorage.setItem(`b4s.eq.custom.${storageKey}`, JSON.stringify(presets));
}
