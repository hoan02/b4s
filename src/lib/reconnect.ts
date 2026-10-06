import type { BleDevice, ScanStatus } from "./ble";

const KEY = "b4s.last-device";

export interface RememberedDevice {
  id: string;
  address: string;
  name: string;
  modelId?: string | null;
  serial?: string | null;
}

const addressKey = (address: string) => address.replace(/[:-]/g, "").toLowerCase();
const usableAddress = (address: string) => /^[0-9a-f]{12}$/.test(addressKey(address)) && addressKey(address) !== "000000000000";

export function rememberDevice(device: BleDevice): void {
  if (device.id.startsWith("mock-") || !["verified", "experimental"].includes(device.support ?? "")) return;
  const saved: RememberedDevice = {
    id: device.id, address: device.address, name: device.modelName || device.name,
    modelId: device.modelId, serial: device.serial,
  };
  try { localStorage.setItem(KEY, JSON.stringify(saved)); } catch { /* Storage is optional. */ }
}

export function readRememberedDevice(): RememberedDevice | null {
  try {
    const value = JSON.parse(localStorage.getItem(KEY) ?? "null");
    if (!value || typeof value.id !== "string" || !value.id || value.id.startsWith("mock-") ||
      typeof value.address !== "string" || typeof value.name !== "string" ||
      (value.modelId != null && typeof value.modelId !== "string") ||
      (value.serial != null && typeof value.serial !== "string")) return null;
    return value;
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
