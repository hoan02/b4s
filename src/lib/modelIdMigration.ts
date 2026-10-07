import { storage } from "./storage";
/** One-time data migration only; never used to resolve live devices. */
export const LEGACY_MODEL_ID_MAP = {
  "server-aequr-g10": "aequr-g10",
  "server-aequr-gh02": "aequr-gh02",
  "server-airgo-1-ring": "airgo-1-ring",
  "server-airgo-ag20": "airgo-ag20",
  "server-airgo-as01": "airgo-as01",
  "server-airnora": "airnora",
  "server-airnora-2": "airnora-2",
  "server-airnora-3": "airnora-3",
  "server-as01": "as01",
  "server-as01-air": "as01-air",
  "server-baseusencokw04": "baseusencokw04",
  "server-baseusencokw11": "baseusencokw11",
  "server-baseusencokw12": "baseusencokw12",
  "server-bass-1-plus": "bass-1-plus",
  "server-bass-bc1-b0f249a2": "bass-bc1-b0f249a2",
  "server-bass-bc1-ec048a8e": "bass-bc1-ec048a8e",
  "server-bass-bc1-lite": "bass-bc1-lite",
  "server-bass-bc2": "bass-bc2",
  "server-bass-bc2s": "bass-bc2s",
  "server-bass-bd1": "bass-bd1",
  "server-bass-bf1": "bass-bf1",
  "server-bass-bf1-lite": "bass-bf1-lite",
  "server-bass-bh1": "bass-bh1",
  "server-bass-bh1-air": "bass-bh1-air",
  "server-bass-bh1-lite": "bass-bh1-lite",
  "server-bass-bh1-nc": "bass-bh1-nc",
  "server-bass-bp1-nc": "bass-bp1-nc",
  "server-bass-bs1": "bass-bs1",
  "server-bass-bs1-nc": "bass-bs1-nc",
  "server-bass-bs2-lite": "bass-bs2-lite",
  "server-bass-e12x": "bass-e12x",
  "server-bass-e19s": "bass-e19s",
  "server-bass-eh10-nc": "bass-eh10-nc",
  "server-bass-ep10-nc": "bass-ep10-nc",
  "server-bass-ep10-pro": "bass-ep10-pro",
  "server-bass-ep10-ultra": "bass-ep10-ultra",
  "server-bass-w04": "bass-w04",
  "server-bass-wm01s": "bass-wm01s",
  "server-bass-wm02s": "bass-wm02s",
  "server-bh1-nc-lite": "bh1-nc-lite",
  "server-bowie-10-max": "bowie-10-max",
  "server-bowie-30": "bowie-30",
  "server-bowie-30-max": "bowie-30-max",
  "server-bowie-35": "bowie-35",
  "server-bowie-d05": "bowie-d05",
  "server-bowie-e10": "bowie-e10",
  "server-bowie-e12": "bowie-e12",
  "server-bowie-e13": "bowie-e13",
  "server-bowie-e3-2025": "bowie-e3-2025",
  "server-bowie-e5": "bowie-e5",
  "server-bowie-e5x": "bowie-e5x",
  "server-bowie-ez10": "bowie-ez10",
  "server-bowie-h1": "bowie-h1",
  "server-bowie-h1-pro": "bowie-h1-pro",
  "server-bowie-h1i": "bowie-h1i",
  "server-bowie-h1s-be5ff335": "bowie-h1s-be5ff335",
  "server-bowie-h1s-e19e2d01": "bowie-h1s-e19e2d01",
  "server-bowie-h1s-pro": "bowie-h1s-pro",
  "server-bowie-h2": "bowie-h2",
  "server-bowie-m1": "bowie-m1",
  "server-bowie-m2": "bowie-m2",
  "server-bowie-m2-plus": "bowie-m2-plus",
  "server-bowie-m2s": "bowie-m2s",
  "server-bowie-m2s-pro": "bowie-m2s-pro",
  "server-bowie-m3": "bowie-m3",
  "server-bowie-m3s": "bowie-m3s",
  "server-bowie-m4s": "bowie-m4s",
  "server-bowie-ma10": "bowie-ma10",
  "server-bowie-ma10-pro": "bowie-ma10-pro",
  "server-bowie-ma10s": "bowie-ma10s",
  "server-bowie-ma20": "bowie-ma20",
  "server-bowie-ma20-pro": "bowie-ma20-pro",
  "server-bowie-mc1": "bowie-mc1",
  "server-bowie-mc1-pro": "bowie-mc1-pro",
  "server-bowie-mc2": "bowie-mc2",
  "server-bowie-mc2-air": "bowie-mc2-air",
  "server-bowie-mc2-nc": "bowie-mc2-nc",
  "server-bowie-mc2-s-713196ac": "bowie-mc2-s-713196ac",
  "server-bowie-mc2-s-ef41b558": "bowie-mc2-s-ef41b558",
  "server-bowie-mf1": "bowie-mf1",
  "server-bowie-mh1": "bowie-mh1",
  "server-bowie-mp1": "bowie-mp1",
  "server-bowie-ms1": "bowie-ms1",
  "server-bowie-mz10": "bowie-mz10",
  "server-bowie-u2": "bowie-u2",
  "server-bowie-u2-pro": "bowie-u2-pro",
  "server-bowie-w04": "bowie-w04",
  "server-bowie-w04-plus-13faaa6c": "bowie-w04-plus-13faaa6c",
  "server-bowie-w04-plus-2153e749": "bowie-w04-plus-2153e749",
  "server-bowie-w04-pro": "bowie-w04-pro",
  "server-bowie-wm01": "bowie-wm01",
  "server-bowie-wm01plus": "bowie-wm01plus",
  "server-bowie-wm03": "bowie-wm03",
  "server-bowie-wm05": "bowie-wm05",
  "server-bowie-wx5": "bowie-wx5",
  "server-bowiee2": "bowiee2",
  "server-bowiee3": "bowiee3",
  "server-bowiee8": "bowiee8",
  "server-c-mic-cm10": "c-mic-cm10",
  "server-e9": "e9",
  "server-eh10-nc-lite": "eh10-nc-lite",
  "server-eli-10i-fit": "eli-10i-fit",
  "server-eli-15i-fit": "eli-15i-fit",
  "server-eli-1i-fit": "eli-1i-fit",
  "server-eli-fit": "eli-fit",
  "server-eli-sport-1": "eli-sport-1",
  "server-eli-sport2": "eli-sport2",
  "server-ex": "ex",
  "server-inspire-xc1": "inspire-xc1",
  "server-inspire-xh1": "inspire-xh1",
  "server-inspire-xp1": "inspire-xp1",
  "server-m2s-ultra": "m2s-ultra",
  "server-p1": "p1",
  "server-p1-lite": "p1-lite",
  "server-p1x": "p1x",
  "server-storm-1": "storm-1",
  "server-storm-3": "storm-3",
  "server-w04-plus-pro": "w04-plus-pro",
  "server-w04-pro": "w04-pro",
  "server-w05lite": "w05lite",
  "server-wm02": "wm02",
  "server-wm02-plus": "wm02-plus",
  "bass-bc1": "bass-bc1-ec048a8e",
  "bass-bp1-pro": "bass-bp1-pro",
  "bass-bp1-ultra": "bass-bp1-ultra",
  "bass-bp1-nc": "bass-bp1-nc",
  "bass-ep10-nc": "bass-ep10-nc",
  "bass-ep10-pro": "bass-ep10-pro",
  "bass-ep10-ultra": "bass-ep10-ultra",
  "bowie-ma10": "bowie-ma10",
  "bowie-ma10-pro": "bowie-ma10-pro",
  "bowie-ma10s": "bowie-ma10s",
  "bowie-ma20": "bowie-ma20",
  "bowie-ma20-pro": "bowie-ma20-pro",
  "bowie-m1": "bowie-m1",
  "bowie-m2": "bowie-m2",
  "bowie-m2-plus": "bowie-m2-plus",
  "bowie-m2s": "bowie-m2s",
  "bowie-m2s-pro": "bowie-m2s-pro",
  "bowie-m3": "bowie-m3",
  "bowie-m3s": "bowie-m3s",
  "bowie-m4s": "bowie-m4s",
  "m2s-ultra": "m2s-ultra",
  "bass-e12x": "bass-e12x",
  "bass-e19s": "bass-e19s",
  "bowie-e10": "bowie-e10",
  "bowie-e12": "bowie-e12",
  "bowie-e13": "bowie-e13",
  "bowie-e3-2025": "bowie-e3-2025",
  "bowie-e5": "bowie-e5",
  "bowie-e5x": "bowie-e5x",
  "e9": "e9",
  "inspire-xc1": "inspire-xc1",
  "inspire-xh1": "inspire-xh1",
  "inspire-xp1": "inspire-xp1",
  "as01": "as01",
  "as01-air": "as01-air",
  "airgo-1-ring": "airgo-1-ring",
  "airgo-ag20": "airgo-ag20",
  "airgo-as01": "airgo-as01",
  "bowie-mc1": "bowie-mc1",
  "bowie-mc1-pro": "bowie-mc1-pro",
  "bowie-mc2": "bowie-mc2",
  "bowie-mc2-air": "bowie-mc2-air",
  "bowie-mc2-nc": "bowie-mc2-nc",
  "bowie-mc2-s": "bowie-mc2-s-713196ac",
  "bowie-mf1": "bowie-mf1",
  "bass-bc1-lite": "bass-bc1-lite",
  "bass-bc2": "bass-bc2",
  "bass-bd1": "bass-bd1",
  "bass-bf1": "bass-bf1",
  "bass-bf1-lite": "bass-bf1-lite",
  "bass-bs1": "bass-bs1",
  "bass-bs1-nc": "bass-bs1-nc",
  "bass-bs2-lite": "bass-bs2-lite",
  "storm-1": "storm-1",
  "storm-3": "storm-3",
  "bass-w04": "bass-w04",
  "bass-wm01s": "bass-wm01s",
  "bass-wm02s": "bass-wm02s",
  "bowie-ez10": "bowie-ez10",
  "bowie-mz10": "bowie-mz10",
  "bowie-w04": "bowie-w04",
  "bowie-w04-plus": "bowie-w04-plus-13faaa6c",
  "bowie-w04-pro": "bowie-w04-pro",
  "bowie-wm01": "bowie-wm01",
  "bowie-wm03": "bowie-wm03",
  "bowie-wm05": "bowie-wm05",
  "w04-pro": "w04-pro",
  "wm02": "wm02",
  "wm02-plus": "wm02-plus",
  "airnora": "airnora",
  "airnora-2": "airnora-2",
  "airnora-3": "airnora-3",
  "eli-10i-fit": "eli-10i-fit",
  "eli-15i-fit": "eli-15i-fit",
  "eli-1i-fit": "eli-1i-fit",
  "eli-fit": "eli-fit",
  "eli-sport-1": "eli-sport-1",
  "aequr-gh02": "aequr-gh02",
  "bh1-nc-lite": "bh1-nc-lite",
  "bass-bh1": "bass-bh1",
  "bass-bh1-air": "bass-bh1-air",
  "bass-bh1-lite": "bass-bh1-lite",
  "bass-bh1-nc": "bass-bh1-nc",
  "bass-eh10-nc": "bass-eh10-nc",
  "bowie-10-max": "bowie-10-max",
  "bowie-30-max": "bowie-30-max",
  "bowie-d05": "bowie-d05",
  "bowie-h1": "bowie-h1",
  "bowie-h1-pro": "bowie-h1-pro",
  "bowie-h1s": "bowie-h1s-e19e2d01",
  "bowie-h1i": "bowie-h1i",
  "bowie-h1s-pro": "bowie-h1s-pro",
  "bowie-h2": "bowie-h2",
  "bowie-mh1": "bowie-mh1",
  "eh10-nc-lite": "eh10-nc-lite",
  "bowie-u2": "bowie-u2",
  "bowie-u2-pro": "bowie-u2-pro",
  "p1": "p1",
  "p1-lite": "p1-lite",
  "p1x": "p1x",
  "aequr-30-air": "aequr-30-air",
  "aequr-ds10": "aequr-ds10",
  "aequr-g10": "aequr-g10",
  "aequr-n10": "aequr-n10",
  "aequr-vo20": "aequr-vo20",
  "bass-1-plus": "bass-1-plus",
  "bowie-30": "bowie-30",
  "bowie-35": "bowie-35",
  "bowie-mp1": "bowie-mp1",
  "bowie-ms1": "bowie-ms1",
  "ex": "ex",
  "t2-pro": "t2-pro",
} as const satisfies Record<string, string>;

const MIGRATION_KEY = "b4s.migration.model-id.v2";
const CUSTOM_EQ_PREFIX = "b4s.eq.custom.";

export function migrateModelIdsOnce(targetStorage = storage): void {
  const storage = targetStorage;
  try {
    if (storage.getItem(MIGRATION_KEY) === "complete") return;

    const currentRaw = storage.getItem("b4s.last-device.v2");
    if (currentRaw) {
      try {
        const envelope = JSON.parse(currentRaw);
        if (envelope?.version === 2 && typeof envelope.device?.modelId === "string") {
          const modelId = LEGACY_MODEL_ID_MAP[envelope.device.modelId as keyof typeof LEGACY_MODEL_ID_MAP];
          if (modelId && modelId !== envelope.device.modelId) {
            storage.setItem("b4s.last-device.v2", JSON.stringify({ ...envelope, device: { ...envelope.device, modelId } }));
          }
        }
      } catch { /* Preserve malformed remembered data for the reconnect decoder. */ }
    }

    const rememberedRaw = storage.getItem("b4s.last-device");
    if (rememberedRaw) {
      try {
        const remembered: unknown = JSON.parse(rememberedRaw);
        if (typeof remembered === "object" && remembered !== null &&
          "modelId" in remembered && typeof remembered.modelId === "string") {
          const modelId = LEGACY_MODEL_ID_MAP[remembered.modelId as keyof typeof LEGACY_MODEL_ID_MAP];
          if (modelId && modelId !== remembered.modelId) {
            storage.setItem("b4s.last-device", JSON.stringify({ ...remembered, modelId }));
          }
        }
      } catch {
        // Leave malformed data untouched; live resolution never reads its model ID as a rule.
      }
    }

    const keys = Array.from({ length: storage.length }, (_, index) => storage.key(index))
      .filter((key): key is string => key !== null && key.startsWith(CUSTOM_EQ_PREFIX));
    for (const key of keys) {
      const [address, oldModelId, ...layout] = key.slice(CUSTOM_EQ_PREFIX.length).split(".");
      if (!address || !oldModelId || layout.length === 0) continue;
      const modelId = LEGACY_MODEL_ID_MAP[oldModelId as keyof typeof LEGACY_MODEL_ID_MAP];
      if (!modelId || modelId === oldModelId) continue;
      const target = `${CUSTOM_EQ_PREFIX}${address}.${modelId}.${layout.join(".")}`;
      const raw = storage.getItem(key);
      if (raw === null) continue;
      if (storage.getItem(target) === null) storage.setItem(target, raw);
      storage.removeItem(key);
    }

    storage.setItem(MIGRATION_KEY, "complete");
  } catch {
    // Storage denial leaves the migration retryable on the next launch.
  }
}
