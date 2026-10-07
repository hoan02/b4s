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
  | { kind: "findBuds"; start: boolean }
  | { kind: "setInEar"; enabled: boolean }
  | { kind: "setMultipoint"; enabled: boolean }
  | { kind: "restoreDefaults" }
  | { kind: "setAdaptiveLr"; enabled: boolean }
  | { kind: "setWindNoise"; enabled: boolean }
  | { kind: "setGesture"; layout: number; left: number | null; right: number | null };

export interface DeviceCommandResponse {
  contractVersion: 1;
  sessionId: number;
  snapshotRevision: number;
  disposition: "deviceStateObserved" | "transportAccepted" | "simulated";
}

async function applyDeviceCommand(command: DeviceCommand): Promise<DeviceCommandResponse> {
  const response = await invoke<unknown>("apply_device_command", {
    request: { contractVersion: 1, command },
  });
  if (response === null || typeof response !== "object" || Array.isArray(response)) {
    throw new Error("Invalid device command response");
  }
  const value = response as Record<string, unknown>;
  if (value.contractVersion !== 1 || !Number.isSafeInteger(value.sessionId) ||
    !Number.isSafeInteger(value.snapshotRevision) ||
    !["deviceStateObserved", "transportAccepted", "simulated"].includes(value.disposition as string)) {
    throw new Error("Invalid device command response");
  }
  return value as unknown as DeviceCommandResponse;
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

export async function setListeningState(state: ListeningStateRequest): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setListeningState", ...state });
}

export interface NoiseProfile {
  adaptive: boolean;
  maxLevel: number;
}

export function profileNoise(profile?: {
  supportsAdaptive: boolean;
  maxCustomLevel: number;
}): NoiseProfile {
  if (!profile) return { adaptive: false, maxLevel: 0 };
  return {
    adaptive: profile.supportsAdaptive,
    maxLevel: profile.maxCustomLevel,
  };
}

export async function setEqPreset(preset: EqPresetId | string): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setEqPreset", preset });
}

export async function setEqIndex(index: number): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setEqIndex", index });
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
): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setCustomEq", bands, dictSort, anc });
}

export async function setGameMode(enabled: boolean): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setGameMode", enabled });
}

export async function setSpatialMode(mode: SpatialMode): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setSpatialMode", mode });
}

export async function setBassBoost(level: number): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setBassBoost", level });
}

export async function setLdac(enabled: boolean): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setLdac", enabled });
}

export async function setHearingProtection(
  enabled: boolean,
  level: number
): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setHearingProtection", enabled, level });
}

export async function findBuds(start = true): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "findBuds", start });
}

export async function setInEar(enabled: boolean): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setInEar", enabled });
}

export async function setMultipoint(enabled: boolean): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setMultipoint", enabled });
}

export async function restoreDefaults(): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "restoreDefaults" });
}

export async function setAdaptiveLr(enabled: boolean): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setAdaptiveLr", enabled });
}

export async function setWindNoise(enabled: boolean): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setWindNoise", enabled });
}

export async function setGesture(
  layout: number,
  left: number | null,
  right: number | null
): Promise<DeviceCommandResponse> {
  return applyDeviceCommand({ kind: "setGesture", layout, left, right });
}
