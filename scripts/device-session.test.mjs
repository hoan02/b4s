import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import ts from "typescript";

const source = readFileSync(new URL("../src/stores/deviceSession.ts", import.meta.url), "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 },
}).outputText;
const { createDeviceSession } = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);

function snapshot(deviceId, sessionId, revision) {
  return { schemaVersion: 1, deviceId, sessionId, revision,
    battery: { left: { percentage: 0, charging: false, observedAtMs: revision }, right: null, case: null } };
}

test("old session, duplicate revision and another device cannot overwrite confirmed state", () => {
  const seen = [];
  const session = createDeviceSession(value => seen.push(value));
  session.selectDevice("A");
  session.accept(snapshot("A", 10, 2));
  session.accept(snapshot("A", 10, 1));
  session.accept(snapshot("A", 10, 2));
  session.accept(snapshot("B", 11, 1));
  session.accept(snapshot("A", 9, 9));
  assert.equal(seen.length, 2);
  assert.equal(seen.at(-1).battery.left.percentage, 0);
  session.accept(snapshot("A", 10, 3));
  assert.equal(seen.at(-1).battery.left.observedAtMs, 3);
});

test("same-device reconnect invalidates pending results and old snapshots", () => {
  const seen = [];
  const session = createDeviceSession(value => seen.push(value));
  session.selectDevice("A");
  session.accept(snapshot("A", 10, 1));
  const operation = session.capture();
  session.selectDevice(null);
  assert.equal(session.isCurrent(operation), false);
  session.accept(snapshot("A", 11, 1));
  assert.equal(seen.at(-1), null);
  session.selectDevice("A");
  session.accept(snapshot("A", 10, 3));
  assert.equal(seen.at(-1), null);
  session.accept(snapshot("A", 12, 1));
  assert.equal(seen.at(-1).sessionId, 12);
  assert.equal(session.isCurrent(operation), false);
});

test("switching devices clears snapshot and rejects unknown schema", () => {
  const seen = [];
  const session = createDeviceSession(value => seen.push(value));
  session.selectDevice("A");
  session.accept(snapshot("A", 1, 1));
  session.selectDevice("B");
  assert.equal(seen.at(-1), null);
  session.accept({ ...snapshot("B", 2, 1), schemaVersion: 2 });
  assert.equal(seen.at(-1), null);
  session.accept(snapshot("B", 2, 1));
  assert.equal(seen.at(-1).deviceId, "B");
});

test("backend session rollover invalidates pending work even without connection callback", () => {
  const session = createDeviceSession(() => {});
  session.selectDevice("A");
  session.accept(snapshot("A", 1, 1));
  const operation = session.capture();
  session.accept(snapshot("A", 2, 1));
  assert.equal(session.isCurrent(operation), false);
});


const selectionSource = readFileSync(new URL("../src/features/equalizer/selection.ts", import.meta.url), "utf8");
const selectionCompiled = ts.transpileModule(selectionSource, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 },
}).outputText;
const { resolveEqSelection } = await import(`data:text/javascript;base64,${Buffer.from(selectionCompiled).toString("base64")}`);

test("EQ readback resolves custom, model presets and unknown without inventing Classic", () => {
  const presets = [{ id: "classic", dictSort: 0 }, { id: "acoustic", dictSort: 10 }];
  assert.deepEqual(resolveEqSelection(null, presets, true), { kind: "unknown", wireIndex: null });
  assert.deepEqual(resolveEqSelection(101, presets, true), { kind: "custom", wireIndex: 101 });
  assert.deepEqual(resolveEqSelection(101, presets, false), { kind: "unknown", wireIndex: 101 });
  assert.deepEqual(resolveEqSelection(10, presets, true), { kind: "preset", wireIndex: 10, id: "acoustic" });
  assert.deepEqual(resolveEqSelection(11, presets, true), { kind: "unknown", wireIndex: 11 });
});


const controllerSource = readFileSync(new URL("../src/features/shared/confirmedOperation.ts", import.meta.url), "utf8");
const controllerCompiled = ts.transpileModule(controllerSource, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 },
}).outputText;
const { createConfirmedOperation } = await import(`data:text/javascript;base64,${Buffer.from(controllerCompiled).toString("base64")}`);

function deferred() {
  let resolve;
  const promise = new Promise(done => { resolve = done; });
  return { promise, resolve };
}

test("EQ controller rejects duplicate writes and session changes during refresh", async () => {
  let generation = 1;
  let writes = 0;
  let confirmed = 0;
  const refresh = deferred();
  const controller = createConfirmedOperation({
    session: { capture: () => generation, isCurrent: token => token === generation },
    refresh: () => refresh.promise, pending: () => {}, error: () => {}, formatError: String,
  });
  const operation = controller.run(async () => { writes++; }, () => confirmed++, () => {});
  await controller.run(async () => { writes++; }, () => confirmed++, () => {});
  await Promise.resolve();
  generation++;
  refresh.resolve();
  await operation;
  assert.equal(writes, 1);
  assert.equal(confirmed, 0);
});

test("EQ reset permits new operation and old completion cannot clear its pending state", async () => {
  let pending = false;
  let confirmed = 0;
  const oldWrite = deferred();
  const newWrite = deferred();
  const controller = createConfirmedOperation({
    session: { capture: () => 1, isCurrent: () => true }, refresh: async () => {},
    pending: value => { pending = value; }, error: () => {}, formatError: String,
  });
  const oldOperation = controller.run(() => oldWrite.promise, () => confirmed++, () => {});
  controller.reset();
  const newOperation = controller.run(() => newWrite.promise, () => confirmed++, () => {});
  oldWrite.resolve();
  await oldOperation;
  assert.equal(pending, true);
  assert.equal(confirmed, 0);
  newWrite.resolve();
  await newOperation;
  assert.equal(pending, false);
  assert.equal(confirmed, 1);
});
