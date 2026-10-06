import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import ts from "typescript";

// Exercise the hardware-independent controller without a browser or BLE adapter.
const source = readFileSync(new URL("../src/lib/reconnect.ts", import.meta.url), "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 },
}).outputText;
const { rememberDevice, readRememberedDevice, matchesRememberedDevice, findRememberedDevice } =
  await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);

const saved = { id: "old-id", address: "AA:BB:CC:DD:EE:FF", name: "My earbuds", modelId: "bass-bp1-pro" };
const device = { ...saved, id: "new-id", support: "verified", isBaseus: true, connected: false, rssi: -50 };

function scanner(devices = [], overrides = {}) {
  let listener;
  let stops = 0;
  let unsubscribes = 0;
  let starts = 0;
  const api = {
    checkAdapter: async () => true,
    startScan: async () => { starts++; },
    stopScan: async () => { stops++; },
    getScanStatus: async () => ({ scanning: true, devices }),
    onScanStatus: async (callback) => {
      listener = callback;
      return () => { unsubscribes++; listener = undefined; };
    },
    ...overrides,
  };
  return { api, emit: (status) => listener?.(status), counts: () => ({ starts, stops, unsubscribes }) };
}

test("remember real supported earbuds, ignore Demo, reject malformed storage", () => {
  let value = null;
  globalThis.localStorage = { getItem: () => value, setItem: (_key, next) => { value = next; } };
  rememberDevice(device);
  assert.equal(readRememberedDevice().id, "new-id");
  rememberDevice({ ...device, id: "mock-bp1" });
  assert.equal(readRememberedDevice().id, "new-id");
  rememberDevice({ ...device, id: "unsupported", support: "scanOnly" });
  assert.equal(readRememberedDevice().id, "new-id");
  value = '{"id":3}';
  assert.equal(readRememberedDevice(), null);
  value = "invalid json";
  assert.equal(readRememberedDevice(), null);
});

test("match changed Windows IDs by address, never another unit with the same name", () => {
  assert.equal(matchesRememberedDevice(device, saved), true);
  assert.equal(matchesRememberedDevice({ ...device, address: "11:22:33:44:55:66" }, saved), false);
  assert.equal(matchesRememberedDevice({ ...device, modelId: "other-model" }, saved), false);
  assert.equal(matchesRememberedDevice({ ...device, support: "scanOnly" }, saved), false);
  assert.equal(matchesRememberedDevice({ ...device, address: "00:00:00:00:00:00" }, { ...saved, address: "00:00:00:00:00:00" }), false);
});

test("recover a matching device from the snapshot even if its discovery event was missed", async () => {
  const scan = scanner([{ ...device, address: "11:22:33:44:55:66" }, device]);
  assert.deepEqual(await findRememberedDevice(saved, scan.api, new AbortController().signal), device);
  assert.deepEqual(scan.counts(), { starts: 1, stops: 1, unsubscribes: 1 });
});

test("a bounded scan falls back when only unrelated same-name earbuds are nearby", async () => {
  const scan = scanner([{ ...device, address: "11:22:33:44:55:66" }]);
  assert.equal(await findRememberedDevice(saved, scan.api, new AbortController().signal, 10), null);
  assert.deepEqual(scan.counts(), { starts: 1, stops: 1, unsubscribes: 1 });
});

test("cancellation stops scanning and releases the discovery listener", async () => {
  const abort = new AbortController();
  const scan = scanner([], { getScanStatus: async () => {
    abort.abort();
    return { scanning: true, devices: [device] };
  } });
  assert.equal(await findRememberedDevice(saved, scan.api, abort.signal), null);
  assert.deepEqual(scan.counts(), { starts: 1, stops: 1, unsubscribes: 1 });
});

test("cancellation during listener registration never starts a scan", async () => {
  const abort = new AbortController();
  let released = false;
  const scan = scanner([], { onScanStatus: async () => {
    abort.abort();
    return () => { released = true; };
  } });
  assert.equal(await findRememberedDevice(saved, scan.api, abort.signal), null);
  assert.equal(released, true);
  assert.equal(scan.counts().starts, 0);
});

test("Bluetooth off does not start scanning", async () => {
  const scan = scanner([], { checkAdapter: async () => false });
  assert.equal(await findRememberedDevice(saved, scan.api, new AbortController().signal), null);
  assert.deepEqual(scan.counts(), { starts: 0, stops: 0, unsubscribes: 0 });
});

test("connect candidate is found from a later discovery event", async () => {
  const scan = scanner([], { getScanStatus: async () => {
    queueMicrotask(() => scan.emit({ scanning: true, devices: [device] }));
    return { scanning: true, devices: [] };
  } });
  assert.deepEqual(await findRememberedDevice(saved, scan.api, new AbortController().signal), device);
  assert.deepEqual(scan.counts(), { starts: 1, stops: 1, unsubscribes: 1 });
});

test("an external scan stop returns to manual selection", async () => {
  const scan = scanner([], { getScanStatus: async () => ({ scanning: false, devices: [] }) });
  assert.equal(await findRememberedDevice(saved, scan.api, new AbortController().signal), null);
  assert.deepEqual(scan.counts(), { starts: 1, stops: 1, unsubscribes: 1 });
});

test("scan startup errors still release the listener and scan", async () => {
  const scan = scanner([], { startScan: async () => { throw new Error("radio unavailable"); } });
  await assert.rejects(findRememberedDevice(saved, scan.api, new AbortController().signal), /radio unavailable/);
  assert.equal(scan.counts().stops, 1);
  assert.equal(scan.counts().unsubscribes, 1);
});

function keyedStorage(initial) {
  const map = new Map(Object.entries(initial));
  return {
    getItem: (key) => (map.has(key) ? map.get(key) : null),
    setItem: (key, value) => { map.set(key, value); },
    removeItem: (key) => { map.delete(key); },
    snapshot: () => Object.fromEntries(map),
  };
}

test("remembered device migrates the legacy record once into a versioned envelope", () => {
  const storage = keyedStorage({ "b4s.last-device": JSON.stringify(saved) });
  globalThis.localStorage = storage;
  assert.equal(readRememberedDevice().id, "old-id");
  const after = storage.snapshot();
  assert.equal(after["b4s.last-device"], undefined);
  assert.equal(JSON.parse(after["b4s.last-device.v2"]).version, 2);
  assert.equal(JSON.parse(after["b4s.last-device.v2"]).device.id, "old-id");
});

test("a corrupt current record never revives or deletes a stale legacy record", () => {
  const storage = keyedStorage({
    "b4s.last-device.v2": "{not json",
    "b4s.last-device": JSON.stringify(saved),
  });
  globalThis.localStorage = storage;
  assert.equal(readRememberedDevice(), null);
  const after = storage.snapshot();
  assert.equal(after["b4s.last-device.v2"], "{not json");
  assert.ok(after["b4s.last-device"]);
});

test("malformed legacy data is discarded instead of retained", () => {
  const storage = keyedStorage({ "b4s.last-device": '{"id":3}' });
  globalThis.localStorage = storage;
  assert.equal(readRememberedDevice(), null);
  assert.equal(storage.snapshot()["b4s.last-device"], undefined);
});

test("rememberDevice writes a versioned envelope and clears the legacy key", () => {
  const storage = keyedStorage({ "b4s.last-device": JSON.stringify(saved) });
  globalThis.localStorage = storage;
  rememberDevice(device);
  const after = storage.snapshot();
  assert.equal(after["b4s.last-device"], undefined);
  assert.equal(JSON.parse(after["b4s.last-device.v2"]).device.id, "new-id");
  assert.equal(JSON.parse(after["b4s.last-device.v2"]).version, 2);
});
