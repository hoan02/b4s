#!/usr/bin/env node
/**
 * Bump app version across package.json, Cargo.toml, tauri.conf.json
 *
 * Usage:
 *   npm run version:bump           # patch 0.1.0 → 0.1.1
 *   npm run version:bump -- minor  # 0.1.0 → 0.2.0
 *   npm run version:bump -- major  # 0.1.0 → 1.0.0
 *   npm run version:bump -- 1.2.3  # set exact
 *
 * Then release:
 *   git commit -am "chore: release vX.Y.Z"
 *   git tag vX.Y.Z
 *   git push && git push --tags
 *   → GitHub Actions builds Win/Mac/Linux + latest.json for auto-update
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const arg = process.argv[2] || "patch";

function readJson(p) {
  return JSON.parse(fs.readFileSync(p, "utf8"));
}
function writeJson(p, obj) {
  fs.writeFileSync(p, JSON.stringify(obj, null, 2) + "\n");
}

function parseVer(v) {
  const m = String(v).replace(/^v/, "").match(/^(\d+)\.(\d+)\.(\d+)/);
  if (!m) throw new Error(`Bad version: ${v}`);
  return [Number(m[1]), Number(m[2]), Number(m[3])];
}

function bump(ver, kind) {
  if (/^\d+\.\d+\.\d+/.test(kind)) return kind.replace(/^v/, "");
  const [a, b, c] = parseVer(ver);
  if (kind === "major") return `${a + 1}.0.0`;
  if (kind === "minor") return `${a}.${b + 1}.0`;
  return `${a}.${b}.${c + 1}`;
}

const pkgPath = path.join(root, "package.json");
const tauriPath = path.join(root, "src-tauri", "tauri.conf.json");
const cargoPath = path.join(root, "src-tauri", "Cargo.toml");

const pkg = readJson(pkgPath);
const next = bump(pkg.version, arg);

pkg.version = next;
writeJson(pkgPath, pkg);

const tauri = readJson(tauriPath);
tauri.version = next;
writeJson(tauriPath, tauri);

let cargo = fs.readFileSync(cargoPath, "utf8");
cargo = cargo.replace(
  /^version\s*=\s*"[^"]+"/m,
  `version = "${next}"`
);
fs.writeFileSync(cargoPath, cargo);

console.log(`Version → ${next}`);
console.log(`
Next steps:
  git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
  git commit -m "chore: release v${next}"
  git tag v${next}
  git push origin main --tags
`);
