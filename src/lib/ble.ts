/**
 * Frontend BLE API wrapper
 * Talks to Rust backend via Tauri invoke + events
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

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
  modelId?: string | null;
  modelName?: string | null;
  /** verified | experimental | scanOnly */
  support?: string | null;
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
}

export async function listModels(): Promise<ModelInfo[]> {
  return invoke<ModelInfo[]>("list_models");
}

export interface ConnectionState {
  connected: boolean;
  device: BleDevice | null;
  error: string | null;
}

export interface ScanStatus {
  scanning: boolean;
  devices: BleDevice[];
  error: string | null;
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
  return invoke<BleDevice>("ble_connect", { deviceId, mock });
}

export async function disconnect(): Promise<void> {
  await invoke("ble_disconnect");
}

export async function getScanStatus(): Promise<ScanStatus> {
  return invoke<ScanStatus>("ble_get_scan_status");
}

export async function getConnection(): Promise<ConnectionState> {
  return invoke<ConnectionState>("ble_get_connection");
}

// ---------------------------------------------------------------------------
// Event listeners
// ---------------------------------------------------------------------------

export function onScanStatus(cb: (status: ScanStatus) => void): Promise<UnlistenFn> {
  return listen<ScanStatus>("ble://scan-status", (e) => cb(e.payload));
}

export function onDevice(cb: (device: BleDevice) => void): Promise<UnlistenFn> {
  return listen<BleDevice>("ble://device", (e) => cb(e.payload));
}

export function onConnection(cb: (state: ConnectionState) => void): Promise<UnlistenFn> {
  return listen<ConnectionState>("ble://connection", (e) => cb(e.payload));
}

export function onConnected(cb: (device: BleDevice) => void): Promise<UnlistenFn> {
  return listen<BleDevice>("ble://connected", (e) => cb(e.payload));
}

export function onDisconnected(cb: (id: string) => void): Promise<UnlistenFn> {
  return listen<string>("ble://disconnected", (e) => cb(e.payload));
}

export function onConnecting(cb: (id: string) => void): Promise<UnlistenFn> {
  return listen<string>("ble://connecting", (e) => cb(e.payload));
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
