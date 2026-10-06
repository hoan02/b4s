import { invoke, listen, type UnlistenFn } from "../lib/tauri";

export interface BatteryReading {
  percentage: number;
  charging: boolean;
  observedAtMs: number;
}

export interface DeviceSnapshot {
  schemaVersion: 1;
  sessionId: number;
  revision: number;
  modelId: string | null;
  deviceId: string | null;
  mock: boolean;
  battery: {
    left: BatteryReading | null;
    right: BatteryReading | null;
    case: BatteryReading | null;
  };
  anc: "off" | "anc" | "transparency" | null;
  eq: "balanced" | "bassBoost" | "voice" | "clear" | "hifiLive" | "pop" | "jazzRock" | "classical" | "acoustic" | "bassReduce" | "trebleReduce" | null;
  eqIndex: number | null;
  game: boolean | null;
  ldac: boolean | null;
  spatialEnabled: boolean | null;
  bassBoost: number | null;
  hearing: { enabled: boolean; level: number; observedAtMs: number } | null;
}

export function getDeviceSnapshot(): Promise<DeviceSnapshot> {
  return invoke<DeviceSnapshot>("get_device_snapshot");
}

export function onDeviceSnapshot(callback: (snapshot: DeviceSnapshot) => void): Promise<UnlistenFn> {
  return listen<DeviceSnapshot>("device://snapshot", (event) => callback(event.payload));
}
