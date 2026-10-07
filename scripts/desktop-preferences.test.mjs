import { test } from "bun:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import ts from "typescript";

const source = readFileSync(new URL("../src/lib/desktopPreferences.ts", import.meta.url), "utf8").replace('import { storage } from "./storage";', 'const storage = { getItem: key => localStorage.getItem(key), setItem: (key, value) => localStorage.setItem(key, value), removeItem: key => localStorage.removeItem(key) };');
const compiled = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 },
}).outputText;
const preferencesModule = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);
const { readDesktopPreferences, writeAutoReconnect, writeExperimentalMode } = preferencesModule;

const currentKey = "b4s.desktop.preferences.v2";
const legacyKey = "b4s.desktop.preferences.v1";
const defaults = { version: 2, autoReconnect: false, experimentalMode: true };

function installStorage(entries = {}) {
  const values = new Map(Object.entries(entries));
  globalThis.localStorage = {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, String(value)),
    removeItem: (key) => values.delete(key),
  };
  return values;
}

test("new preferences enable experimental mode by default", () => {
  installStorage();
  assert.deepEqual(readDesktopPreferences(), defaults);
});

test("an explicitly saved disabled experimental mode survives reload and unrelated preference edits", () => {
  installStorage({ [currentKey]: JSON.stringify({ version: 2, autoReconnect: false, experimentalMode: false }) });
  assert.equal(readDesktopPreferences().experimentalMode, false);
  assert.equal(writeAutoReconnect(true), true);
  assert.equal(readDesktopPreferences().experimentalMode, false);
  assert.equal(writeExperimentalMode(true), true);
  assert.equal(readDesktopPreferences().experimentalMode, true);
});

test("version 1 preferences migrate once and remove the legacy key", () => {
  const values = installStorage({
    [legacyKey]: JSON.stringify({ version: 1, autoReconnect: true }),
  });

  assert.deepEqual(readDesktopPreferences(), {
    version: 2, autoReconnect: true, experimentalMode: true,
  });
  assert.deepEqual(JSON.parse(values.get(currentKey)), {
    version: 2, autoReconnect: true, experimentalMode: true,
  });
  assert.equal(values.has(legacyKey), false);
});

test("corrupt current preferences recover to defaults without resurrecting legacy state", () => {
  const values = installStorage({
    [currentKey]: JSON.stringify({ version: 2, autoReconnect: "yes", experimentalMode: true }),
    [legacyKey]: JSON.stringify({ version: 1, autoReconnect: true }),
  });

  assert.deepEqual(readDesktopPreferences(), defaults);
  assert.equal(values.get(legacyKey), JSON.stringify({ version: 1, autoReconnect: true }));
});

test("writes recover corrupt values into the current schema", () => {
  const values = installStorage({
    [currentKey]: "not-json",
    [legacyKey]: JSON.stringify({ version: 1, autoReconnect: false }),
  });

  assert.equal(writeAutoReconnect(true), true);
  assert.deepEqual(JSON.parse(values.get(currentKey)), {
    version: 2, autoReconnect: true, experimentalMode: true,
  });
  assert.equal(writeExperimentalMode(true), true);
  assert.deepEqual(readDesktopPreferences(), {
    version: 2, autoReconnect: true, experimentalMode: true,
  });
  assert.equal(values.has(legacyKey), false);
});

test("storage access failures keep startup on defaults and report failed writes", () => {
  globalThis.localStorage = {
    getItem: () => { throw new Error("storage unavailable"); },
    setItem: () => { throw new Error("storage unavailable"); },
    removeItem: () => { throw new Error("storage unavailable"); },
  };

  assert.deepEqual(readDesktopPreferences(), defaults);
  assert.equal(writeAutoReconnect(true), false);
  assert.equal(writeExperimentalMode(true), false);
});
