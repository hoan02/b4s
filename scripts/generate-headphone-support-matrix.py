#!/usr/bin/env python3
"""Generate the documented headphone support matrix from reviewed inputs."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PUBLIC_PATH = ROOT / "src-tauri/catalog/baseus-public.json"
PROFILES_PATH = ROOT / "src-tauri/catalog/models"
OUTPUT_PATH = ROOT / "docs/model-support-matrix.md"

CAPABILITY_LABELS = (
    ("anc", "ANC/transparency"),
    ("eq", "EQ presets"),
    ("customEq", "Custom EQ"),
    ("gameMode", "Game mode"),
    ("bassBoost", "Bass boost"),
    ("spatial", "Spatial"),
    ("ldac", "LDAC"),
    ("hearingProtection", "Hearing protection"),
    ("findBuds", "Find buds"),
)


def identity_key(name: str) -> str:
    normalized = name.strip().casefold()
    return normalized.removeprefix("baseus ")


def is_speaker_only(model: dict) -> bool:
    variants = model.get("variants", [])
    return bool(variants) and all(
        "speaker" in (variant.get("categoryPath") or [""])[-1].casefold()
        for variant in variants
    )


def load_profiles() -> dict[str, dict]:
    profiles: dict[str, dict] = {}
    for path in sorted(PROFILES_PATH.glob("*.json")):
        profile = json.loads(path.read_text(encoding="utf-8"))
        key = identity_key(profile["displayName"])
        if key in profiles:
            raise ValueError(f"duplicate reviewed profile identity: {profile['displayName']}")
        profiles[key] = profile
    return profiles


def model_row(model: dict, profile: dict | None) -> str:
    model_name = model["model"].replace("|", "\\|")
    if profile is None:
        category = (model.get("variants") or [{}])[0].get("categoryPath", ["Audio"])[-1]
        return (
            f"| `{model['id']}` | {model_name} | {category} | `scanOnly` | `unknown` "
            "| — | No reviewed control profile; recognition metadata only. |"
        )

    capabilities = profile.get("capabilities", {})
    enabled = [label for key, label in CAPABILITY_LABELS if capabilities.get(key) is True]
    feature_text = ", ".join(enabled) if enabled else "—"
    note = profile.get("connection", {}).get("provenance") or "No connection evidence."
    note = note.replace("|", "\\|").replace("\n", " ")
    return (
        f"| `{model['id']}` | {model_name} | {profile['category']} | `" 
        f"{profile['support']}` | `{profile['protocolFamily']}` | {feature_text} | {note} |"
    )


def main() -> None:
    snapshot = json.loads(PUBLIC_PATH.read_text(encoding="utf-8"))
    profiles = load_profiles()
    audio = [model for model in snapshot["models"] if model.get("audio")]
    speakers = [model for model in audio if is_speaker_only(model)]
    headphones = [model for model in audio if not is_speaker_only(model)]
    matched: set[str] = set()
    rows = []
    for model in sorted(headphones, key=lambda item: (item["model"].casefold(), item["id"])):
        profile = profiles.get(identity_key(model["model"]))
        if profile is not None:
            matched.add(identity_key(model["model"]))
        rows.append(model_row(model, profile))

    unmatched_profiles = set(profiles) - matched
    if unmatched_profiles:
        raise ValueError(f"reviewed profiles absent from headphone catalog: {sorted(unmatched_profiles)}")

    speaker_rows = "\n".join(
        f"| `{model['id']}` | {model['model']} | Excluded: speaker-only category in every region. |"
        for model in sorted(speakers, key=lambda item: item["model"].casefold())
    )
    document = f"""# Headphone model support matrix

Generated from `src-tauri/catalog/baseus-public.json` (fetched {snapshot['fetchedAt']}) and reviewed JSON profiles in `src-tauri/catalog/models/` by `scripts/generate-headphone-support-matrix.py`.

The public snapshot has {len(snapshot['models'])} products, of which {len(audio)} are audio products. Category data consistently classifies {len(speakers)} audio products as speakers; the remaining {len(headphones)} identities are candidates for headphone discovery. Recognition is metadata only. `scanOnly` and `unknown` mean no feature can be controlled.

The matrix records profile permissions, not per-feature hardware acceptance. The BP1 Pro profile currently enables the listed features, while its firmware scope and release acceptance remain tracked in [`headphone-desktop-progress.md`](headphone-desktop-progress.md). BP1 Ultra remains scan-only pending captured transport/firmware evidence. Every public candidate without a reviewed profile is scan-only with no inferred feature support.

| Catalog ID | Product | Catalog group | Support | Family | Profile-enabled features | Evidence / limit |
|---|---|---|---|---|---|---|
{chr(10).join(rows)}

## Speaker products excluded from headphone pairing

| Catalog ID | Product | Classification |
|---|---|---|
{speaker_rows}

This matrix is refreshed when the public snapshot or a reviewed profile changes. It does not replace feature/model/firmware/platform hardware reports, and synthetic replay does not promote support to verified.
"""
    OUTPUT_PATH.write_text(document, encoding="utf-8", newline="\n")


if __name__ == "__main__":
    main()
