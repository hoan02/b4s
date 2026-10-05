# Contributing to B4S

Contributions to device support, protocol research, UI, documentation and bug fixes
are welcome. Start with [the architecture](docs/architecture.md) and
[the model guide](docs/model-catalog.md).

## Local development

Install Node.js (CI uses 20), Rust stable and the platform prerequisites for
Tauri 2. Run `npm ci`, then `npm run tauri:dev`. Bluetooth hardware is only
needed for integration testing; Rust frame and catalog tests run without it.

Before submitting a pull request, run:

```sh
npx tsc --noEmit
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

## Choose the right contribution

- A model using an existing wire protocol: add a JSON profile and validate it.
- Different commands or replies: add or extend a protocol family with frame tests.
- Connection or discovery issues: inspect the registry and BLE transport first.
- UI changes: include screenshots and check supported and unsupported features.

Keep changes focused. Do not duplicate a screen or copy an adapter for each model.
Use two-space indentation in TypeScript and standard rustfmt formatting in Rust.
Avoid repository-wide formatting in an unrelated change.

## Evidence for device support

Report the exact model, firmware, OS and each feature exercised. Distinguish
recognition, successful GATT connection, decoded state and confirmed writes.
A command acknowledged by the device does not by itself prove the feature works.
Use `experimental` until hardware captures and observed behavior support the
claim. Recognition alone belongs to `scanOnly` with an `unknown` protocol.

Attach minimal, sanitized hex frames and expected output to tests or protocol
documentation. Remove addresses, serial numbers and account information. Do not
commit APKs, decompiled proprietary source, firmware, secrets or generated files.

## Pull requests

Describe the user-visible change, validation results and remaining limitations.
For protocol changes, link the documented evidence. State explicitly when real
hardware, a firmware version or a platform could not be tested. The PR template
contains the review checklist.
