import { test } from "bun:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import ts from "typescript";

const source = readFileSync(new URL("../src/lib/deviceImages.ts", import.meta.url), "utf8")
  .replace(/^import .*;\r?\n/gm, "");
const compiled = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 },
}).outputText;
const mocks = `const defaultEar = "placeholder"; const isTauri = () => true;
const createSignal = initial => { let value = initial; return [() => value, next => { value = next; }]; };
const invoke = (...args) => globalThis.imageInvoke(...args);
const convertFileSrc = path => "asset:" + path;\n`;
const { loadDeviceImage } = await import(`data:text/javascript;base64,${Buffer.from(mocks + compiled).toString("base64")}`);

test("concurrent image loads and subsequent mounts share one cache request", async () => {
  let calls = 0;
  globalThis.imageInvoke = async (command, args) => {
    calls++;
    assert.equal(command, "cache_product_image");
    assert.equal(args.url, "https://cdn.test/product.png");
    return "/cache/product.png";
  };
  assert.deepEqual(await Promise.all([
    loadDeviceImage("https://cdn.test/product.png"), loadDeviceImage("https://cdn.test/product.png"),
  ]), ["asset:/cache/product.png", "asset:/cache/product.png"]);
  assert.equal(await loadDeviceImage("https://cdn.test/product.png"), "asset:/cache/product.png");
  assert.equal(calls, 1);
});

test("failed image requests retain placeholder and back off on repeated renders", async () => {
  let calls = 0;
  globalThis.imageInvoke = async () => { calls++; throw new Error("offline"); };
  assert.equal(await loadDeviceImage("https://cdn.test/offline.png"), "placeholder");
  assert.equal(await loadDeviceImage("https://cdn.test/offline.png"), "placeholder");
  assert.equal(calls, 1);
});
