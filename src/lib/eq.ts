/**
 * EQ presets + custom band model (UI mirrors official ear-EQ screens).
 * Wire: BA43 + index for presets; custom is local curve (best-effort BLE later).
 */

import type { EqPresetId } from "./device";

export type { EqPresetId };

export interface EqPresetMeta {
  id: EqPresetId;
  label: string;
  sub: string;
  /** Relative band gains for visualization / custom seed (-6…+6) */
  curve: number[];
}

/** 5 bands: 60 · 250 · 1k · 4k · 8k Hz */
export const EQ_BANDS = [
  { id: "60", label: "60", unit: "Hz" },
  { id: "250", label: "250", unit: "Hz" },
  { id: "1k", label: "1k", unit: "Hz" },
  { id: "4k", label: "4k", unit: "Hz" },
  { id: "8k", label: "8k", unit: "Hz" },
] as const;

export const EQ_PRESETS: EqPresetMeta[] = [
  { id: "classic", label: "Classic", sub: "Cân bằng", curve: [0, 0, 0, 0, 0] },
  { id: "bass", label: "Powerful Bass", sub: "Tăng trầm", curve: [5, 3, 0, -1, -2] },
  { id: "hifi", label: "Hi-Fi Live", sub: "Sân khấu", curve: [1, 0, 1, 2, 3] },
  { id: "pop", label: "Pop", sub: "Nhạc pop", curve: [-1, 2, 3, 1, 0] },
  { id: "jazz", label: "Jazz Rock", sub: "Jazz / rock", curve: [2, 1, 0, 2, 1] },
  { id: "classical", label: "Classical", sub: "Cổ điển", curve: [0, 0, 1, 2, 2] },
  { id: "clear", label: "Clear Treble", sub: "Tăng cao", curve: [-2, -1, 0, 3, 5] },
  { id: "acoustic", label: "Acoustic", sub: "Acoustic", curve: [1, 2, 1, 0, 1] },
  { id: "voice", label: "Voice", sub: "Giọng nói", curve: [-2, 0, 4, 3, 0] },
  { id: "bassReduce", label: "Giảm trầm", sub: "Less bass", curve: [-4, -2, 0, 1, 1] },
  { id: "trebleReduce", label: "Giảm cao", sub: "Less treble", curve: [1, 1, 0, -2, -4] },
];

export const EQ_LABEL: Record<string, string> = Object.fromEntries(
  EQ_PRESETS.map((p) => [p.id, p.label])
);

export function curveForPreset(id: EqPresetId): number[] {
  return (
    EQ_PRESETS.find((p) => p.id === id)?.curve.slice() ?? [0, 0, 0, 0, 0]
  );
}

export function clampBand(v: number): number {
  return Math.max(-6, Math.min(6, Math.round(v)));
}

export function defaultCustomBands(): number[] {
  return [0, 0, 0, 0, 0];
}
