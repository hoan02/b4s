# Baseus 2.17.0.1 model evidence review

Reviewed on 2026-10-07 using the supplied `Baseus_2.17.0.1.xapk`, the local JADX
sources and the existing public CN/US/EU discovery snapshot. The XAPK manifest
identifies `com.baseus.intelligent`, version 2.17.0.1, version code 184, with a
base APK plus arm64-v8a and mdpi splits. JADX 1.5.1 decoded its resources locally.
The original bundle, decompiled files and extracted assets remain ignored.

## Coverage

| Evidence | Result |
| --- | ---: |
| Headphone candidates from the discovery snapshot | 124 |
| Exact model references in expanded source/resources | 124 |
| Model branches in gesture function configuration | 120 |
| Assets in the base APK | 764 |
| Text assets extracted directly for inspection | 526 |

The earlier 66-model result searched selected class names, skipped long lines
and had no decoded resources. Expanded tracing searches all Java packages,
includes DeviceManager mapping lines, follows explicit alias/canonical pairs
and inspects textual resources. It separates JADX comments from source code
references and deduplicates identical resource copies by hash.

The four models without an exact branch in the inspected gesture configuration
are Baseus P1, Baseus P1 Lite, Baseus P1x and Bowie H1 Pro. They do appear in
DeviceManager source branches; absence from these assets does not prove lack
of gesture support. They require tracing the relevant consumers and any
dynamic/default configuration.

## BP1 Ultra cross-check

BP1 Ultra has 50 distinct reference records after resource deduplication:
37 source-code records, nine resource records and four JADX comment records.
Its gesture configuration has nine structured entries, including both button
IDs, layout types and per-action function lists. The V2 schema also describes
child actions; listing a function ID alone does not define its complete command.
The inspected assets agree with the reviewed profile's two-button layout types
0 through 3. The runtime profile intentionally remains a separately reviewed
experimental profile with its existing transport and protocol adapter.

See [hardware and protocol evidence](protocol/bp1-ultra-ble.md) for the scope
of previous device readbacks. This review did not exercise hardware, establish
firmware coverage or authorize additional controls. Empty public EQ arrays do
not prove absence of EQ, and shared configuration lists do not establish a
shared BLE protocol family.

## Local artifacts

- `.tmp/model-profile-drafts/evidence-review.md`: complete per-model matrix.
- `.tmp/model-profile-drafts/report.json`: coverage and resource availability.
- `.tmp/model-profile-drafts/profiles/*.json`: source path/line/hash references,
  gesture JSON pointers, configuration summaries and non-importable drafts.
- `.tmp/model-profile-drafts/xapk-assets.json`: local asset inventory.

The [extraction guide](model-profile-extraction.md) explains replay and review.
Models must still have their transport, framing, queries, writes and state
confirmation traced before entering the runtime control catalog.
