import { test } from "bun:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import ts from "typescript";

const source = readFileSync(new URL("../src/lib/i18n.ts", import.meta.url), "utf8");
const formatter = source.slice(source.indexOf("export function formatError("), source.indexOf("export async function setLocale("));
const compiled = ts.transpileModule(`const t = key => key;\n${formatter}`, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 },
}).outputText;
const { formatError } = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);

test("experimental rejection uses the translated guidance in the existing error toast", () => {
  const message = "Experimental control is disabled; model evidence must be reviewed first";
  assert.equal(formatError({ contractVersion: 1, code: "deviceCommandFailed", retryable: false, message }),
    "error.operationFailed: error.experimentalControlDisabled");
  assert.equal(formatError(new Error(message)), "error.operationFailed: error.experimentalControlDisabled");
});

test("other errors retain their original details", () => {
  assert.equal(formatError(new Error("Readback timed out")), "error.operationFailed: Readback timed out");
  assert.equal(formatError("Disconnected"), "error.operationFailed: Disconnected");
});
