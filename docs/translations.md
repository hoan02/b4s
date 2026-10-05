# Translation guide

B4S bundles its interface translations so every language works offline. English
is the source locale and the fallback. The app currently includes:

| Locale | Language | Status |
|---|---|---|
| `en` | English | Source and first-launch default |
| `vi` | Vietnamese | Shipped |
| `zh-CN` | Simplified Chinese | Shipped |
| `es` | Spanish | Shipped |
| `pt-BR` | Brazilian Portuguese | Shipped |

Translations live in `src/locales/<locale>/translation.json`. The app registers
locales and their native display names in `src/lib/i18n.ts`. Locale identifiers
must match i18next language codes and the directory names exactly.

## Update a translation

1. Edit the English source key in `src/locales/en/translation.json`.
2. Add or update that key in every shipped locale. Keep keys flat and grouped by
   feature prefix, such as `settings.language` or `device.connected`.
3. Preserve named placeholders exactly, for example `{{version}}` and
   `{{count}}`. Use i18next plural suffixes when a message varies by count.
4. Keep protocol bytes, model IDs, capability values and storage keys unchanged.
   Translate user-facing descriptions, not values exchanged with a device.
5. Run `npm run check:i18n` and `npm run build`.

The locale checker reports missing or extra keys, placeholder mismatches and
static UI references without an English source key. Add a dynamic key family to
the checker's allowlist when the UI derives the key from a stable ID.

## Add a locale

Add its JSON resource under `src/locales/`, import and register it in
`src/lib/i18n.ts`, and add its locale code and native display name to `LOCALES`
and `LOCALE_NAMES`. Run the locale checker and build. Include the language in
the README language links when the translation is ready to be advertised.

Use the language's standard i18next plural categories. The checker allows
locale-specific plural forms, but every message still needs a valid fallback
form. Dates, numbers and units should use locale-aware formatting when they are
presented as localized values.

## Review checklist

- Check the first-launch language, saved choice and live language switching.
- Check long labels, dialogs, toasts and narrow window sizes.
- Confirm placeholders, plural forms and accessibility labels read naturally.
- Request a native-speaker review when possible and mention its status in the PR.

The shipped set currently uses left-to-right scripts. Adding a right-to-left
language also requires a layout and interaction review before calling it
supported.
