import { test } from "bun:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import ts from "typescript";

const source = readFileSync(new URL("../src/lib/storage.ts", import.meta.url), "utf8")
  .replace(/^import .*;\r?\n/gm, "");
const compiled = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 },
}).outputText;
const { createNativeStorage } = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);
const journalKey = "b4s.store.pending.v1";

function legacyStorage(entries = {}) {
  const values = new Map(Object.entries(entries));
  return {
    get length() { return values.size; },
    key: index => [...values.keys()][index] ?? null,
    getItem: key => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, value),
    removeItem: key => values.delete(key),
  };
}
function backend(snapshot) {
  let staged;
  return {
    snapshot, fail: false,
    get: async () => snapshot,
    set: async (_key, value) => { staged = structuredClone(value); },
    async save() {
      if (this.fail) throw new Error("Disk unavailable");
      this.snapshot = structuredClone(staged);
    },
  };
}

test("first migration preserves all owned values and leaves unrelated browser data alone", async () => {
  const entries = { "b4s-theme": "light", "b4s.locale": "vi", "b4s.eq.custom.A.model.layout": "malformed", "other-app": "private" };
  const legacy = legacyStorage(entries);
  const store = backend();
  const storage = await createNativeStorage(store, legacy);
  assert.equal(storage.getItem("b4s-theme"), "light");
  assert.equal(storage.getItem("b4s.eq.custom.A.model.layout"), "malformed");
  assert.equal(storage.getItem("other-app"), null);
  assert.equal(legacy.getItem("other-app"), "private");
  assert.equal(store.snapshot.version, 1);
});

test("native snapshot wins over stale legacy keys and never resurrects deleted settings", async () => {
  const store = backend({ version: 1, values: { "b4s.locale": "es" } });
  const storage = await createNativeStorage(store, legacyStorage({ "b4s.locale": "vi", "b4s.last-device": "stale" }));
  assert.equal(storage.getItem("b4s.locale"), "es");
  assert.equal(storage.getItem("b4s.last-device"), null);
});

test("failed native save retains journal and next launch recovers the latest change", async () => {
  const legacy = legacyStorage();
  const store = backend();
  const storage = await createNativeStorage(store, legacy);
  store.fail = true;
  storage.setItem("b4s.locale", "vi");
  await assert.rejects(storage.flush(), /Disk unavailable/);
  assert.ok(legacy.getItem(journalKey));
  store.fail = false;
  const recovered = await createNativeStorage(backend(store.snapshot), legacy);
  assert.equal(recovered.getItem("b4s.locale"), "vi");
  assert.equal(legacy.getItem(journalKey), null);
});

test("queued edits and deletion persist in order and clear the recovery journal", async () => {
  const legacy = legacyStorage();
  const store = backend();
  const storage = await createNativeStorage(store, legacy);
  storage.setItem("b4s.locale", "vi");
  storage.setItem("b4s.locale", "es");
  storage.setItem("b4s-theme", "dark");
  storage.removeItem("b4s.locale");
  await storage.flush();
  assert.deepEqual(store.snapshot.values, { "b4s-theme": "dark" });
  assert.equal(legacy.getItem(journalKey), null);
});

test("journal write denial rolls back in-memory edits", async () => {
  const legacy = legacyStorage();
  const storage = await createNativeStorage(backend(), legacy);
  legacy.setItem = () => { throw new Error("Storage denied"); };
  assert.throws(() => storage.setItem("b4s.locale", "vi"), /Storage denied/);
  assert.equal(storage.getItem("b4s.locale"), null);
});

test("corrupt native snapshot and migration save failures never replace legacy data", async () => {
  const legacy = legacyStorage({ "b4s.locale": "vi" });
  await assert.rejects(createNativeStorage(backend({ version: 99, values: {} }), legacy), /Invalid settings snapshot/);
  const store = backend(); store.fail = true;
  await assert.rejects(createNativeStorage(store, legacy), /Disk unavailable/);
  assert.equal(legacy.getItem("b4s.locale"), "vi");
  assert.equal(store.snapshot, undefined);
});
