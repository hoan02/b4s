import { readdir, readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve("src/locales");
const localeIds = await readdir(root);
const readLocale = async (locale) => JSON.parse(await readFile(resolve(root, locale, "translation.json"), "utf8"));
const source = await readLocale("en");
const keys = new Set(Object.keys(source));
const placeholders = (value) => [...value.matchAll(/{{\s*([\w.-]+)\s*}}/g)].map((match) => match[1]).sort().join(",");
let failed = false;

for (const locale of localeIds.filter((id) => id !== "en")) {
  const dictionary = await readLocale(locale);
  const extra = Object.keys(dictionary).filter((key) => !keys.has(key));
  const missing = [...keys].filter((key) => !(key in dictionary) && !(key.endsWith("_one") && `${key.slice(0, -4)}_other` in dictionary));
  const badPlaceholders = Object.keys(dictionary).filter((key) => keys.has(key) && placeholders(dictionary[key]) !== placeholders(source[key]));
  if (missing.length || extra.length || badPlaceholders.length) failed = true;
  console.log(`${locale}: ${keys.size - missing.length}/${keys.size} translated; ${missing.length} use English fallback${extra.length ? `; unknown keys: ${extra.join(", ")}` : ""}${badPlaceholders.length ? `; placeholder mismatch: ${badPlaceholders.join(", ")}` : ""}`);
}

for (const id of ["classic", "bass", "cinema", "hifi", "voice", "dj", "pop", "jazz", "classical", "clear", "acoustic", "original", "rock"]) {
  for (const key of [`eqPreset.${id}`, `eqPresetSub.${id}`]) {
    if (!(key in source) && !(`${key}_one` in source && `${key}_other` in source)) {
      console.error(`English source is missing dynamic key "${key}"`);
      failed = true;
    }
  }
}

const codeFiles = ["src/App.tsx", ...((await readdir("src/components")).filter((file) => file.endsWith(".tsx")).map((file) => `src/components/${file}`))];
for (const file of codeFiles) {
  const text = await readFile(file, "utf8");
  for (const [, key] of text.matchAll(/\bt\("([\w.-]+)"/g)) {
    if (!(key in source) && !(`${key}_one` in source && `${key}_other` in source)) {
      console.error(`${file}: missing English key "${key}"`);
      failed = true;
    }
  }
}

if (failed) process.exitCode = 1;
