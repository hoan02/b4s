import { test } from "bun:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import ts from "typescript";

// Isolate forwarding logic from native IPC so failures can be tested without a webview.
const source = readFileSync(new URL("../src/lib/logging.ts", import.meta.url), "utf8")
  .replace(/^import .*;\r?\n/gm, "");
const compiled = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 },
}).outputText;
const { forwardConsole } = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);

test("forwarding retains browser output and error stacks while handling circular objects", async () => {
  const browser = [];
  const native = [];
  const target = { error: (...values) => browser.push(values) };
  const circular = {}; circular.self = circular;
  const failure = new Error("BLE disconnected");
  forwardConsole(target, { error: async message => { native.push(message); } });
  target.error("connect", failure, circular);
  assert.deepEqual(browser[0], ["connect", failure, circular]);
  assert.ok(native[0].includes(failure.stack));
  assert.ok(native[0].includes("[object Object]"));
});

test("native logging rejection does not recurse or prevent browser diagnostics", async () => {
  let calls = 0;
  const browser = [];
  const target = { warn: (...values) => browser.push(values) };
  forwardConsole(target, { warn: async () => { calls++; throw new Error("IPC unavailable"); } });
  target.warn("adapter unavailable");
  await new Promise(resolve => setTimeout(resolve, 0));
  assert.equal(calls, 1);
  assert.deepEqual(browser, [["adapter unavailable"]]);
});
