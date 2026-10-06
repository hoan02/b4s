/**
 * Frontend BLE API wrapper
 * Talks to Rust backend via Tauri invoke + events
 */

import { invoke, listen, type UnlistenFn } from "./tauri";
import { rememberDevice } from "./reconnect";

// ---------------------------------------------------------------------------
// Types (mirror Rust serde)
// ---------------------------------------------------------------------------

export interface BleDevice {
  id: string;
  name: string;
  address: string;
  rssi: number;
  isBaseus: boolean;
  connected: boolean;
  headphoneCandidate: boolean;
  modelId: string | null;
  modelName: string | null;
  deviceProfile: DeviceProfile;
  /** verified | experimental | scanOnly */
  support: string | null;
  /** Dual-entry / pairing tip from backend */
  hint: string | null;
  imageUrl: string | null;
  imageProvenance: string;
  colorVariants: string[];
  serial: string | null;
  advertisedServices: string[];
}

export interface DeviceProfile {
  capabilities: { anc: boolean; eq: boolean; customEq: boolean; gameMode: boolean; bassBoost: boolean; spatial: boolean; ldac: boolean; hearingProtection: boolean; findBuds: boolean };
  connection: {
    transport: "bleGatt" | "unresolved";
    framing: "bareAaBa" | "headphone789c" | "unresolved";
    serviceUuid: string | null;
    writeUuid: string | null;
    notifyUuid: string | null;
    handshake: number[];
    initStateQuery: boolean;
    firmwareVersions: string[];
    provenance: string;
  } | null;

  modelId?: string | null;
  modelName?: string | null;
  firmware?: string | null;
  protocol: "bp1Pro" | "unknown";
  verified: boolean;
  noise: {
    supportsAdaptive: boolean;
    environments: number[];
    maxCustomLevel: number;
    supportsTransparencyVoice: boolean;
  };
}

export interface ModelInfo {
  id: string;
  displayName: string;
  namePatterns: string[];
  support: "verified" | "experimental" | "scanOnly";
  protocol: string;
  hasAnc: boolean;
  hasEq: boolean;
  hasGameMode: boolean;
  category: string;
  /** Product family from official app (e.g. Bass BP1 / EP10) */
  group?: string;
  capabilities: {
    anc: boolean;
    eq: boolean;
    gameMode: boolean;
    bassBoost: boolean;
    ldac: boolean;
    hearingProtection: boolean;
    spatial: boolean;
  };
  transport: {
    serviceUuid: string | null;
    writeUuid: string | null;
    notifyUuid: string | null;
    useSelfUuid: boolean;
    requiredAdvertisedService: boolean;
  };
  colorVariants: string[];
  imageUrl: string | null;
  imageProvenance: "local" | "cached" | "remote" | "fallback" | string;
}

export async function listModels(): Promise<ModelInfo[]> {
  return invoke<ModelInfo[]>("list_models");
}

export interface ModelProfile {
  id: string;
  displayName: string;
  aliases: string[];
  support: "verified" | "experimental" | "scanOnly" | string;
  protocolFamily: string;
  category: string;
  group: string;
  capabilities: Record<string, boolean>;
  noise: {
    supportsAdaptive: boolean;
    environments: number[];
    maxCustomLevel: number;
    supportsTransparencyVoice: boolean;
  };
  eq: {
    bands: number[];
    minGain: number;
    maxGain: number;
    customSlots: number;
    presets: Array<{ id: string; label: string; description: string; dictSort: number; curve: number[] }>;
  } | null;
  image: string | null;
}

export async function listModelProfiles(): Promise<ModelProfile[]> {
  return invoke<ModelProfile[]>("list_model_profiles");
}

/** Proof of real control link — not just "connected" UI flag */
export type LinkLevel = "live" | "waiting" | "dead" | "demo" | "offline";

export interface LinkHealth {
  contractVersion: 1;
  connected: boolean;
  mock: boolean;
  peripheralConnected: boolean;
  hasWriteUuid: boolean;
  hasNotifyUuid: boolean;
  handshakeOk: boolean;
  notifyCount: number;
  txCount: number;
  lastNotifyMs: number | null;
  lastTxMs: number | null;
  lastRxHex: string | null;
  lastTxHex: string | null;
  writeChar: string | null;
  notifyChar: string | null;
  level: LinkLevel;
  message: string;
}

export interface ConnectionState {
  contractVersion: 1;
  connected: boolean;
  device: BleDevice | null;
  error: string | null;
  link: LinkHealth;
}

export interface ScanStatus {
  contractVersion: 1;
  scanning: boolean;
  devices: BleDevice[];
  error: string | null;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isNullable(value: unknown, guard: (item: unknown) => boolean): boolean {
  return value === null || guard(value);
}

function isCounter(value: unknown): value is number {
  return Number.isSafeInteger(value) && (value as number) >= 0;
}

function isLinkHealth(value: unknown): value is LinkHealth {
  if (!isRecord(value)) return false;
  return value.contractVersion === 1 &&
    ["connected", "mock", "peripheralConnected", "hasWriteUuid", "hasNotifyUuid", "handshakeOk"]
      .every((key) => typeof value[key] === "boolean") &&
    ["notifyCount", "txCount"].every((key) => isCounter(value[key])) &&
    ["lastNotifyMs", "lastTxMs", "lastRxHex", "lastTxHex", "writeChar", "notifyChar"]
      .every((key) => isNullable(value[key], (item) =>
        key.endsWith("Ms") ? isCounter(item) : typeof item === "string")) &&
    ["live", "waiting", "dead", "demo", "offline"].includes(value.level as string) &&
    typeof value.message === "string";
}

function isBleDevice(value: unknown): value is BleDevice {
  if (!isRecord(value)) return false;
  return ["id", "name", "address"].every((key) => typeof value[key] === "string") &&
    typeof value.rssi === "number" &&
    typeof value.isBaseus === "boolean" &&
    typeof value.connected === "boolean" &&
    typeof value.headphoneCandidate === "boolean" &&
    ["modelId", "modelName", "support", "hint", "imageUrl", "serial"].every((key) =>
      isNullable(value[key], (item) => typeof item === "string")) &&
    typeof value.imageProvenance === "string" &&
    Array.isArray(value.colorVariants) && value.colorVariants.every((item) => typeof item === "string") &&
    Array.isArray(value.advertisedServices) && value.advertisedServices.every((item) => typeof item === "string") &&
    isDeviceProfile(value.deviceProfile);
}

function isDeviceProfile(value: unknown): value is DeviceProfile {
  if (!isRecord(value) || !isRecord(value.capabilities) || !isRecord(value.noise)) return false;
  const capabilities = value.capabilities;
  const noise = value.noise;
  const connection = value.connection;
  const validConnection = connection === null || (isRecord(connection) &&
    ["bleGatt", "unresolved"].includes(connection.transport as string) &&
    ["bareAaBa", "headphone789c", "unresolved"].includes(connection.framing as string) &&
    ["serviceUuid", "writeUuid", "notifyUuid"].every((key) =>
      isNullable(connection[key], (item) => typeof item === "string")) &&
    Array.isArray(connection.handshake) &&
    connection.handshake.every((item) => Number.isInteger(item) && item >= 0 && item <= 255) &&
    typeof connection.initStateQuery === "boolean" &&
    Array.isArray(connection.firmwareVersions) &&
    connection.firmwareVersions.every((item) => typeof item === "string") &&
    typeof connection.provenance === "string");
  return ["anc", "eq", "customEq", "gameMode", "bassBoost", "spatial", "ldac", "hearingProtection", "findBuds"]
      .every((key) => typeof capabilities[key] === "boolean") &&
    isNullable(value.modelId, (item) => typeof item === "string") &&
    isNullable(value.modelName, (item) => typeof item === "string") &&
    isNullable(value.firmware, (item) => typeof item === "string") &&
    ["bp1Pro", "unknown"].includes(value.protocol as string) &&
    typeof value.verified === "boolean" &&
    typeof noise.supportsAdaptive === "boolean" &&
    Array.isArray(noise.environments) && noise.environments.every((item) => Number.isInteger(item)) &&
    typeof noise.maxCustomLevel === "number" &&
    typeof noise.supportsTransparencyVoice === "boolean" && validConnection;
}

function isConnectionState(value: unknown): value is ConnectionState {
  return isRecord(value) && value.contractVersion === 1 &&
    typeof value.connected === "boolean" &&
    isNullable(value.device, isBleDevice) &&
    isNullable(value.error, (item) => typeof item === "string") &&
    isLinkHealth(value.link);
}

function isScanStatus(value: unknown): value is ScanStatus {
  return isRecord(value) && value.contractVersion === 1 &&
    typeof value.scanning === "boolean" &&
    Array.isArray(value.devices) && value.devices.every(isBleDevice) &&
    isNullable(value.error, (item) => typeof item === "string");
}

function decodeContractV1<T>(
  payload: unknown,
  contractName: string,
  guard: (value: unknown) => value is T
): T {
  if (
    !isRecord(payload) || payload.contractVersion !== 1
  ) {
    throw new Error(`Unsupported ${contractName} contract version`);
  }
  if (!guard(payload)) throw new Error(`Invalid ${contractName} payload`);
  return payload as T;
}

function listenContractV1<T>(
  eventName: string,
  contractName: string,
  guard: (value: unknown) => value is T,
  cb: (payload: T) => void
): Promise<UnlistenFn> {
  return listen<unknown>(eventName, (event) => {
    try {
      cb(decodeContractV1(event.payload, contractName, guard));
    } catch (error) {
      console.error(`[BLE] rejected ${contractName} event`, error);
    }
  });
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

export async function checkAdapter(): Promise<boolean> {
  try {
    return await invoke<boolean>("ble_check_adapter");
  } catch {
    return false;
  }
}

export async function startScan(mock = false): Promise<void> {
  await invoke("ble_start_scan", { mock });
}

export async function stopScan(): Promise<void> {
  await invoke("ble_stop_scan");
}

export async function connect(deviceId: string, mock = false): Promise<BleDevice> {
  const device = await invoke<BleDevice>("ble_connect", { deviceId, mock });
  if (!mock) rememberDevice(device);
  return device;
}

export async function disconnect(): Promise<void> {
  await invoke("ble_disconnect");
}

export async function getScanStatus(): Promise<ScanStatus> {
  return decodeContractV1(await invoke<unknown>("ble_get_scan_status"), "scan status", isScanStatus);
}

export async function getConnection(): Promise<ConnectionState> {
  return decodeContractV1(await invoke<unknown>("ble_get_connection"), "connection state", isConnectionState);
}

export async function getLinkHealth(): Promise<LinkHealth> {
  return decodeContractV1(await invoke<unknown>("ble_get_link_health"), "link health", isLinkHealth);
}

// ---------------------------------------------------------------------------
// Event listeners
// ---------------------------------------------------------------------------

export function onScanStatus(cb: (status: ScanStatus) => void): Promise<UnlistenFn> {
  return listenContractV1("ble://scan-status", "scan status", isScanStatus, cb);
}

export function onConnection(cb: (state: ConnectionState) => void): Promise<UnlistenFn> {
  return listenContractV1("ble://connection", "connection state", isConnectionState, cb);
}

export function onLinkHealth(cb: (link: LinkHealth) => void): Promise<UnlistenFn> {
  return listenContractV1("ble://link", "link health", isLinkHealth, cb);
}


export function onConnecting(cb: (id: string) => void): Promise<UnlistenFn> {
  return listenContractV1<{ contractVersion: 1; deviceId: string }>(
    "ble://connecting",
    "connecting state",
    (value): value is { contractVersion: 1; deviceId: string } =>
      isRecord(value) && value.contractVersion === 1 && typeof value.deviceId === "string",
    (state) => cb(state.deviceId)
  );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** RSSI → signal bars 1-4 */
export function rssiToBars(rssi: number): number {
  if (rssi >= -50) return 4;
  if (rssi >= -60) return 3;
  if (rssi >= -70) return 2;
  return 1;
}

export function rssiLabel(rssi: number): string {
  if (rssi >= -50) return "Excellent";
  if (rssi >= -60) return "Good";
  if (rssi >= -70) return "Fair";
  return "Weak";
}

export function emptyLink(): LinkHealth {
  return {
    contractVersion: 1,
    connected: false,
    mock: false,
    peripheralConnected: false,
    hasWriteUuid: false,
    hasNotifyUuid: false,
    handshakeOk: false,
    notifyCount: 0,
    txCount: 0,
    lastNotifyMs: null,
    lastTxMs: null,
    lastRxHex: null,
    lastTxHex: null,
    writeChar: null,
    notifyChar: null,
    level: "offline",
    message: "Not connected",
  };
}
