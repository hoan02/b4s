#!/usr/bin/env node
/**
 * One-shot release helper for B4S.
 *
 * Bumps version → commit → tag vX.Y.Z → push (triggers GitHub Actions "Release").
 *
 * Usage:
 *   npm run release                 # patch (0.1.0 → 0.1.1)
 *   npm run release -- minor
 *   npm run release -- major
 *   npm run release -- 0.2.0
 *   npm run release -- patch --dry-run
 *   npm run release -- --no-bump    # tag current package.json version only
 *   npm run release -- patch --no-push
 *
 * Flags:
 *   --dry-run   Print steps, do not write / commit / push
 *   --no-bump   Skip version bump (release current version)
 *   --no-push   Commit + tag locally, do not push
 *   --force     Allow release with a dirty working tree (not recommended)
 */

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const bumpScript = path.join(root, "scripts", "bump-version.mjs");

const VERSION_FILES = [
  "package.json",
  "src-tauri/tauri.conf.json",
  "src-tauri/Cargo.toml",
];

// ── args ───────────────────────────────────────────────────────────────────

const raw = process.argv.slice(2);
const flags = new Set(raw.filter((a) => a.startsWith("--")));
const positionals = raw.filter((a) => !a.startsWith("--"));

const dryRun = flags.has("--dry-run");
const noBump = flags.has("--no-bump");
const noPush = flags.has("--no-push");
const force = flags.has("--force");

const bumpKind = positionals[0] || "patch"; // patch | minor | major | X.Y.Z

if (positionals.length > 1) {
  die(`Unexpected args: ${positionals.slice(1).join(" ")}\n${usage()}`);
}

// ── helpers ────────────────────────────────────────────────────────────────

function usage() {
  return `
Usage:
  npm run release [-- patch|minor|major|X.Y.Z] [--dry-run] [--no-bump] [--no-push] [--force]
`.trim();
}

function die(msg) {
  console.error(`\n✗ ${msg}\n`);
  process.exit(1);
}

function log(msg) {
  console.log(msg);
}

function step(title) {
  console.log(`\n→ ${title}`);
}

function run(cmd, args, opts = {}) {
  const mutate = opts.mutate === true;
  if (dryRun && mutate) {
    log(`  [dry-run] ${cmd} ${args.join(" ")}`);
    return "";
  }
  return execFileSync(cmd, args, {
    cwd: root,
    encoding: "utf8",
    stdio: opts.stdio ?? ["ignore", "pipe", "pipe"],
  });
}

/** Read-only git (always executes). */
function git(args, opts = {}) {
  return run("git", args, { ...opts, mutate: false });
}

/** Mutating git (skipped in --dry-run). */
function gitWrite(args, opts = {}) {
  return run("git", args, { ...opts, mutate: true });
}

function readPkgVersion() {
  const pkg = JSON.parse(
    fs.readFileSync(path.join(root, "package.json"), "utf8")
  );
  return pkg.version;
}

function gitOk() {
  try {
    git(["rev-parse", "--is-inside-work-tree"]);
    return true;
  } catch {
    return false;
  }
}

function dirtyFiles() {
  const out = git(["status", "--porcelain"]).trim();
  if (!out) return [];
  return out.split("\n").map((l) => l.trim());
}

function currentBranch() {
  return git(["rev-parse", "--abbrev-ref", "HEAD"]).trim();
}

function remoteExists(name = "origin") {
  try {
    git(["remote", "get-url", name]);
    return true;
  } catch {
    return false;
  }
}

function tagExists(tag) {
  try {
    git(["rev-parse", "--verify", `refs/tags/${tag}`]);
    return true;
  } catch {
    return false;
  }
}

function remoteTagExists(tag) {
  try {
    const out = git(["ls-remote", "--tags", "origin", `refs/tags/${tag}`]).trim();
    return out.length > 0;
  } catch {
    return false;
  }
}

// ── main ───────────────────────────────────────────────────────────────────

console.log("B4S release");
if (dryRun) console.log("(dry-run — no changes will be made)");

if (!gitOk()) die("Not a git repository.");

const branch = currentBranch();
if (branch !== "main" && branch !== "master") {
  console.warn(
    `\n⚠  You are on branch "${branch}", not main/master.\n   Release workflow expects tags on the default branch.`
  );
  if (!force) {
    die('Switch to main, or re-run with --force if intentional.');
  }
}

step("Check working tree");
const dirty = dirtyFiles();
if (dirty.length && !force) {
  console.error("  Working tree is not clean:");
  for (const line of dirty) console.error(`    ${line}`);
  die("Commit or stash changes first, or pass --force.");
}
if (dirty.length && force) {
  log("  ⚠ Dirty tree allowed via --force");
} else {
  log("  clean");
}

const prevVersion = readPkgVersion();
let nextVersion = prevVersion;

if (!noBump) {
  step(`Bump version (${bumpKind})`);
  if (dryRun) {
    // Approximate next version for display (no file writes)
    try {
      const [a, b, c] = prevVersion
        .match(/^(\d+)\.(\d+)\.(\d+)/)
        .slice(1)
        .map(Number);
      if (/^\d+\.\d+\.\d+/.test(bumpKind)) {
        nextVersion = bumpKind.replace(/^v/, "");
      } else if (bumpKind === "major") nextVersion = `${a + 1}.0.0`;
      else if (bumpKind === "minor") nextVersion = `${a}.${b + 1}.0`;
      else nextVersion = `${a}.${b}.${c + 1}`;
      log(`  [dry-run] ${prevVersion} → ${nextVersion}`);
      log(`  [dry-run] node scripts/bump-version.mjs ${bumpKind}`);
    } catch {
      log(`  [dry-run] node scripts/bump-version.mjs ${bumpKind}`);
    }
  } else {
    run("node", [bumpScript, bumpKind], { stdio: "inherit", mutate: true });
    nextVersion = readPkgVersion();
  }
} else {
  step("Skip bump (--no-bump)");
  log(`  current version: ${prevVersion}`);
  nextVersion = prevVersion;
}

const tag = `v${nextVersion}`;

step(`Validate tag ${tag}`);
if (tagExists(tag)) {
  die(`Local tag ${tag} already exists. Delete it or choose another version.`);
}
if (remoteExists() && remoteTagExists(tag)) {
  die(`Remote tag ${tag} already exists on origin.`);
}
log(`  ${tag} is free`);

step("Commit version files");
const commitMsg = `chore: release ${tag}`;
if (noBump && !dirty.length) {
  log("  nothing to commit (no bump); will tag HEAD");
} else {
  gitWrite(["add", ...VERSION_FILES]);
  if (dryRun) {
    log(`  [dry-run] git commit -m "${commitMsg}"`);
  } else {
    const staged = git(["diff", "--cached", "--name-only"]).trim();
    if (staged) {
      gitWrite(["commit", "-m", commitMsg], { stdio: "inherit" });
    } else if (noBump) {
      log("  no staged version changes; tagging current HEAD");
    } else {
      die("Version files did not change; aborting.");
    }
  }
}

step(`Create tag ${tag}`);
gitWrite(["tag", "-a", tag, "-m", `Release ${tag}`], { stdio: "inherit" });

if (noPush) {
  step("Skip push (--no-push)");
  log(`  Local tag ${tag} created. Push when ready:`);
  log(`    git push origin HEAD --tags`);
} else {
  step("Push branch + tags to origin");
  if (!remoteExists()) {
    die('Remote "origin" not found.');
  }
  // Push current branch + tag (tag push triggers Release workflow on v*)
  gitWrite(["push", "origin", `HEAD:refs/heads/${branch}`], {
    stdio: "inherit",
  });
  gitWrite(["push", "origin", tag], { stdio: "inherit" });
}

// ── summary ────────────────────────────────────────────────────────────────

console.log(`
✓ Release prepared: ${tag}
  previous: ${prevVersion}
  next:     ${nextVersion}
  branch:   ${branch}
  dry-run:  ${dryRun}
  push:     ${!noPush && !dryRun}

GitHub Actions → workflow "Release" builds Windows / macOS / Linux
and publishes installers + latest.json for auto-update.

  Releases: https://github.com/hoan02/b4s/releases
  Actions:  https://github.com/hoan02/b4s/actions

Reminder: secret TAURI_SIGNING_PRIVATE_KEY must be set for updater signing.
`);
