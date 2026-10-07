#!/usr/bin/env python3
"""One-time catalog/file migration; keep legacy IDs only in saved-data migration."""
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent


def main():
    directory = ROOT / "src-tauri/catalog/models"
    profiles = [(path, json.loads(path.read_text(encoding="utf-8"))) for path in directory.glob("*.json")]
    destinations = [profile["id"].removeprefix("server-") for _, profile in profiles]
    if len(set(destinations)) != len(destinations):
        raise ValueError("Normalized model IDs collide")
    mapping = {profile["id"]: profile["id"].removeprefix("server-") for _, profile in profiles}
    migration = ROOT / "src/lib/modelIdMigration.ts"
    source = migration.read_text(encoding="utf-8")
    # Existing historical aliases and the immediately previous server IDs both map to canonical IDs.
    source = re.sub(r': "server-([^"\n]+)"', r': "\1"', source)
    entries = "\n".join(f'  "{old}": "{new}",' for old, new in sorted(mapping.items())
                        if old != new and f'"{old}":' not in source)
    if entries:
        source = source.replace('export const LEGACY_MODEL_ID_MAP = {', 'export const LEGACY_MODEL_ID_MAP = {\n' + entries)
    source = source.replace('b4s.migration.model-id.v1', 'b4s.migration.model-id.v2')
    for path, profile in profiles:
        profile["id"] = mapping[profile["id"]]
        target = directory / (profile["id"] + ".json")
        target.write_text(json.dumps(profile, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        if path != target:
            path.unlink()
    snapshot_path = ROOT / "src-tauri/catalog/baseus-public.json"
    snapshot = json.loads(snapshot_path.read_text(encoding="utf-8"))
    for model in snapshot["models"]:
        model["id"] = model["id"].removeprefix("server-")
    snapshot_path.write_text(json.dumps(snapshot, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    migration.write_text(source, encoding="utf-8")
    print(f"Normalized {len(profiles)} runtime profiles and saved-data migration.")


if __name__ == "__main__":
    main()
