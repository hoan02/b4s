# Rust dependency maintenance

The backend uses Tauri's official desktop plugins, btleplug for platform BLE,
Tokio for asynchronous synchronization, Serde for serialized contracts, and
thiserror for typed errors. Device-specific protocol rules stay in the existing
adapters and are checked against reviewed capture tests.

## Stable baseline (2026-10-07)

| Dependency | Locked version |
| --- | --- |
| Tauri | 2.12.1 |
| btleplug | 0.13.4 |
| reqwest | 0.13.5 |
| thiserror (direct) | 2.0.21 |
| semver | 1.0.28 |
| Tokio | 1.53.2 |

`Cargo.lock` records the exact versions of all official Tauri plugins. Dependency
constraints remain on stable major/minor lines; Tauri 3 prereleases are excluded.
HTTP uses reqwest's `rustls-no-provider` feature with the `ring` provider selected
through the direct rustls dependency, matching the updater's crypto backend.

Fallback release comparisons use `semver::Version::cmp_precedence`, including
prerelease ordering and ignoring build metadata. Global lazy initialization uses
Rust's `LazyLock` and `OnceLock` rather than a direct once_cell dependency.

The Baseus CRC16 and frame layouts are device-specific. Do not replace them with
a generic CRC/framing library without proving byte-for-byte equivalence against
the captured frames and malformed-frame tests.

After updating the lockfile, run `cargo check`, `cargo test --lib`, and the frontend
build. Changes to btleplug additionally need hardware checks for scanning,
connect/disconnect, notifications, battery reads and acknowledged commands on
each supported operating system; compilation and unit tests cannot verify the
OS Bluetooth stack.
