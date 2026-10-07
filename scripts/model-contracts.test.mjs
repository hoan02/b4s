import { test } from "bun:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import ts from "typescript";

const source = readFileSync(new URL("../src/lib/modelContracts.ts", import.meta.url), "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 },
}).outputText;
const { assertModelContracts, isSoundContract, isEvidenceMap } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);
const profile = id => JSON.parse(readFileSync(new URL(`../src-tauri/catalog/models/${id}.json`, import.meta.url), "utf8"));

test("all promoted profiles satisfy frontend contracts and keep explicit unresolved transport", () => {
  const files = readdirSync(new URL("../src-tauri/catalog/models/", import.meta.url)).filter(name => name.endsWith(".json"));
  assert.equal(files.length, 124);
  for (const file of files) {
    const model = profile(file.slice(0, -5));
    assert.equal(file, `${model.id}.json`);
    assertModelContracts(model);
    assert.ok(!model.id.startsWith("server-"));
    if (model.support === "scanOnly") {
      assert.equal(model.support, "scanOnly");
      assert.equal(model.connection.transport, "unresolved");
      assert.deepEqual(model.capabilities, {});
      assert.ok(model.aliases.length > 0);
    }
  }
});

test("reviewed models supply explicit sound limits, EQ write contract and evidence", () => {
  const pro = profile("bass-bp1-pro");
  const ultra = profile("bass-bp1-ultra");
  assertModelContracts(pro);
  assertModelContracts(ultra);
  assert.equal(pro.sound.maxBassLevel, 1);
  assert.equal(ultra.sound.maxBassLevel, 5);
  assert.deepEqual(pro.eq.customWrite, { slot: 101, ancBank: false });
  assert.equal(ultra.featureEvidence.gestureV2.status, "unknown");
});

test("missing schema, Q values or slot never receive defaults", () => {
  for (const mutate of [p => p.schemaVersion = 2, p => delete p.sound,
    p => p.eq.qValues = [], p => delete p.eq.customWrite,
    p => delete p.featureEvidence.gestureV2]) {
    const pro = profile("bass-bp1-pro");
    mutate(pro);
    assert.throws(() => assertModelContracts(pro));
  }
});

test("unknown runtime has explicit null sound; malformed limits and evidence are rejected", () => {
  assert.equal(isSoundContract(null), true);
  assert.equal(isSoundContract(undefined), false);
  assert.equal(isSoundContract({ maxBassLevel: 0, provenance: "wrong" }), false);
  assert.equal(isEvidenceMap({ typo: { status: "implemented", provenance: "source", firmwareVersions: [] } }), false);
  assert.equal(isEvidenceMap({ anc: { status: "implemented", provenance: "", firmwareVersions: [] } }), false);
});
