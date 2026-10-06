/**
 * Device control + live state — listening features
 */

import { invoke } from "./tauri";

export type AncMode = "off" | "anc" | "transparency";
export type TransparencyMode = "full" | "voice";
export type NoiseEnvironment = 101 | 102 | 103 | 108;
export type SpatialMode = "off" | "music" | "cinema" | "game";
export type EqPresetId = string;

export async function queryBattery(): Promise<void> {
  await invoke("query_battery");
}

export interface ListeningStateRequest {
  mode: AncMode;
  transparencyMode: TransparencyMode;
  adaptive: boolean;
  environment: NoiseEnvironment;
  level: number;
}

export async function setListeningState(state: ListeningStateRequest): Promise<void> {
  await invoke("set_listening_state", state as unknown as Record<string, unknown>);
}

export interface NoiseProfile {
  adaptive: boolean;
  maxLevel: 0 | 3 | 5;
}

export function profileNoise(profile?: {
  supportsAdaptive: boolean;
  maxCustomLevel: number;
}): NoiseProfile {
  if (!profile) return { adaptive: false, maxLevel: 0 };
  return {
    adaptive: profile.supportsAdaptive,
    maxLevel: profile.maxCustomLevel === 3 ? 3 : profile.maxCustomLevel > 0 ? 5 : 0,
  };
}

export async function setEqPreset(preset: EqPresetId | string): Promise<void> {
  await invoke("set_eq_preset", { preset });
}

export async function setEqIndex(index: number): Promise<void> {
  await invoke("set_eq_index", { index });
}

export interface EqBandPayload {
  frequency: number;
  qValue: number;
  gain: number;
  filter: number;
}

export async function setCustomEq(
  bands: EqBandPayload[],
  dictSort: number,
  anc: boolean
): Promise<void> {
  await invoke("set_custom_eq", { bands, dictSort, anc });
}

export async function setGameMode(enabled: boolean): Promise<void> {
  await invoke("set_game_mode", { enabled });
}

export async function setSpatialMode(mode: SpatialMode): Promise<void> {
  await invoke("set_spatial_mode", { mode });
}

export async function setBassBoost(level: number): Promise<void> {
  await invoke("set_bass_boost", { level });
}

export async function setLdac(enabled: boolean): Promise<void> {
  await invoke("set_ldac", { enabled });
}

export async function setHearingProtection(
  enabled: boolean,
  level: number
): Promise<void> {
  await invoke("set_hearing_protection", { enabled, level });
}

export async function findBuds(start = true): Promise<void> {
  await invoke("find_buds", { start });
}
