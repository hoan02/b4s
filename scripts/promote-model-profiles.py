#!/usr/bin/env python3
"""Promote extracted identities into schema-3 runtime profiles without guessing controls."""
import json
from pathlib import Path
import runpy

ROOT = Path(__file__).resolve().parent.parent
helpers = runpy.run_path(str(ROOT / "scripts/extract-model-profile-drafts.py"))


def main():
    destination = ROOT / "src-tauri/catalog/models"
    catalog = json.loads((ROOT / "src-tauri/catalog/baseus-public.json").read_text(encoding="utf-8"))
    existing = {helpers["identity"](json.loads(path.read_text(encoding="utf-8"))["displayName"])
                for path in destination.glob("*.json")}
    pending = []
    for model in helpers["headphone_candidates"](catalog):
        if helpers["identity"](model["model"]) in existing:
            continue
        path = ROOT / ".tmp/model-profile-drafts/profiles" / f'{model["id"]}.json'
        envelope = json.loads(path.read_text(encoding="utf-8"))
        profile = envelope["runtimeProfileDraft"]
        if (profile["id"] != model["id"] or profile["displayName"] != model["model"]
                or profile["schemaVersion"] != 3 or profile["support"] != "scanOnly"
                or profile["protocolFamily"] != "unknown" or profile["capabilities"]
                or profile["connection"] is not None):
            raise ValueError(f"Unexpected draft contract: {path}")
        profile["aliases"] = sorted({model["model"].lower(), helpers["identity"](model["model"])})
        profile["category"] = "audio"
        categories = model["variants"][0]["categoryPath"] if model["variants"] else []
        profile["group"] = categories[-1] if categories else "Audio"
        profile["noise"] = {"supportsAdaptive": False, "environments": [],
                            "maxCustomLevel": 0, "supportsTransparencyVoice": False}
        profile["connection"] = {"transport": "unresolved", "framing": "unresolved",
                                 "serviceUuid": None, "writeUuid": None, "notifyUuid": None,
                                 "handshake": [], "initStateQuery": False, "firmwareVersions": [],
                                 "provenance": "docs/model-profile-extraction.md; APK 2.17.0.1 identity extraction; control transport unresolved"}
        pending.append((destination / f'{model["id"]}.json', profile))
    # Validate the entire input before writing; existing model profiles are never replaced.
    for path, profile in pending:
        with path.open("x", encoding="utf-8", newline="\n") as stream:
            stream.write(json.dumps(profile, ensure_ascii=False, indent=2) + "\n")
    print(f"Promoted {len(pending)} profiles; preserved {len(existing)} existing profiles.")


if __name__ == "__main__":
    main()
