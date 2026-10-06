"""Inventory local Android inputs without exporting proprietary source or binaries."""
import argparse
import hashlib
import json
from pathlib import Path
import zipfile


def digest(path):
    result = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            result.update(chunk)
    return result.hexdigest()


def archive_digest(archive, entry):
    result = hashlib.sha256()
    with archive.open(entry) as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            result.update(chunk)
    return result.hexdigest()


def inventory(root):
    inputs, native, resources, errors = [], [], [], []
    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        relative = path.relative_to(root).as_posix()
        if path.suffix.lower() in (".apk", ".xapk", ".dex", ".so"):
            inputs.append({"path": relative, "size": path.stat().st_size, "sha256": digest(path)})
        if path.suffix.lower() == ".apk":
            with zipfile.ZipFile(path) as archive:
                for entry in sorted(archive.infolist(), key=lambda item: item.filename):
                    if entry.is_dir():
                        continue
                    is_native = (
                        entry.filename.startswith("lib/") and entry.filename.endswith(".so")
                    )
                    is_resource = (
                        entry.filename.startswith(("res/", "assets/"))
                        or entry.filename in ("AndroidManifest.xml", "resources.arsc")
                    )
                    if not (is_native or is_resource):
                        continue
                    record = {
                        "apk": relative,
                        "entry": entry.filename,
                        "size": entry.file_size,
                        "sha256": archive_digest(archive, entry),
                    }
                    if is_native:
                        native.append(record)
                    else:
                        resources.append(record)
        if path.suffix == ".java":
            with path.open(encoding="utf-8", errors="replace") as stream:
                for number, line in enumerate(stream, 1):
                    # Store location/category only, never source snippets.
                    if "JADX ERROR" in line or "Method not decompiled:" in line:
                        errors.append({"path": relative, "line": number,
                                       "kind": "jadx-error" if "JADX ERROR" in line else "method-not-decompiled"})
    return {
        "schemaVersion": 2,
        "inputs": inputs,
        "nativeLibraries": native,
        "apkResources": resources,
        "decompilerMarkers": errors,
        "limitations": [
            "Marker count is not JADX run error count.",
            "Package/version/tool provenance must be supplied from the original run.",
            "Only APK native and resource entries are inspected; unpack XAPK splits first.",
            "Resource inventory contains archive paths, sizes and hashes, not file contents.",
        ],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("--output", type=Path, default=Path(".tmp/android-source-inventory.json"))
    args = parser.parse_args()
    root = args.root.resolve(strict=True)
    if not root.is_dir():
        parser.error("root must be a directory")
    workspace = Path(__file__).resolve().parent.parent
    output = args.output.resolve()
    if not output.is_relative_to(workspace / ".tmp"):
        parser.error("output must stay inside the repository .tmp directory")
    report = inventory(root)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"Inventory: {len(report['inputs'])} hashed inputs, "
          f"{len(report['nativeLibraries'])} native entries, "
          f"{len(report['apkResources'])} resource entries, "
          f"{len(report['decompilerMarkers'])} decompiler markers. Local output: {output}")


if __name__ == "__main__":
    main()
