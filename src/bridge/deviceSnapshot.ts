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

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isNullable(value: unknown, guard: (item: unknown) => boolean): boolean {
  return value === null || guard(value);
}

function isBatteryReading(value: unknown): boolean {
  return isRecord(value) && typeof value.percentage === "number" &&
    typeof value.charging === "boolean" && typeof value.observedAtMs === "number";
}

function isDeviceSnapshot(value: unknown): value is DeviceSnapshot {
  if (!isRecord(value) || value.schemaVersion !== 1 ||
    !Number.isSafeInteger(value.sessionId) || !Number.isSafeInteger(value.revision) ||
    !isNullable(value.modelId, (item) => typeof item === "string") ||
    !isNullable(value.deviceId, (item) => typeof item === "string") ||
    typeof value.mock !== "boolean" || !isRecord(value.battery)) return false;

  const battery = value.battery;
  const hearing = value.hearing;
  const validHearing = hearing === null || (isRecord(hearing) &&
    typeof hearing.enabled === "boolean" && typeof hearing.level === "number" &&
    typeof hearing.observedAtMs === "number");
  return ["left", "right", "case"].every((key) =>
    isNullable(battery[key], isBatteryReading)) &&
    isNullable(value.anc, (item) => ["off", "anc", "transparency"].includes(item as string)) &&
    isNullable(value.eq, (item) => [
      "balanced", "bassBoost", "voice", "clear", "hifiLive", "pop", "jazzRock",
      "classical", "acoustic", "bassReduce", "trebleReduce",
    ].includes(item as string)) &&
    isNullable(value.eqIndex, (item) => Number.isInteger(item)) &&
    ["game", "ldac", "spatialEnabled"].every((key) =>
      isNullable(value[key], (item) => typeof item === "boolean")) &&
    isNullable(value.bassBoost, (item) => Number.isInteger(item)) && validHearing;
}

function decodeSnapshotV1(payload: unknown): DeviceSnapshot {
  if (!isRecord(payload) || payload.schemaVersion !== 1) {
    throw new Error("Unsupported device snapshot schema version");
  }
  if (!isDeviceSnapshot(payload)) throw new Error("Invalid device snapshot payload");
  return payload;
}

export async function getDeviceSnapshot(): Promise<DeviceSnapshot> {
  return decodeSnapshotV1(await invoke<unknown>("get_device_snapshot"));
}

export function onDeviceSnapshot(callback: (snapshot: DeviceSnapshot) => void): Promise<UnlistenFn> {
  return listen<unknown>("device://snapshot", (event) => {
    try {
      callback(decodeSnapshotV1(event.payload));
    } catch (error) {
      console.error("[Device] rejected snapshot event", error);
    }
  });
}
