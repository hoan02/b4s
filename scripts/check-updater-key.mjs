import { readFileSync, mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";

// Sign a disposable payload so CI catches a mismatched updater key before release.
const require = createRequire(import.meta.url);
const config = JSON.parse(readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url)));
const key = process.env.TAURI_SIGNING_PRIVATE_KEY;
if (!key) throw new Error("TAURI_SIGNING_PRIVATE_KEY is required.");
const directory = mkdtempSync(join(tmpdir(), "b4s-updater-"));
try {
  const payload = join(directory, "check.txt");
  writeFileSync(payload, "B4S updater key check\n");
  const result = spawnSync(process.execPath, [
    join(require.resolve("@tauri-apps/cli/package.json"), "..", "tauri.js"),
    "signer", "sign", "--password", process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD || "", payload,
  ], { encoding: "utf8", timeout: 30000, input: "\n" });
  // CLI output is deliberately suppressed to avoid exposing signing material.
  if (result.error || result.status !== 0) throw new Error("Updater signing check failed; check the private key and password.");
  const decode = (encoded) => Buffer.from(encoded.trim(), "base64").toString("utf8");
  const publicPacket = Buffer.from(decode(config.plugins.updater.pubkey).split(/\r?\n/)[1], "base64");
  const signaturePacket = Buffer.from(decode(readFileSync(`${payload}.sig`, "utf8")).split(/\r?\n/)[1], "base64");
  if (publicPacket.length !== 42 || signaturePacket.length !== 74 ||
      !publicPacket.subarray(2, 10).equals(signaturePacket.subarray(2, 10))) {
    throw new Error("Updater signing key does not match plugins.updater.pubkey. Release aborted.");
  }
  console.log("Updater signing key matches the configured public key.");
} finally {
  rmSync(directory, { recursive: true, force: true });
}
