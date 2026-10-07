import { test } from "bun:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import ts from "typescript";

const source = readFileSync(new URL("../src/lib/modelIdMigration.ts", import.meta.url), "utf8")
  .replace(/^import .*;\r?\n/gm, "");
const compiled = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 },
}).outputText;
const { migrateModelIdsOnce } = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);

test("migration upgrades remembered identities and EQ keys even after v1 completed", () => {
  const values = new Map([
    ["b4s.migration.model-id.v1", "complete"],
    ["b4s.last-device", JSON.stringify({ address: "AA:BB", modelId: "server-bowie-ma10" })],
    ["b4s.last-device.v2", JSON.stringify({ version: 2, device: { address: "AA:BB", modelId: "server-bowie-ma10" } })],
    ["b4s.eq.custom.AA:BB.server-bowie-ma10.100.200", "old"],
    ["b4s.eq.custom.AA:BB.bowie-ma10.100.200", "new"],
  ]);
  const storage = {
    get length() { return values.size; },
    key: index => [...values.keys()][index] ?? null,
    getItem: key => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, value),
    removeItem: key => values.delete(key),
  };
  migrateModelIdsOnce(storage);
  assert.equal(JSON.parse(values.get("b4s.last-device")).modelId, "bowie-ma10");
  assert.equal(JSON.parse(values.get("b4s.last-device.v2")).device.modelId, "bowie-ma10");
  assert.equal(values.get("b4s.eq.custom.AA:BB.bowie-ma10.100.200"), "new");
  assert.equal(values.has("b4s.eq.custom.AA:BB.server-bowie-ma10.100.200"), false);
  const snapshot = [...values];
  migrateModelIdsOnce(storage);
  assert.deepEqual([...values], snapshot);
});
