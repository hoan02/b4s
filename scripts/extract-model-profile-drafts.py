#!/usr/bin/env python3
"""Collect model evidence and recognition-only drafts; never enable device commands."""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import runpy
import shutil
import subprocess
import sys
import tempfile
import urllib.parse
import urllib.request

HOSTS = runpy.run_path(str(Path(__file__).with_name("sync-baseus-catalog.py")))["HOSTS"]

ROOT = Path(__file__).resolve().parent.parent
MAX_BYTES = 4 * 1024 * 1024
FEATURE_FILES = {
    "Noise": "listening", "Gesture": "gesture", "SoundEffect": "sound",
    "Eq": "eq", "Ldac": "ldac", "EarSound": "sound", "Bluetooth": "transport",
    "DeviceVersion": "model-selection", "DeviceManager": "model-selection", "HeadPhoneData": "framing",
}


def identity(name: str) -> str:
    return re.sub(r"^baseus\s+", "", name.strip(), flags=re.I).casefold()


def without_java_comments(content: str) -> str:
    return re.sub(r'"(?:\\.|[^"\\])*"|/\*.*?\*/|//[^\r\n]*',
        lambda match: match.group() if match.group().startswith('"') else " ", content, flags=re.S)


def candidate_java_files(source: Path, names: set[str]) -> list[Path]:
    # Optional fast prefilter; exact identity and comment checks still happen below.
    rg = shutil.which("rg")
    if not rg:
        return sorted(source.rglob("*.java"))
    with tempfile.TemporaryDirectory() as directory:
        patterns = Path(directory) / "models.txt"
        patterns.write_text("\n".join(r'"\s*(?:baseus\s+)?' + re.escape(name) + r'\s*"' for name in sorted(names)) + "\n", encoding="utf-8")
        process = subprocess.run([rg, "-l", "-i", "--hidden", "--no-ignore", "-g", "*.java",
            "-f", str(patterns), str(source)], capture_output=True, encoding="utf-8")
        if process.returncode not in (0, 1):
            raise ValueError(f"Source prefilter failed: {process.stderr.strip()}")
        return sorted(Path(line) for line in process.stdout.splitlines())


def headphone_candidates(catalog: dict) -> list[dict]:
    if catalog.get("schemaVersion") != 1 or not isinstance(catalog.get("models"), list):
        raise ValueError("Invalid public catalog")
    candidates = []
    ids = set()
    for model in catalog["models"]:
        if not isinstance(model.get("id"), str) or not re.fullmatch(r"[a-z0-9-]+", model["id"]):
            raise ValueError("Invalid model id")
        if model["id"] in ids or not isinstance(model.get("model"), str):
            raise ValueError("Duplicate id or invalid model name")
        ids.add(model["id"])
        variants = model.get("variants", [])
        speakers = variants and all(any("speaker" in str(part).lower()
            for part in variant.get("categoryPath", [])) for variant in variants)
        if model.get("audio") and not speakers:
            candidates.append(model)
    return candidates


def index_source(apk: Path, models: list[dict]) -> dict[str, list[dict]]:
    source = apk / "jadx-out" / "sources"
    if not source.is_dir():
        raise ValueError(f"Missing JADX sources: {source}")
    names: dict[str, list[str]] = {}
    for model in models:
        names.setdefault(identity(model["model"]), []).append(model["id"])
    result = {model["id"]: [] for model in models}
    # Only explicit alias -> canonical pairs establish alternate model names.
    aliases: dict[str, set[str]] = {}
    for path in source.rglob("DeviceManager.java"):
        mapping = without_java_comments(path.read_text(encoding="utf-8", errors="replace"))
        for alias, canonical in re.findall(r'TuplesKt\.to\("([^"\\]+)",\s*"([^"\\]+)"\)', mapping):
            keys = names.get(identity(canonical), [])
            aliases.setdefault(identity(alias), set()).update(keys)
    for path in candidate_java_files(source, set(names) | set(aliases)):
            features = sorted({feature for marker, feature in FEATURE_FILES.items() if marker in path.name})
            if not features:
                features = ["other-source"]
            content = path.read_bytes()
            digest = hashlib.sha256(content).hexdigest()
            in_comment = False
            for number, line in enumerate(content.decode("utf-8", errors="replace").splitlines(), 1):
                if "@Metadata" in line:
                    continue
                # Tokenize comments separately: JADX hints are evidence, not executable branches.
                tokens = re.finditer(r'"(?:\\.|[^"\\])*"|/\*|\*/|//', line)
                matched = {}
                line_comment = False
                for token in tokens:
                    text = token.group()
                    if text == "/*":
                        in_comment = True
                        continue
                    if text == "*/":
                        in_comment = False
                        continue
                    if text == "//":
                        line_comment = True
                        continue
                    literal = text[1:-1]
                    if not 1 <= len(literal) <= 160 or "\\" in literal:
                        continue
                    keys = names.get(identity(literal), [])
                    kind = "exact"
                    if not keys:
                        keys = sorted(aliases.get(identity(literal), set()))
                        kind = "explicit-alias"
                    # Shared/short names cannot establish an individual model mapping.
                    if len(keys) == 1:
                        matched[(keys[0], kind, in_comment or line_comment)] = literal
                for (key, kind, comment), literal in matched.items():
                    result[key].append({"path": path.relative_to(apk).as_posix(),
                        "line": number, "sha256": digest, "areas": features,
                        "status": "decompiler-comment" if comment else "model-reference-only",
                        "matchKind": kind, "matchedName": literal})
    # Inspect textual assets when available; never interpret binary resources as text.
    seen_resources = set()
    for root in [apk / "jadx-out/resources", apk / "extracted/assets", apk / "extracted/res"]:
        for path in sorted(root.rglob("*")):
            if not path.is_file() or path.suffix.lower() not in {".json", ".xml", ".txt", ".html", ".js"}:
                continue
            content = path.read_bytes()
            if len(content) > MAX_BYTES:
                continue
            digest = hashlib.sha256(content).hexdigest()
            if digest in seen_resources:
                continue
            seen_resources.add(digest)
            for number, line in enumerate(content.decode("utf-8", errors="replace").splitlines(), 1):
                literals = re.findall(r'"([^"\\\r\n]{1,160})"|>([^<>\r\n]{1,160})<', line)
                for pair in literals:
                    literal = pair[0] or pair[1]
                    keys = names.get(identity(literal), [])
                    kind = "exact"
                    if not keys:
                        keys = sorted(aliases.get(identity(literal), set()))
                        kind = "explicit-alias"
                    if len(keys) == 1:
                        result[keys[0]].append({"path": path.relative_to(apk).as_posix(), "line": number,
                            "sha256": digest, "areas": ["resources"],
                            "status": "resource-reference-only", "matchKind": kind, "matchedName": literal})
    return result


def evidence_status(refs: list[dict]) -> str:
    active = [ref for ref in refs if ref["status"] != "decompiler-comment"]
    if any(ref["matchKind"] == "exact" for ref in active):
        return "exact-reference"
    if active:
        return "explicit-alias-reference"
    return "comment-only" if refs else "not-found"


def index_gesture_configuration(apk: Path, models: list[dict]) -> dict[str, list[dict]]:
    names: dict[str, list[str]] = {}
    for model in models:
        names.setdefault(identity(model["model"]), []).append(model["id"])
    result = {model["id"]: [] for model in models}
    roots = [apk / "extracted/assets/gesture"]
    if not roots[0].is_dir():
        roots = list((apk / "jadx-out/resources").rglob("assets/gesture"))
    for root in roots:
        for path in sorted(root.glob("*.json")):
            body = path.read_bytes()
            digest = hashlib.sha256(body).hexdigest()

            def visit(node, pointer=""):
                if isinstance(node, dict):
                    model_names = node.get("modelList", node.get("model", []))
                    if isinstance(model_names, list):
                        keys = {key for name in model_names if isinstance(name, str)
                            for key in names.get(identity(name), []) if len(names.get(identity(name), [])) == 1}
                        for key in keys:
                            summary = {"path": path.relative_to(apk).as_posix(), "sha256": digest,
                                "jsonPointer": pointer, "kind": "gesture-configuration-only"}
                            if isinstance(node.get("functionList"), list):
                                summary["functionIds"] = [item for item in node["functionList"] if type(item) is int]
                                summary["childActionIds"] = [item["functionId"] for item in node.get("childList", [])
                                    if isinstance(item, dict) and type(item.get("functionId")) is int]
                            if isinstance(node.get("buttonType"), list):
                                summary["buttons"] = [{"buttonId": item.get("buttonId"),
                                    "layoutTypes": item.get("layoutType", []), "customActions": item.get("custom", {})}
                                    for item in node["buttonType"] if isinstance(item, dict)]
                            if isinstance(node.get("layoutType"), list):
                                summary["layoutTypes"] = [item for item in node["layoutType"] if type(item) is int]
                                summary["defaultFunctions"] = {name: value for name, value in node.get("defaultFuntion", {}).items()
                                    if type(value) is int}
                            result[key].append(summary)
                    for field, value in node.items():
                        visit(value, pointer + "/" + field.replace("~", "~0").replace("/", "~1"))
                elif isinstance(node, list):
                    for offset, value in enumerate(node):
                        visit(value, pointer + f"/{offset}")

            visit(json.loads(body))
    return result


def write_review(path: Path, report: dict) -> None:
    lines = ["# APK model evidence review", "", f"Generated: {report['generatedAt']}", "",
        "This indexes model references, not verified feature capabilities or protocol families.", "",
        "| Model | Evidence | References | Gesture entries | Source areas |", "| --- | --- | ---: | ---: | --- |"]
    for row in report["models"]:
        name = row["model"].replace("|", "\\|")
        lines.append(f"| {name} | {row['evidenceStatus']} | {row['sourceReferences']} | {row.get('gestureConfigurationEntries', 0)} | {', '.join(row['areas'])} |")
    lines.extend(["", "## Resource availability", ""])
    lines.extend(f"- {root}: {'present' if present else 'missing'}" for root, present in report["resourceRoots"].items())
    lines.extend(["", "Missing evidence does not prove that the official app lacks support.",
        "See profiles/<catalog-id>.json for local file/line/hash references. No control commands are enabled.", ""])
    path.write_text("\n".join(lines), encoding="utf-8")


def summarize_params(response: dict) -> dict:
    if not isinstance(response, dict) or response.get("code") != 0 or not isinstance(response.get("data"), dict):
        raise ValueError("Model params API did not return a successful object")
    data = response["data"]
    eq = data.get("eq_sound_mode", [])
    return {"keys": sorted(data), "listCounts": {key: len(value) for key, value in data.items() if isinstance(value, list)},
        "eqPresetSorts": [item["dictSort"] for item in eq if isinstance(item, dict) and isinstance(item.get("dictSort"), int)] if isinstance(eq, list) else [],
        "note": "Empty EQ metadata does not prove absence of EQ; no command capabilities inferred."}


def fetch_params(model: dict, region: str) -> tuple[dict, str]:
    url = HOSTS[region] + "/app/category/getModelParams?" + urllib.parse.urlencode({"model": model["model"]})
    request = urllib.request.Request(url, headers={"Accept": "application/json", "platform": "1", "lang": "en",
        "appVersion": "2.17.0.1", "versionCode": "184"})
    with urllib.request.urlopen(request, timeout=15) as response:
        body = response.read(MAX_BYTES + 1)
    if len(body) > MAX_BYTES:
        raise ValueError("API response exceeds size limit")
    payload = json.loads(body)
    return {"model": model["model"], "url": url, "region": region, "sha256": hashlib.sha256(body).hexdigest(),
        "summary": summarize_params(payload)}, "ok"


def atomic_json(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    temporary.replace(path)


def validate_cached_identity(value: dict, model_name: str, region: str) -> None:
    # Earlier caches recorded the exact requested model in the URL only.
    cached_model = value.get("model")
    if cached_model is None:
        cached_model = urllib.parse.parse_qs(urllib.parse.urlsplit(value.get("url", "")).query).get("model", [None])[0]
    if cached_model != model_name or value.get("region") != region:
        raise ValueError("Cached model or region identity mismatch")
    value["model"] = cached_model


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apk-dir", type=Path, default=ROOT / "docs/re/apk-2.17.0.1")
    parser.add_argument("--catalog", type=Path, default=ROOT / "src-tauri/catalog/baseus-public.json")
    parser.add_argument("--output", type=Path, default=ROOT / ".tmp/model-profile-drafts")
    parser.add_argument("--fetch", action="store_true", help="Read public per-model configuration APIs")
    parser.add_argument("--region", choices=HOSTS, default="us")
    parser.add_argument("--model", action="append", help="Exact public model name; repeat to select several")
    args = parser.parse_args()
    # Local evidence stays ignored and cannot overwrite embedded runtime profiles.
    output = args.output.resolve()
    if not output.is_relative_to((ROOT / ".tmp").resolve()):
        parser.error("Output must be under .tmp; drafts must not overwrite runtime profiles")
    try:
        catalog = json.loads(args.catalog.read_text(encoding="utf-8"))
        models = headphone_candidates(catalog)
        if args.model:
            wanted = set(args.model)
            available = {model["model"] for model in models}
            if wanted - available:
                raise ValueError(f"Unknown exact model names: {sorted(wanted - available)}")
            models = [model for model in models if model["model"] in wanted]
        evidence = index_source(args.apk_dir, models)
        gesture_assets = index_gesture_configuration(args.apk_dir, models)
        runtime_profiles = [(path, json.loads(path.read_text(encoding="utf-8")))
            for path in (ROOT / "src-tauri/catalog/models").glob("*.json")]
        reviewed = {identity(profile["displayName"]): path.name
            for path, profile in runtime_profiles if profile["support"] != "scanOnly"}
        feature_keys = sorted({key for path in (ROOT / "src-tauri/catalog/models").glob("*.json")
            for key in json.loads(path.read_text(encoding="utf-8"))["featureEvidence"]})

        def collect(model: dict) -> tuple[str, dict]:
            key = model["id"]
            cache = output / "params" / f"{args.region}-{key}.json"
            if args.fetch:
                try:
                    value, _ = fetch_params(model, args.region)
                    value["fetchedAt"] = datetime.now(timezone.utc).isoformat()
                    atomic_json(cache, value)
                    return key, {"status": "fetched", **value}
                except (OSError, ValueError, TypeError) as error:
                    return key, {"status": "failed", "reason": str(error)}
            if cache.exists():
                value = json.loads(cache.read_text(encoding="utf-8"))
                validate_cached_identity(value, model["model"], args.region)
                return key, {"status": "cached", **value}
            legacy = args.apk_dir / (re.sub(r"[^a-z0-9]+", "-", model["model"].lower()).strip("-") + "-params.json")
            if legacy.exists():
                body = legacy.read_bytes()
                return key, {"status": "local", "path": legacy.relative_to(args.apk_dir).as_posix(),
                    "sha256": hashlib.sha256(body).hexdigest(), "summary": summarize_params(json.loads(body))}
            return key, {"status": "not-fetched"}

        with ThreadPoolExecutor(max_workers=3) as pool:
            params = dict(pool.map(collect, models))
        rows = []
        for model in models:
            key = model["id"]
            refs = evidence[key]
            draft = {"draftSchemaVersion": 1, "model": model["model"], "catalogId": key,
                "runtimeProfileDraft": {"schemaVersion": 3, "id": key, "displayName": model["model"],
                    "aliases": [], "support": "scanOnly", "protocolFamily": "unknown", "capabilities": {},
                    "connection": None, "eq": None, "image": None, "sound": None,
                    "featureEvidence": {feature: {"status": "unknown", "provenance": "", "firmwareVersions": []}
                        for feature in feature_keys}},
                "existingReviewedProfile": reviewed.get(identity(model["model"])),
                "regions": sorted({variant["region"] for variant in model.get("variants", [])}),
                "sourceReferences": refs, "evidenceStatus": evidence_status(refs), "modelParams": params[key],
                "gestureConfigurationEvidence": gesture_assets[key],
                "reviewRequired": ["Trace exact model/firmware branch", "Verify transport, UUIDs and framing",
                    "Trace each feature query/write/reply", "Verify state confirmation on hardware"],
                "importable": False}
            atomic_json(output / "profiles" / f"{key}.json", draft)
            rows.append({"model": model["model"], "id": key, "areas": sorted({area for ref in refs for area in ref["areas"]}),
                "sourceReferences": len(refs), "paramsStatus": params[key]["status"],
                "evidenceStatus": evidence_status(refs),
                "gestureConfigurationEntries": len(gesture_assets[key]),
                "reviewedProfile": draft["existingReviewedProfile"]})
        report = {"schemaVersion": 1, "generatedAt": datetime.now(timezone.utc).isoformat(),
            "apkVersion": "2.17.0.1", "catalogFetchedAt": catalog.get("fetchedAt"), "models": rows,
            "coverage": {status: sum(row["evidenceStatus"] == status for row in rows)
                for status in ["exact-reference", "explicit-alias-reference", "comment-only", "not-found"]},
            "resourceRoots": {relative: (args.apk_dir / relative).is_dir() for relative in
                ["jadx-out/resources", "extracted/assets", "extracted/res"]},
            "groups": {area: [row["id"] for row in rows if area in row["areas"]]
                for area in sorted({area for row in rows for area in row["areas"]})},
            "note": "Groups index source areas, not verified protocol families. No runtime capabilities enabled."}
        atomic_json(output / "report.json", report)
        write_review(output / "evidence-review.md", report)
        failures = sum(row["paramsStatus"] == "failed" for row in rows)
        print(f"Saved {len(rows)} drafts; {sum(bool(row['sourceReferences']) for row in rows)} with exact APK references; {failures} API failures.")
        print(f"Report: {output / 'report.json'}")
        return 2 if failures else 0
    except (OSError, ValueError, TypeError, KeyError) as error:
        print(f"Extraction failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
