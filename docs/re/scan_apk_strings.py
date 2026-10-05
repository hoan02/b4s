#!/usr/bin/env python3
"""Scan Baseus base APK for UUIDs, BA opcodes, model names."""
from __future__ import annotations

import re
import argparse
import struct
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT_ROOT = ROOT.parent.parent
LEGACY_APK = ROOT / "apk-2.14.1" / "extracted" / "com.baseus.intelligent.apk"
ROOT_APK = PROJECT_ROOT / "baseus-2-14-1.apk"
APK = LEGACY_APK if LEGACY_APK.is_file() else ROOT_APK
OUT = ROOT / "apk-2.14.1" / "strings"


def dex_strings(data: bytes):
    """Read DEX string IDs rather than treating length-prefixed bytes as text."""
    if len(data) < 112 or not data.startswith(b"dex\n"):
        raise ValueError("Invalid DEX header")
    count, offset = struct.unpack_from("<II", data, 56)
    if offset + count * 4 > len(data):
        raise ValueError("Invalid DEX string table")
    for index in range(count):
        cursor = struct.unpack_from("<I", data, offset + index * 4)[0]
        # Skip the ULEB128 UTF-16 length. Strings below are evidence candidates;
        # decoding modified UTF-8 lossily is sufficient for ASCII opcodes/UUIDs.
        for _ in range(5):
            if cursor >= len(data):
                raise ValueError("Truncated DEX string length")
            byte = data[cursor]
            cursor += 1
            if byte < 128:
                break
        else:
            raise ValueError("Invalid DEX string length")
        end = data.find(b"\0", cursor)
        if end == -1:
            raise ValueError("Unterminated DEX string")
        yield data[cursor:end].decode("utf-8", errors="replace")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apk", type=Path, default=APK)
    parser.add_argument("--out", type=Path, default=OUT)
    args = parser.parse_args()
    if not args.apk.is_file():
        raise SystemExit(
            f"APK not found: {args.apk}\nUse --apk to select an extracted base APK."
        )

    args.out.mkdir(parents=True, exist_ok=True)
    ba_cmds: set[str] = set()
    uuids: set[str] = set()
    models: set[str] = set()
    with zipfile.ZipFile(args.apk) as z:
        for name in z.namelist():
            if not name.endswith(".dex"):
                continue
            data = z.read(name)
            for s in dex_strings(data):
                if re.fullmatch(r"BA(?:[0-9A-Fa-f]{2}){1,32}", s):
                    ba_cmds.add(s.upper())
                for uuid in re.findall(
                    r"[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-"
                    r"[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}", s
                ):
                    uuids.add(uuid.upper())
                if len(s) <= 100 and re.match(r"^(Baseus\s|Bowie\s|AeQur\s|AirNora\s|Bass\s)", s):
                    models.add(s)

    (args.out / "ba_commands.txt").write_text("\n".join(sorted(ba_cmds)), encoding="utf-8")
    (args.out / "uuids.txt").write_text("\n".join(sorted(uuids)), encoding="utf-8")
    (args.out / "models_hits.txt").write_text("\n".join(sorted(models)), encoding="utf-8")
    print(f"BA commands: {len(ba_cmds)}")
    print(f"UUIDs:       {len(uuids)}")
    print(f"Model hits:  {len(models)}")
    print(f"Wrote: {args.out}")
    print("Candidates only: model names/opcodes are not proof of hardware support.")


if __name__ == "__main__":
    main()
