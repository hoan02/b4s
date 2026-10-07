# ADR: offline headphone desktop scope

Status: accepted product decision, 2026-10-06.

B4S keeps SolidJS, Tauri, Rust and its five locales. Windows is the first
complete release platform. Earbuds, over-ear, neckband and open-ear belong in
scope; speakers and other products do not. Public metadata is discovery
information, not proof of a writable protocol or feature support.

Offline device control is the initial release. Catalog/image refresh is a
deliberate user action. Accounts, cloud/AI, SoundFit and OTA have separate
research/release gates; they cannot justify speculative offline commands.
macOS/Linux transport work follows Windows and has explicit platform evidence.

Close hides to tray; Quit performs cleanup. Auto-start and auto-reconnect are
separate opt-in preferences, off by default. Experimental mode is on by
default for new settings; a saved disabled choice is preserved. It never
overrides backend validation, unknown firmware constraints
or an absent transport. Only traced/replay-implemented and safety-bounded
features are eligible. Hardware verification is per feature/model/firmware/
transport/platform; replay alone cannot promote support to verified.

Metadata, reviewed profiles, family codecs and transports are separate
boundaries. Backend snapshots own confirmed device state. Desired state remains
pending until device evidence confirms it. Identity/session keys scope events,
commands and persistence. Never copy a shorter marketing alias's capabilities
onto a Plus/Ultra edition or select framing by a substring.

The runtime is being replaced by a clean transport/session architecture in
verified vertical slices. Do not keep legacy facades, generic runtime fallbacks
or dual execution after a slice has migrated. Preserve user data through an
explicit one-time, versioned migration, then remove the old resolver/schema.
Each delivery records implementation and remaining gates in
`docs/headphone-desktop-progress.md`. This direction supersedes earlier
incremental-compatibility notes in the initial plan.
