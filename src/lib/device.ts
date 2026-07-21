/**
 * Device control + live state from Baseus protocol events
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import type { BatteryData } from "../components/Battery";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type AncMode = "off" | "anc" | "transparency";
export type EqPreset = "classic" | "bass" | "voice" | "clear" | "custom" | "game";

export interface DeviceBattery {
  left: number;
  right: number;
  case: number;
  leftCharging: boolean;
  rightCharging: boolean;
  caseCharging: boolean;
}

// Map protocol battery → UI BatteryData
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

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

export async function fetchBattery(): Promise<DeviceBattery> {
  return invoke<DeviceBattery>("get_battery");
}

export async function setAncMode(mode: AncMode, strength = 70): Promise<void> {
  await invoke("set_anc_mode", { mode, strength });
}

export async function setEqPreset(preset: EqPreset | string): Promise<void> {
  // Map UI ids → protocol names
  const map: Record<string, string> = {
    classic: "balanced",
    bass: "bass",
    voice: "voice",
    clear: "clear",
    custom: "balanced",
    game: "bass",
  };
  await invoke("set_eq_preset", { preset: map[preset] ?? preset });
}

export async function setGameMode(enabled: boolean): Promise<void> {
  await invoke("set_game_mode", { enabled });
}

export async function findBuds(): Promise<void> {
  await invoke("find_buds");
}

// ---------------------------------------------------------------------------
// Live events from protocol decoder
// ---------------------------------------------------------------------------

export function onBattery(cb: (b: DeviceBattery) => void): Promise<UnlistenFn> {
  return listen<DeviceBattery>("device://battery", (e) => cb(e.payload));
}

export function onAnc(cb: (mode: AncMode) => void): Promise<UnlistenFn> {
  return listen<string>("device://anc", (e) => {
    const m = String(e.payload).toLowerCase();
    if (m.includes("off")) cb("off");
    else if (m.includes("transp") || m.includes("ambient")) cb("transparency");
    else cb("anc");
  });
}

export function onEq(cb: (preset: string) => void): Promise<UnlistenFn> {
  return listen<string>("device://eq", (e) => cb(String(e.payload)));
}

export function onGameMode(cb: (on: boolean) => void): Promise<UnlistenFn> {
  return listen<boolean>("device://game", (e) => cb(!!e.payload));
}

export function onDeviceEvent(cb: (ev: unknown) => void): Promise<UnlistenFn> {
  return listen("device://event", (e) => cb(e.payload));
}

export function onRawNotify(cb: (raw: { hex?: string; cmd?: number }) => void): Promise<UnlistenFn> {
  return listen("ble://raw", (e) => cb(e.payload as { hex?: string; cmd?: number }));
}
