/** Confirmed selection is separate from the local custom filter draft. */
export type EqSelection =
  | { kind: "unknown"; wireIndex: number | null }
  | { kind: "preset"; wireIndex: number; id: string }
  | { kind: "custom"; wireIndex: 101 };

export function resolveEqSelection(
  wireIndex: number | null,
  presets: ReadonlyArray<{ id: string; dictSort: number }>,
  customSupported: boolean,
): EqSelection {
  if (wireIndex === 101 && customSupported) return { kind: "custom", wireIndex };
  const preset = presets.find((preset) => preset.dictSort === wireIndex);
  return preset
    ? { kind: "preset", wireIndex: preset.dictSort, id: preset.id }
    : { kind: "unknown", wireIndex };
}
