#!/usr/bin/env python3
"""Fetch public Baseus category metadata; never infer protocol capabilities.

No third-party dependencies, account, device serial, or private key required.
The output contains allowlisted product metadata, not the raw API response.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOSTS = {
    "cn": "https://bds-api-cn.baseus.cn",
    "us": "https://bds-api-us.baseus.com",
    "eu": "https://bds-api-eu.baseus.com",
}
MAX_RESPONSE_BYTES = 4 * 1024 * 1024


def model_id(model: str) -> str:
    name = re.sub(r"^baseus\s+", "", model.strip(), flags=re.I).lower()
    name = name.replace("+", " plus ")
    slug = re.sub(r"[^a-z0-9]+", "-", name).strip("-")
    # Unicode/punctuation-only names must still have a stable, nonempty ID.
    return slug or hashlib.sha256(model.encode()).hexdigest()[:12]


def public_url(value: object) -> str | None:
    if not isinstance(value, str):
        return None
    parsed = urllib.parse.urlsplit(value.strip())
    if parsed.scheme != "https" or not parsed.hostname or parsed.username or parsed.password:
        return None
    return value.strip()


def fetch(region: str, language: str, version: int) -> dict:
    url = HOSTS[region] + "/app/category?" + urllib.parse.urlencode({"version": version})
    request = urllib.request.Request(url, headers={
        "Accept": "application/json", "platform": "1", "lang": language,
        "appVersion": "2.17.0.1", "versionCode": "184",
    })
    with urllib.request.urlopen(request, timeout=20) as response:
        body = response.read(MAX_RESPONSE_BYTES + 1)
    if len(body) > MAX_RESPONSE_BYTES:
        raise ValueError(f"{region}: category response exceeds size limit")
    return json.loads(body)


def flatten(response: dict, region: str) -> list[dict]:
    if not isinstance(response, dict) or response.get("code") != 0:
        raise ValueError(f"{region}: category API did not return success")
    data = response.get("data")
    if not isinstance(data, list) or not data:
        raise ValueError(f"{region}: expected a nonempty category tree")
    products = []

    def walk(nodes: list, path: list[str], audio: bool = False, depth: int = 0) -> None:
        if depth > 12:
            raise ValueError(f"{region}: category tree is too deep")
        for node in nodes:
            if not isinstance(node, dict):
                raise ValueError(f"{region}: invalid category node")
            names = path + [str(node.get("name") or "").strip()]
            # API category type 2 is audio; IDs differ between regions.
            is_audio = audio or node.get("type") == 2
            for product in node.get("products") or []:
                if not isinstance(product, dict):
                    raise ValueError(f"{region}: invalid product")
                model = product.get("model")
                if not isinstance(model, str) or not model.strip():
                    raise ValueError(f"{region}: product has no model identity")
                colors = []
                for color in product.get("colorList") or []:
                    if isinstance(color, dict) and isinstance(color.get("color"), int):
                        colors.append({"code": color["color"], "imageUrl": public_url(color.get("url"))})
                products.append({
                    "model": model.strip(), "audio": is_audio,
                    "variant": {
                        "region": region, "productName": str(product.get("prodName") or model).strip(),
                        "categoryPath": names, "categoryId": product.get("categoryId"),
                        "imageUrl": public_url(product.get("icon")),
                        "largeImageUrl": public_url(product.get("iconLarge")),
                        "colors": sorted(colors, key=lambda c: c["code"]),
                    },
                })
            children = node.get("child") or []
            if not isinstance(children, list):
                raise ValueError(f"{region}: invalid category children")
            walk(children, names, is_audio, depth + 1)

    walk(data, [])
    if not products:
        raise ValueError(f"{region}: no products in category response")
    return products


def merge(responses: dict[str, dict]) -> tuple[list[dict], list[dict]]:
    models: dict[str, dict] = {}
    sources = []
    for region in sorted(responses):
        products = flatten(responses[region], region)
        sources.append({"region": region, "productCount": len(products),
                        "audioCount": sum(p["audio"] for p in products)})
        for product in products:
            key = product["model"].casefold()
            record = models.setdefault(key, {
                "id": model_id(product["model"]), "model": product["model"],
                "audio": product["audio"], "variants": [],
            })
            if record["audio"] != product["audio"]:
                raise ValueError(f"Conflicting category type for {product['model']}")
            if product["variant"] not in record["variants"]:
                record["variants"].append(product["variant"])
    result = sorted(models.values(), key=lambda m: m["model"].casefold())
    ids = [m["id"] for m in result]
    # Slug collisions must not silently merge distinct server model identities.
    for record in result:
        if ids.count(record["id"]) > 1:
            record["id"] += "-" + hashlib.sha256(record["model"].encode()).hexdigest()[:8]
        record["variants"].sort(key=lambda v: (v["region"], v["categoryPath"]))
    return result, sources


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--regions", nargs="+", choices=HOSTS, default=list(HOSTS))
    parser.add_argument("--lang", default="en")
    parser.add_argument("--version", type=int, default=1, help="Category schema version, not APK version")
    parser.add_argument("--input-dir", type=Path, help="Offline replay: catalog-<region>-raw.json files")
    parser.add_argument("--output", type=Path, default=ROOT / "src-tauri/catalog/baseus-public.json")
    args = parser.parse_args()
    try:
        responses = {}
        for region in dict.fromkeys(args.regions):
            responses[region] = (
                json.loads((args.input_dir / f"catalog-{region}-raw.json").read_text(encoding="utf-8"))
                if args.input_dir else fetch(region, args.lang, args.version)
            )
        models, sources = merge(responses)
        for source in sources:
            source["url"] = HOSTS[source["region"]] + f"/app/category?version={args.version}"
        snapshot = {
            "schemaVersion": 1, "fetchedAt": datetime.now(timezone.utc).isoformat(),
            "appVersion": "2.17.0.1", "categoryVersion": args.version, "language": args.lang,
            "sources": sources, "models": models,
        }
        args.output.parent.mkdir(parents=True, exist_ok=True)
        temporary = args.output.with_suffix(args.output.suffix + ".tmp")
        temporary.write_text(json.dumps(snapshot, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        temporary.replace(args.output)
        print(f"Saved {len(models)} models ({sum(m['audio'] for m in models)} audio) to {args.output}")
        print("Coverage: published category models in selected regions/language; not all firmware or hidden products.")
        return 0
    except (OSError, ValueError, TypeError) as error:
        print(f"Catalog sync failed; existing snapshot was preserved: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
