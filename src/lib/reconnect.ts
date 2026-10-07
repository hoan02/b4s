import { storage } from "./storage";
import type { AudioTarget, BleDevice, ScanStatus } from "./ble";

const KEY = "b4s.last-device.v2";
const LEGACY_KEY = "b4s.last-device";
const STORAGE_VERSION = 2 as const;

export interface RememberedDevice {
  id: string;
  address: string;
  name: string;
  modelId?: string | null;
  serial?: string | null;
}

interface RememberedDeviceEnvelope {
  version: typeof STORAGE_VERSION;
  device: RememberedDevice;
}

const addressKey = (address: string) => address.replace(/[:-]/g, "").toLowerCase();
const usableAddress = (address: string) => /^[0-9a-f]{12}$/.test(addressKey(address)) && addressKey(address) !== "000000000000";

function decodeRememberedDevice(value: unknown): RememberedDevice | null {
  if (!value || typeof value !== "object") return null;
  const candidate = value as Record<string, unknown>;
  if (typeof candidate.id !== "string" || !candidate.id || candidate.id.startsWith("mock-")) return null;
  if (typeof candidate.address !== "string" || typeof candidate.name !== "string") return null;
  if (candidate.modelId != null && typeof candidate.modelId !== "string") return null;
  if (candidate.serial != null && typeof candidate.serial !== "string") return null;
  return {
    id: candidate.id,
    address: candidate.address,
    name: candidate.name,
    modelId: candidate.modelId as string | null | undefined,
    serial: candidate.serial as string | null | undefined,
  };
}

function decodeCurrentDevice(raw: string): RememberedDevice | null {
  try {
    const value: unknown = JSON.parse(raw);
    if (typeof value === "object" && value !== null && (value as Record<string, unknown>).version === STORAGE_VERSION) {
      return decodeRememberedDevice((value as Record<string, unknown>).device);
    }
  } catch {
    // Invalid persisted values are recovered to "no remembered device".
  }
  return null;
}

export function rememberDevice(device: BleDevice): void {
  if (device.id.startsWith("mock-") || !["verified", "experimental"].includes(device.support ?? "")) return;
  const envelope: RememberedDeviceEnvelope = {
    version: STORAGE_VERSION,
    device: {
      id: device.id, address: device.address, name: device.modelName || device.name,
      modelId: device.modelId, serial: device.serial,
    },
  };
  try {
    storage.setItem(KEY, JSON.stringify(envelope));
    storage.removeItem(LEGACY_KEY);
  } catch { /* Storage is optional. */ }
}

export function readRememberedDevice(): RememberedDevice | null {
  try {
    const current = storage.getItem(KEY);
    if (current !== null) return decodeCurrentDevice(current);

    // The legacy key is read only when the current key is absent, then removed
    // after a one-time migration. A corrupt current value never revives it.
    const legacy = storage.getItem(LEGACY_KEY);
    if (legacy === null) return null;
    let legacyDevice: RememberedDevice | null = null;
    try {
      legacyDevice = decodeRememberedDevice(JSON.parse(legacy));
    } catch {
      legacyDevice = null;
    }
    if (!legacyDevice) {
      try { storage.removeItem(LEGACY_KEY); } catch { /* Recovery is best effort. */ }
      return null;
    }
    try {
      storage.setItem(KEY, JSON.stringify({ version: STORAGE_VERSION, device: legacyDevice }));
      storage.removeItem(LEGACY_KEY);
    } catch {
      // Keep the migrated in-memory value if storage is temporarily read-only.
    }
    return legacyDevice;
  } catch { return null; }
}

export function matchesRememberedDevice(device: BleDevice, saved: RememberedDevice): boolean {
  if (device.id.startsWith("mock-") || !["verified", "experimental"].includes(device.support ?? "")) return false;
  if (saved.modelId && device.modelId !== saved.modelId) return false;
  // Names identify a product model, not an individual pair of earbuds.
  return device.id === saved.id ||
    (!!saved.serial && device.serial === saved.serial) ||
    (usableAddress(saved.address) && addressKey(device.address) === addressKey(saved.address));
}

interface ReconnectApi {
  checkAdapter: () => Promise<boolean>;
  startScan: () => Promise<void>;
  stopScan: () => Promise<void>;
  getScanStatus: () => Promise<ScanStatus>;
  onScanStatus: (callback: (status: ScanStatus) => void) => Promise<() => void>;
}

/** Prefer the selected audio output before the legacy remembered-device scan. */
export async function findReconnectTarget(
  saved: RememberedDevice,
  api: ReconnectApi & {
    getAudioTarget: () => Promise<AudioTarget | null>;
    prepareAudioTarget: (endpointId: string) => Promise<BleDevice>;
  },
  signal: AbortSignal,
): Promise<{ device: BleDevice; audioEndpointId?: string } | null> {
  if (signal.aborted) return null;
  const audio = await api.getAudioTarget().catch(() => null);
  if (signal.aborted) return null;
  if (audio) {
    const device = await api.prepareAudioTarget(audio.endpointId);
    return signal.aborted ? null : { device, audioEndpointId: audio.endpointId };
  }
  const device = await findRememberedDevice(saved, api, signal);
  return device ? { device } : null;
}

/** One bounded scan for the previous device; releases its listener and scan before returning. */
export async function findRememberedDevice(
  saved: RememberedDevice,
  api: ReconnectApi,
  signal: AbortSignal,
  timeoutMs = 12_000,
): Promise<BleDevice | null> {
  if (signal.aborted || !await api.checkAdapter() || signal.aborted) return null;
  let finish!: (device: BleDevice | null) => void;
  let settled = false;
  const result = new Promise<BleDevice | null>((resolve) => {
    finish = (device) => { settled = true; resolve(device); };
  });
  const cancel = () => finish(null);
  const timer = setTimeout(cancel, timeoutMs);
  signal.addEventListener("abort", cancel, { once: true });
  let unsubscribe: (() => void) | undefined;
  let started = false;
  let generation = -1;
  let revision = -1;
  const inspect = (status: ScanStatus) => {
    if (settled || signal.aborted) return;
    if (status.generation < generation ||
      (status.generation === generation && status.revision < revision)) return;
    generation = status.generation;
    revision = status.revision;
    const match = status.devices.find((device) => matchesRememberedDevice(device, saved));
    if (match) finish(match);
    else if (status.error || (started && !status.scanning)) finish(null);
  };
  try {
    unsubscribe = await api.onScanStatus(inspect);
    if (signal.aborted || settled) return null;
    started = true;
    await api.startScan();
    if (!settled) inspect(await api.getScanStatus());
    return await result;
  } finally {
    clearTimeout(timer);
    signal.removeEventListener("abort", cancel);
    unsubscribe?.();
    if (started) await api.stopScan().catch(() => {});
  }
}
