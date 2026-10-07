import { isTauri, invoke } from "./tauri";

interface Snapshot { version: 1; values: Record<string, string> }
interface Backend {
  get<T>(key: string): Promise<T | undefined>;
  set(key: string, value: unknown): Promise<void>;
  save(): Promise<void>;
}
const JOURNAL = "b4s.store.pending.v1";
const owns = (key: string) => key === "b4s-theme" || (key.startsWith("b4s.") && key !== JOURNAL);
function decode(value: unknown): Snapshot {
  if (typeof value !== "object" || value === null) throw new Error("Invalid settings snapshot");
  const snapshot = value as Snapshot;
  if (snapshot.version !== 1 || !snapshot.values || typeof snapshot.values !== "object" || Array.isArray(snapshot.values) ||
      !Object.entries(snapshot.values).every(([key, value]) => owns(key) && typeof value === "string")) {
    throw new Error("Invalid settings snapshot");
  }
  return snapshot;
}

/** Hydrate once; retain a local recovery journal until each native save succeeds. */
export async function createNativeStorage(backend: Backend, legacy: Storage) {
  let snapshot = await backend.get<Snapshot>("snapshot");
  if (snapshot !== undefined) snapshot = decode(snapshot);
  const pending = legacy.getItem(JOURNAL);
  if (pending !== null) snapshot = decode(JSON.parse(pending));
  if (!snapshot) {
    const values: Record<string, string> = {};
    for (let index = 0; index < legacy.length; index++) {
      const key = legacy.key(index);
      if (!key || !owns(key)) continue;
      const value = legacy.getItem(key);
      if (value !== null) values[key] = value;
    }
    snapshot = { version: 1, values };
  }
  const values = new Map(Object.entries(snapshot.values));
  // Save before rendering or marking migration complete; legacy data remains a backup.
  await backend.set("snapshot", snapshot);
  await backend.save();
  if (pending !== null) legacy.removeItem(JOURNAL);
  let queue = Promise.resolve();
  let failure: unknown;
  function persist() {
    const next: Snapshot = { version: 1, values: Object.fromEntries(values) };
    const journal = JSON.stringify(next);
    // A synchronous journal write must succeed before callers report success.
    legacy.setItem(JOURNAL, journal);
    queue = queue.then(async () => {
      await backend.set("snapshot", next);
      await backend.save();
      failure = undefined;
      if (legacy.getItem(JOURNAL) === journal) legacy.removeItem(JOURNAL);
    }).catch(error => { failure = error; console.error("Settings save failed; recovery journal retained", error); });
  }
  return {
    get length() { return values.size; },
    key: (index: number) => Array.from(values.keys())[index] ?? null,
    getItem: (key: string) => values.get(key) ?? null,
    setItem(key: string, value: string) {
      if (!owns(key)) throw new Error("Unsupported settings key");
      const previous = values.get(key);
      values.set(key, value);
      try { persist(); } catch (error) {
        if (previous === undefined) values.delete(key); else values.set(key, previous);
        throw error;
      }
    },
    removeItem(key: string) {
      const previous = values.get(key);
      values.delete(key);
      try { persist(); } catch (error) {
        if (previous !== undefined) values.set(key, previous);
        throw error;
      }
    },
    async flush() { await queue; if (failure) throw failure; },
  };
}

let native: Awaited<ReturnType<typeof createNativeStorage>> | undefined;
export async function initializeStorage() {
  if (!isTauri()) return;
  await invoke("validate_settings_store");
  const { load } = await import("@tauri-apps/plugin-store");
  const backend = await load("settings.json", { autoSave: false, defaults: {} });
  native = await createNativeStorage(backend, localStorage);
}

// Preserve synchronous callers and existing validation/migrations after hydration.
export const storage = {
  get length() { return (native ?? localStorage).length; },
  key: (index: number) => (native ?? localStorage).key(index),
  getItem: (key: string) => (native ?? localStorage).getItem(key),
  setItem: (key: string, value: string) => (native ?? localStorage).setItem(key, value),
  removeItem: (key: string) => (native ?? localStorage).removeItem(key),
};
