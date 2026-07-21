//! Baseus earbud model registry
//!
//! Display names from official Baseus app (com.baseus.intelligent) supported list.
//! BLE advertising names are best-effort patterns (case-insensitive contains).
//!
//! SupportLevel:
//!   Verified     — packet table confirmed on real hardware
//!   Experimental — listed in official app; try BP1-compatible GATT/framing
//!   ScanOnly     — recognised in scan UI only (no control commands yet)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupportLevel {
    /// Full control verified on hardware
    Verified,
    /// Official app supports it; we try BP1-compatible protocol
    Experimental,
    /// Name matched only — no protocol mapping yet
    ScanOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtocolFamily {
    /// Bass BP1 Pro ANC — AA/BA frames, custom GATT UUIDs
    Bp1Pro,
    /// Unknown / not mapped
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    /// Substrings matched against BLE local_name (lowercase)
    pub name_patterns: Vec<String>,
    pub support: SupportLevel,
    pub protocol: ProtocolFamily,
    pub has_anc: bool,
    pub has_eq: bool,
    pub has_game_mode: bool,
    pub category: String,
}

fn m(
    id: &str,
    display: &str,
    patterns: &[&str],
    support: SupportLevel,
    protocol: ProtocolFamily,
    anc: bool,
    eq: bool,
    game: bool,
    category: &str,
) -> ModelInfo {
    ModelInfo {
        id: id.into(),
        display_name: display.into(),
        name_patterns: patterns.iter().map(|s| s.to_lowercase()).collect(),
        support,
        protocol,
        has_anc: anc,
        has_eq: eq,
        has_game_mode: game,
        category: category.into(),
    }
}

/// Full catalog — TWS + related from official Baseus app listing (Jun 2026).
pub fn all_models() -> Vec<ModelInfo> {
    vec![
        // ── Verified ──────────────────────────────────────────────
        m(
            "bass-bp1-pro",
            "Baseus Bass BP1 Pro ANC",
            &["bass bp1 pro", "bp1 pro", "bowie bp1 pro", "bass bp1"],
            SupportLevel::Verified,
            ProtocolFamily::Bp1Pro,
            true,
            true,
            true,
            "tws",
        ),
        // ── Experimental (official app + try BP1 family) ──────────
        // Popular ANC flagships first
        m("bowie-ma10", "Baseus Bowie MA10", &["bowie ma10", "ma10"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        m("bowie-ma10s", "Baseus Bowie MA10s", &["bowie ma10s", "ma10s"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        m("bowie-ma10-pro", "Baseus Bowie MA10 Pro", &["bowie ma10 pro", "ma10 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        m("bowie-ma20", "Baseus Bowie MA20", &["bowie ma20", "ma20"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        m("bowie-ma20-pro", "Baseus Bowie MA20 Pro", &["bowie ma20 pro", "ma20 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        m("bass-bd1", "Baseus Bass BD1", &["bass bd1", "bd1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        m("bass-1-plus", "Baseus Bass 1+", &["bass 1+", "bass 1 plus"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        // Bowie M series
        m("bowie-m3", "Baseus Bowie M3", &["bowie m3"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        m("bowie-m2s-ultra", "Baseus M2s Ultra", &["m2s ultra"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        m("bowie-m2s-pro", "Baseus Bowie M2s Pro", &["bowie m2s pro", "m2s pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        m("bowie-m2s", "Baseus Bowie M2s", &["bowie m2s", "m2s"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        m("bowie-m2-plus", "Baseus Bowie M2+", &["bowie m2+", "bowie m2 plus", "m2+"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws"),
        m("bowie-m2", "Baseus Bowie M2", &["bowie m2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws"),
        m("bowie-m1", "Baseus Bowie M1", &["bowie m1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws"),
        // Bowie E series
        m("bowie-e13", "Baseus Bowie E13", &["bowie e13", "e13"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, true, "tws"),
        m("bowie-e12", "Baseus Bowie E12", &["bowie e12", "e12"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, true, "tws"),
        m("bowie-e10", "Baseus Bowie E10", &["bowie e10", "e10"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-e8", "Baseus Bowie E8", &["bowie e8"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-e5", "Baseus Bowie E5", &["bowie e5"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-e5x", "Baseus Bowie E5x", &["bowie e5x", "e5x"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-e3", "Baseus Bowie E3", &["bowie e3"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, true, "tws"),
        m("bowie-e2", "Baseus Bowie E2", &["bowie e2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("baseus-e9", "Baseus E9", &["baseus e9", "e9"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-ex", "Baseus Bowie EX", &["bowie ex"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        // Bowie W / WM
        m("bowie-w04-plus", "Baseus Bowie W04 Plus", &["bowie w04 plus", "w04 plus", "w04+"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-w04-pro", "Baseus Bowie W04 Pro", &["bowie w04 pro", "w04 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-w04", "Baseus Bowie W04", &["bowie w04", "w04"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-wm05", "Baseus Bowie WM05", &["bowie wm05", "wm05"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-wm03", "Baseus Bowie WM03", &["bowie wm03", "wm03"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-wm02-plus", "Baseus Bowie WM02+", &["bowie wm02+", "wm02+", "wm02 plus"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-wm02", "Baseus Bowie WM02", &["bowie wm02", "wm02"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-wm01-plus", "Baseus Bowie WM01 Plus", &["bowie wm01 plus", "wm01 plus", "wm01plus"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-wm01", "Baseus Bowie WM01", &["bowie wm01", "wm01"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-wx5", "Baseus Bowie WX5", &["bowie wx5", "wx5"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-mz10", "Baseus Bowie MZ10", &["bowie mz10", "mz10"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-ez10", "Baseus Bowie EZ10", &["bowie ez10", "ez10"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        // Open-ear / MC / MF / AirGo
        m("bowie-mc1", "Baseus Bowie MC1", &["bowie mc1", "mc1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-mc2", "Baseus Bowie MC2", &["bowie mc2", "mc2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-mf1", "Baseus Bowie MF1", &["bowie mf1", "mf1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("airgo-as01", "Baseus AirGo AS01", &["airgo as01", "as01"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("airgo-1-ring", "Baseus AirGo 1 Ring", &["airgo 1 ring", "airgo ring"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("airgo-ag20", "Baseus AirGo AG20", &["airgo ag20", "ag20"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        // Eli / sport
        m("eli-sport-1", "Baseus Eli Sport 1", &["eli sport", "eli sport 1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("eli-fit", "Baseus Eli Fit", &["eli fit"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("eli-10i-fit", "Baseus Eli 10i Fit", &["eli 10i", "10i fit"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        // AirNora / Bowie 30-35
        m("airnora-3", "Baseus AirNora 3", &["airnora 3"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws"),
        m("airnora-2", "Baseus AirNora 2", &["airnora 2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws"),
        m("airnora", "Baseus AirNora", &["airnora"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("bowie-35", "Baseus Bowie 35", &["bowie 35"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws"),
        m("bowie-30", "Baseus Bowie 30", &["bowie 30"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws"),
        m("aequr-g10", "Baseus AeQur G10", &["aequr g10", "g10"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        // Storm / C-Mic
        m("storm-1", "Baseus Storm 1", &["storm 1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("storm-3", "Baseus Storm 3", &["storm 3"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("cmic-cm10", "Baseus C-Mic CM10", &["c-mic cm10", "cm10", "c-mic"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        // Encok legacy
        m("encok-w04-pro", "Baseus Encok W04 Pro", &["encok w04 pro", "w04 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("encok-w04", "Baseus Encok W04", &["encok w04"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("encok-w05-lite", "Baseus Encok W05 Lite", &["encok w05", "w05 lite"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("encok-w11", "Baseus Encok W11", &["encok w11", "w11"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        m("encok-w12", "Baseus Encok W12", &["encok w12", "w12"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws"),
        // Inspire (app reviews mention; not always on Play list)
        m("inspire-xp1", "Baseus Inspire XP1", &["inspire xp1", "xp1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        m("inspire-xh1", "Baseus Inspire XH1", &["inspire xh1", "xh1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        m("inspire-xc1", "Baseus Inspire XC1", &["inspire xc1", "xc1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws"),
        // Headset (over-ear) — scan + experimental
        m("bowie-30-max", "Baseus Bowie 30 Max", &["bowie 30 max", "30 max"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset"),
        m("bowie-10-max", "Baseus Bowie 10 Max", &["bowie 10 max", "10 max"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset"),
        m("bowie-h2", "Baseus Bowie H2", &["bowie h2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset"),
        m("bowie-h1-pro", "Baseus Bowie H1 Pro", &["bowie h1 pro", "h1 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset"),
        m("bowie-h1s", "Baseus Bowie H1s", &["bowie h1s", "h1s"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset"),
        m("bowie-h1", "Baseus Bowie H1", &["bowie h1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset"),
        m("bowie-d05", "Baseus Bowie D05", &["bowie d05", "d05"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "headset"),
        m("aequr-gh02", "Baseus AeQur GH02", &["aequr gh02", "gh02"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "headset"),
        // Neckband
        m("baseus-p1-lite", "Baseus P1 Lite", &["p1 lite"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "neck"),
        m("baseus-p1x", "Baseus P1x", &["p1x"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "neck"),
        m("baseus-p1", "Baseus P1", &["baseus p1", " p1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "neck"),
        m("bowie-u2-pro", "Baseus Bowie U2 Pro", &["bowie u2 pro", "u2 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "neck"),
        m("bowie-u2", "Baseus Bowie U2", &["bowie u2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "neck"),
    ]
}

/// Match BLE advertising name → best model (longest pattern wins).
pub fn identify(ble_name: &str) -> Option<ModelInfo> {
    let lower = ble_name.to_lowercase();
    let mut best: Option<(usize, ModelInfo)> = None;
    for model in all_models() {
        for pat in &model.name_patterns {
            if lower.contains(pat.as_str()) {
                let score = pat.len();
                if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                    best = Some((score, model.clone()));
                }
            }
        }
    }
    best.map(|(_, m)| m)
}

/// Generic Baseus-looking name (for scan badge when not in catalog).
pub fn looks_like_baseus(ble_name: &str) -> bool {
    if identify(ble_name).is_some() {
        return true;
    }
    let lower = ble_name.to_lowercase();
    ["baseus", "bowie", "bass", "encok", "airgo", "airnora", "aequr", "eli ", "inspire", "storm"]
        .iter()
        .any(|k| lower.contains(k))
}

/// JSON-friendly list for frontend.
pub fn catalog_json() -> Vec<ModelInfo> {
    all_models()
}
