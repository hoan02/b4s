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
  headphoneCandidate?: boolean;
  modelId?: string | null;
  modelName?: string | null;
  deviceProfile?: DeviceProfile;
  /** verified | experimental | scanOnly */
  support?: string | null;
  /** Dual-entry / pairing tip from backend */
  hint?: string | null;
  imageUrl?: string | null;
  imageProvenance?: string;
  colorVariants?: string[];
  serial?: string | null;
  advertisedServices?: string[];
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
  protocol: "bp1Pro" | "baseusAaBaExperimental" | "unknown" | string;
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

function decodeContractV1<T>(payload: unknown, contractName: string): T {
  if (
    payload === null ||
    typeof payload !== "object" ||
    (payload as { contractVersion?: unknown }).contractVersion !== 1
  ) {
    throw new Error(`Unsupported ${contractName} contract version`);
  }
  return payload as T;
}

function listenContractV1<T>(
  eventName: string,
  contractName: string,
  cb: (payload: T) => void
): Promise<UnlistenFn> {
  return listen<unknown>(eventName, (event) => {
    try {
      cb(decodeContractV1<T>(event.payload, contractName));
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
  return decodeContractV1<ScanStatus>(await invoke<unknown>("ble_get_scan_status"), "scan status");
}

export async function getConnection(): Promise<ConnectionState> {
  return decodeContractV1<ConnectionState>(await invoke<unknown>("ble_get_connection"), "connection state");
}

export async function getLinkHealth(): Promise<LinkHealth> {
  return decodeContractV1<LinkHealth>(await invoke<unknown>("ble_get_link_health"), "link health");
}

// ---------------------------------------------------------------------------
// Event listeners
// ---------------------------------------------------------------------------

export function onScanStatus(cb: (status: ScanStatus) => void): Promise<UnlistenFn> {
  return listenContractV1("ble://scan-status", "scan status", cb);
}

export function onConnection(cb: (state: ConnectionState) => void): Promise<UnlistenFn> {
  return listenContractV1("ble://connection", "connection state", cb);
}

export function onLinkHealth(cb: (link: LinkHealth) => void): Promise<UnlistenFn> {
  return listenContractV1("ble://link", "link health", cb);
}


export function onConnecting(cb: (id: string) => void): Promise<UnlistenFn> {
  return listenContractV1<{ contractVersion: 1; deviceId: string }>(
    "ble://connecting",
    "connecting state",
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
