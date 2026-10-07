# B4S

English | [Tiếng Việt](README.vi.md) | [Español](README.es.md) | [简体中文](README.zh-CN.md) | [Português (Brasil)](README.pt-BR.md)

B4S is an independent desktop companion for controlling selected Bluetooth LE
earbuds on Windows, macOS and Linux. It is built with SolidJS, Tauri and Rust.

| Scan and connect | Device controls | Settings |
|---|---|---|
| ![Scan and connect](assets/i1.png) | ![Battery, ANC and audio controls](assets/i2.png) | ![Settings](assets/i3.png) |

Screenshots show the Vietnamese interface.

## What is new in 0.1.3

The catalog embeds 124 headphone profiles, including 122 recognition-only models. Product names and images use public metadata; thumbnails and large photos are cached locally (64 MiB) for reuse offline. Model IDs are normalized and saved identities/EQ are migrated automatically.

Users of 0.1.1/0.1.2 must install 0.1.3 manually once because the updater signing key changed.

[Changelog](CHANGELOG.md) · [Model contract](docs/model-identity-presentation.md) · [Image cache](docs/product-image-cache.md)

## Device support

BP1 Pro has a reviewed model profile. BP1 Ultra supports experimental BLE/789C
connection, battery reporting, ANC, game mode, spatial audio, Bass Boost, LDAC,
hearing protection and gestures. EQ/SoundFit remain unavailable. Other catalog entries may
be experimental or recognition-only. A name match does not mean that control
has been verified; check the support level shown by the app and the model
catalog before relying on a feature.

| Level | Meaning |
|---|---|
| Verified | Commands and behavior have been checked on real hardware. |
| Experimental | A protocol profile exists, but model or firmware behavior needs more testing. |
| Scan only | The app recognizes the device; control is not enabled. |

See [the model catalog](docs/model-catalog.md) and [protocol notes](docs/protocol/overview.md).

## Features

- Scan for and connect to Bluetooth LE earbuds.
- Optionally reconnect to the last supported earbuds once at startup. Turn on
  **Reconnect automatically** in Settings; it is off by default. B4S searches
  for up to 12 seconds before falling back to manual selection. The earbuds
  must expose their BLE control service; a Windows audio connection alone does
  not guarantee this.
- Show left, right and case battery levels when the device reports them.
- Control noise cancellation, transparency and supported listening modes.
- Adjust EQ presets and custom EQ where the model profile allows it.
- Use spatial audio, game mode and find-earbuds controls on supported models.
- Choose a light or dark theme and check for app updates.
- Optionally start at sign-in. Closing the window hides B4S to the system tray
  when the tray is available; use **Quit B4S** in the tray menu to exit.

Controls vary by model and firmware. B4S avoids sending unsupported commands
when the profile does not provide the required capability.

## Development

Requirements: Bun 1.4.0, Rust stable, the Tauri platform prerequisites, and
Bluetooth hardware for device testing. Install dependencies and start the app:

```sh
bun install --frozen-lockfile
bun run tauri:dev
```

Before submitting changes, run the checks in [Contributing](CONTRIBUTING.md).

## Languages

The app starts in English and includes Vietnamese, Simplified Chinese, Spanish
and Brazilian Portuguese. Change the language in **Settings**. Translations are
bundled with the app and work offline. See the [translation guide](docs/translations.md)
to improve an existing locale or contribute another one.

## Project guides

- [Contributing](CONTRIBUTING.md)
- [Architecture and extension points](docs/architecture.md)
- [Adding a model or protocol family](docs/model-catalog.md)
- [Protocol overview](docs/protocol/overview.md)
- [Release and auto-update](docs/release.md)
- [Desktop troubleshooting and diagnostics](docs/desktop-troubleshooting.md)

## Disclaimer and safe use

B4S is independent, non-commercial open-source software. It is not sponsored,
certified or officially affiliated with Baseus or any earbud manufacturer.
Product names and trademarks belong to their respective owners and are used
only to identify compatibility.

Device controls run locally over Bluetooth. The app does not require an account
or send device or personal data to a server. Internet access is only needed for
actions you initiate, such as checking for updates or opening an external link.

You are responsible for device connections, firmware updates and changes to
volume, EQ, ANC or spatial audio. Find-earbuds may play a loud sound: remove the
earbuds before using it. Stop if you feel pain, ringing or discomfort. B4S is
provided as-is and does not guarantee compatibility, uninterrupted operation,
hardware safety or firmware recovery.

Do not add official APKs, private keys, account data, firmware or copyrighted
decompiled source to this repository. Follow applicable laws, device terms and
intellectual-property rights.

## License

MIT
