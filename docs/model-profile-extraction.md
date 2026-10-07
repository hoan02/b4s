# Extract model profile drafts

`scripts/extract-model-profile-drafts.py` collects public configuration metadata
and exact model references from the local Baseus 2.17.0.1 JADX source. It writes
recognition-only drafts under `.tmp/model-profile-drafts`, outside the embedded
runtime catalog. It never modifies the reviewed BP1 Pro or Ultra profiles.

The 2026-10-07 run against the existing discovery snapshot produced 124 drafts:
66 have exact model references in the local 2.17.0.1 source, all 124 public US
configuration requests succeeded, and 87 returned nonempty EQ preset indices.
These counts describe available evidence, not supported control features.
Offline replay of all 124 cached configurations was also verified.

After adding the original XAPK and expanding the scan, all 124 candidates have
exact source/resource references and 120 have model branches in gesture function
configuration. See the [detailed evidence review](model-evidence-review-2.17.0.1.md).

## Run

```powershell
# Replay local model-param files and any previously fetched summaries.
python scripts/extract-model-profile-drafts.py

# Collect current public model configuration, with at most three concurrent requests.
python scripts/extract-model-profile-drafts.py --fetch

# Inspect a particular exact catalog name or another API region.
python scripts/extract-model-profile-drafts.py --fetch --region eu --model "Baseus Bowie MA10"

# Refresh discovery metadata separately without replacing the runtime snapshot.
python scripts/sync-baseus-catalog.py --output .tmp/baseus-category-current.json
python scripts/extract-model-profile-drafts.py --catalog .tmp/baseus-category-current.json --fetch

python scripts/test-extract-model-profile-drafts.py
```

When the bundled local JADX tool is available, decode the original XAPK resources
before rerunning the extraction (the APK directory is ignored by Git):

```powershell
& docs/re/tools/jadx/bin/jadx.bat --no-src --output-dir-res docs/re/apk-2.17.0.1/jadx-out/resources Baseus_2.17.0.1.xapk
python scripts/extract-model-profile-drafts.py
```

The default discovery input is the existing `baseus-public.json`; its fetch
timestamp is recorded in the report. A region may return empty or different
configuration for a model. Fetch failures appear per model and produce exit
code 2; previous cached summaries remain intact. Offline replay performs no
network requests. Cached identities must match the exact model and region.

## Outputs

- `report.json`: per-model source areas, API status, and existing reviewed profile.
- `evidence-review.md`: readable per-model evidence matrix and missing resource roots.
- `profiles/<catalog-id>.json`: envelope containing a non-importable profile draft,
  source file/line/hash references, gesture model-branch JSON pointers, configuration
  summary, and review checklist. Legacy layouts and V2 button layouts are kept
  separate, as are child-action IDs requiring additional parameters.
- `params/<region>-<catalog-id>.json`: API URL, model, fetch time, response hash and
  allowlisted summary. Original responses and APK source are not copied.

Only audio candidates are included. Models classified as speakers consistently
in every published region are excluded. All Java packages are searched, including
long mapping lines in DeviceManager. Explicit alias/canonical TuplesKt pairs
are followed; shortened names are never guessed. Metadata annotations are
excluded and ambiguous names do not match. Text assets/resources are searched
when available; the report records which resource directories are present.
JADX comment references are recorded separately from code references.

`evidenceStatus` distinguishes exact references, explicit alias references,
comment-only evidence, and models not found in the available files. No category
establishes working controls. This remains an evidence index rather than semantic
execution of every firmware/model condition; obfuscated constants and missing
resources may still need manual tracing.

Source groups such as listening, gestures, EQ and transport help prioritize
review. They are **not protocol family assignments or feature permissions**.
Empty server EQ arrays do not establish that a device lacks EQ.

## Promote a draft

Trace the referenced model branches, firmware conditions, model assets and
feature dictionaries. `/app/category/getModelParams` is a configuration source;
`/app/homepage/dictByName` requires dictionary names and conditions traced from
the consumers and is not bulk-guessed by this tool. Verify UUIDs, handshake,
framing, query/write/reply layouts and state confirmation. Then create a reviewed
profile in `src-tauri/catalog/models` with an implemented family adapter and
focused replay tests. Use `experimental` until hardware behavior is checked.

Generated drafts keep `scanOnly`, `unknown`, empty capabilities and no connection.
Run `python scripts/promote-model-profiles.py` to convert missing identities into
complete schema-3 runtime profiles: explicit aliases, category/group, passive noise
constraints and an unresolved connection contract are added. All 124 candidates
are now embedded, including the two existing BP1 profiles, which are preserved.
This promotion enables per-model runtime catalog maintenance; it does not supply
working control commands. Draft envelopes and APK/source references stay local.
