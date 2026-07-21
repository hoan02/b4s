/**
 * Device control + live state — listening features
 */

import { invoke, listen, type UnlistenFn } from "./tauri";
import type { BatteryData } from "../components/Battery";

export type AncMode = "off" | "anc" | "transparency";
export type SpatialMode = "off" | "music" | "cinema" | "game";
export type EqPresetId =
  | "classic"
  | "bass"
  | "hifi"
  | "pop"
  | "jazz"
  | "classical"
  | "clear"
  | "acoustic"
  | "bassReduce"
  | "trebleReduce"
  | "voice";

export interface DeviceBattery {
  left: number;
  right: number;
  case: number;
  leftCharging: boolean;
  rightCharging: boolean;
  caseCharging: boolean;
}

export function toBatteryData(b: DeviceBattery): BatteryData {
  return {
    left: b.left,
    right: b.right,
    case: b.case,
    leftCharging: b.leftCharging,
    rightCharging: b.rightCharging,
    caseCharging: b.caseCharging,
  };
}

export async function fetchBattery(): Promise<DeviceBattery> {
  return invoke<DeviceBattery>("get_battery");
}

export async function queryBattery(): Promise<DeviceBattery> {
  return invoke<DeviceBattery>("query_battery");
}

export async function setAncMode(mode: AncMode, strength = 70): Promise<void> {
  await invoke("set_anc_mode", { mode, strength });
}

export async function setEqPreset(preset: EqPresetId | string): Promise<void> {
  await invoke("set_eq_preset", { preset });
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

export async function findBuds(): Promise<void> {
  await invoke("find_buds");
}

export function onBattery(cb: (b: DeviceBattery) => void): Promise<UnlistenFn> {
  return listen<DeviceBattery>("device://battery", (e) => cb(e.payload));
}

export function onAnc(cb: (mode: AncMode) => void): Promise<UnlistenFn> {
  return listen<string>("device://anc", (e) => {
    // Strict parse — do NOT default unknown → "anc" (that snapped UI to Giảm ồn)
    const m = String(e.payload ?? "")
      .toLowerCase()
      .replace(/[^a-z]/g, "");
    if (m === "off" || m === "normal") cb("off");
    else if (m === "transparency" || m === "ambient" || m === "transp")
      cb("transparency");
    else if (m === "anc" || m === "noisereduction" || m === "noisereduce")
      cb("anc");
    // else: ignore garbage / partial payloads
  });
}

export function onEq(cb: (preset: string) => void): Promise<UnlistenFn> {
  return listen<string>("device://eq", (e) => cb(String(e.payload)));
}

export function onGameMode(cb: (on: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>("device://game", (e) => cb(!!e.payload));
}

export function onRawNotify(
  cb: (raw: { hex?: string; cmd?: number }) => void
): Promise<UnlistenFn> {
  return listen("ble://raw", (e) => cb(e.payload as { hex?: string; cmd?: number }));
}
