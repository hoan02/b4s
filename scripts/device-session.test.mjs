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
