#!/usr/bin/env python3
"""Scan Baseus base APK for UUIDs, BA opcodes, model names."""
from __future__ import annotations

import re
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT_ROOT = ROOT.parent.parent
LEGACY_APK = ROOT / "apk-2.14.1" / "extracted" / "com.baseus.intelligent.apk"
ROOT_APK = PROJECT_ROOT / "baseus-2-14-1.apk"
APK = LEGACY_APK if LEGACY_APK.is_file() else ROOT_APK
OUT = ROOT / "apk-2.14.1" / "strings"


def main() -> None:
    if not APK.is_file():
        raise SystemExit(
            f"APK not found: {ROOT_APK} or {LEGACY_APK}\n"
            "Place baseus-2-14-1.apk at the project root or extract the legacy XAPK."
        )

    OUT.mkdir(parents=True, exist_ok=True)
    ba_cmds: set[str] = set()
    uuids: set[str] = set()
    models: set[str] = set()
    ascii_re = re.compile(rb"[\x20-\x7e]{4,200}")

    with zipfile.ZipFile(APK) as z:
        for name in z.namelist():
            if not name.endswith(".dex"):
                continue
            data = z.read(name)
            for m in re.finditer(rb"BA[0-9A-Fa-f]{2,8}", data):
                ba_cmds.add(m.group().decode())
            for m in re.finditer(
                rb"\$[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-"
                rb"[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}",
                data,
            ):
                uuids.add(m.group().decode().lstrip("$"))
            for m in ascii_re.finditer(data):
                s = m.group().decode("ascii", errors="ignore")
                if "BP1" in s or "Baseus Bass" in s or "EP10 Ultra" in s:
                    models.add(s[:200])

    (OUT / "ba_commands.txt").write_text("\n".join(sorted(ba_cmds)), encoding="utf-8")
    (OUT / "uuids.txt").write_text("\n".join(sorted(uuids)), encoding="utf-8")
    (OUT / "models_hits.txt").write_text("\n".join(sorted(models)), encoding="utf-8")
    print(f"BA commands: {len(ba_cmds)}")
    print(f"UUIDs:       {len(uuids)}")
    print(f"Model hits:  {len(models)}")
    print(f"Wrote: {OUT}")


if __name__ == "__main__":
    main()
