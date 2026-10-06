/**
 * Device control + live state — listening features
 */

import { invoke } from "./tauri";

export type AncMode = "off" | "anc" | "transparency";
export type TransparencyMode = "full" | "voice";
export type NoiseEnvironment = 101 | 102 | 103 | 108;
export type SpatialMode = "off" | "music" | "cinema" | "game";
export type EqPresetId = string;

type DeviceCommand =
  | { kind: "setListeningState"; mode: AncMode; transparencyMode: TransparencyMode; adaptive: boolean; environment: NoiseEnvironment; level: number }
  | { kind: "setEqPreset"; preset: EqPresetId }
  | { kind: "setEqIndex"; index: number }
  | { kind: "setCustomEq"; bands: EqBandPayload[]; dictSort: number; anc: boolean }
  | { kind: "setGameMode"; enabled: boolean }
  | { kind: "setSpatialMode"; mode: SpatialMode }
  | { kind: "setBassBoost"; level: number }
  | { kind: "setLdac"; enabled: boolean }
  | { kind: "setHearingProtection"; enabled: boolean; level: number }
  | { kind: "findBuds"; start: boolean };

async function applyDeviceCommand(command: DeviceCommand): Promise<void> {
  await invoke("apply_device_command", {
    request: { contractVersion: 1, command },
  });
}

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
  await applyDeviceCommand({ kind: "setListeningState", ...state });
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
  await applyDeviceCommand({ kind: "setEqPreset", preset });
}

export async function setEqIndex(index: number): Promise<void> {
  await applyDeviceCommand({ kind: "setEqIndex", index });
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
  await applyDeviceCommand({ kind: "setCustomEq", bands, dictSort, anc });
}

export async function setGameMode(enabled: boolean): Promise<void> {
  await applyDeviceCommand({ kind: "setGameMode", enabled });
}

export async function setSpatialMode(mode: SpatialMode): Promise<void> {
  await applyDeviceCommand({ kind: "setSpatialMode", mode });
}

export async function setBassBoost(level: number): Promise<void> {
  await applyDeviceCommand({ kind: "setBassBoost", level });
}

export async function setLdac(enabled: boolean): Promise<void> {
  await applyDeviceCommand({ kind: "setLdac", enabled });
}

export async function setHearingProtection(
  enabled: boolean,
  level: number
): Promise<void> {
  await applyDeviceCommand({ kind: "setHearingProtection", enabled, level });
}

export async function findBuds(start = true): Promise<void> {
  await applyDeviceCommand({ kind: "findBuds", start });
}
