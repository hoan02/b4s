//! Baseus earbud model registry
//!
//! Auto-derived from official app com.baseus.intelligent 2.14.1 DEX strings.
//! Focus: listening devices only (TWS / open-ear / headset / neck).
//!
//! SupportLevel:
//!   Verified     — packet table confirmed on real hardware
//!   Experimental — listed in official app; try BP1-compatible GATT/framing
//!   ScanOnly     — recognised in scan UI only

use serde::{Deserialize, Serialize};

use super::advertisement::BASEUS_SERVICE_UUID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SupportLevel {
    Verified,
    Experimental,
    ScanOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtocolFamily {
    /// Packet table verified on Bass BP1 Pro / Ultra hardware.
    Bp1Pro,
    /// Best-effort shared Baseus AA/BA implementation; not hardware-verified for this model.
    BaseusAaBaExperimental,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilities {
    pub anc: bool,
    pub eq: bool,
    pub game_mode: bool,
    pub bass_boost: bool,
    pub ldac: bool,
    pub hearing_protection: bool,
    pub spatial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BleTransportConfig {
    pub service_uuid: Option<String>,
    pub write_uuid: Option<String>,
    pub notify_uuid: Option<String>,
    pub use_self_uuid: bool,
    pub required_advertised_service: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub name_patterns: Vec<String>,
    pub support: SupportLevel,
    pub protocol: ProtocolFamily,
    pub has_anc: bool,
    pub has_eq: bool,
    pub has_game_mode: bool,
    pub category: String,
    /// UI grouping (from official app families)
    pub group: String,
    pub capabilities: ModelCapabilities,
    pub transport: BleTransportConfig,
    pub color_variants: Vec<String>,
    pub image_url: Option<String>,
    pub image_provenance: String,
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
    group: &str,
) -> ModelInfo {
    let protocol = match (support, protocol) {
        (SupportLevel::Verified, protocol) => protocol,
        (_, ProtocolFamily::Bp1Pro) => ProtocolFamily::BaseusAaBaExperimental,
        (_, protocol) => protocol,
    };

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
        group: group.into(),
        capabilities: ModelCapabilities {
            anc,
            eq,
            game_mode: game,
            bass_boost: eq,
            ldac: false,
            hearing_protection: false,
            spatial: eq,
        },
        transport: BleTransportConfig {
            service_uuid: Some(BASEUS_SERVICE_UUID.into()),
            write_uuid: Some("EE684B1A-1E9B-ED3E-EE55-F894667E92AC".into()),
            notify_uuid: Some("654B749C-E37A-AE1F-EBAB-40CA133E3690".into()),
            use_self_uuid: false,
            required_advertised_service: false,
        },
        color_variants: Vec::new(),
        image_url: None,
        image_provenance: "fallback".into(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoiseCapability {
    pub supports_adaptive: bool,
    pub environments: Vec<u16>,
    pub max_custom_level: u8,
    pub supports_transparency_voice: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceProfile {
    pub model_id: Option<String>,
    pub model_name: Option<String>,
    pub firmware: Option<String>,
    pub protocol: ProtocolFamily,
    pub verified: bool,
    pub noise: NoiseCapability,
}

pub fn unknown_profile(model_id: Option<&str>, model_name: Option<&str>) -> DeviceProfile {
    DeviceProfile {
        model_id: model_id.map(str::to_owned),
        model_name: model_name.map(str::to_owned),
        firmware: None,
        protocol: ProtocolFamily::Unknown,
        verified: false,
        noise: NoiseCapability {
            supports_adaptive: false,
            environments: Vec::new(),
            max_custom_level: 0,
            supports_transparency_voice: false,
        },
    }
}

pub fn profile_for(
    model_id: Option<&str>,
    model_name: Option<&str>,
    firmware: Option<&str>,
) -> DeviceProfile {
    let Some(id) = model_id else {
        return unknown_profile(model_id, model_name);
    };
    let Some(model) = all_models().into_iter().find(|item| item.id == id) else {
        return unknown_profile(model_id, model_name);
    };
    let max_custom_level = if !model.has_anc {
        0
    } else if id == "eh10-nc-lite" || id == "bh1-nc-lite" {
        3
    } else {
        5
    };
    DeviceProfile {
        model_id: Some(model.id),
        model_name: Some(model.display_name),
        firmware: firmware.map(str::to_owned),
        protocol: model.protocol,
        verified: matches!(model.support, SupportLevel::Verified),
        noise: NoiseCapability {
            supports_adaptive: model.has_anc,
            environments: if model.has_anc {
                vec![101, 102, 103, 108]
            } else {
                Vec::new()
            },
            max_custom_level,
            supports_transparency_voice: model.has_anc,
        },
    }
}

/// Full listening catalog from Baseus app 2.14.1.
pub fn all_models() -> Vec<ModelInfo> {
    vec![
        m("bass-bp1-pro", "Baseus Bass BP1 Pro", &["bass bp1 pro", "bp1 pro"], SupportLevel::Verified, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bass BP1 / EP10"),
        m("bass-bp1-ultra", "Baseus Bass BP1 Ultra", &["bass bp1 ultra", "bp1 ultra"], SupportLevel::Verified, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bass BP1 / EP10"),
        m("bass-bp1-nc", "Baseus Bass BP1 NC", &["bass bp1 nc", "bp1 nc"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bass BP1 / EP10"),
        m("bass-ep10-nc", "Baseus Bass EP10 NC", &["bass ep10 nc", "ep10 nc"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bass BP1 / EP10"),
        m("bass-ep10-pro", "Baseus Bass EP10 Pro", &["bass ep10 pro", "ep10 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bass BP1 / EP10"),
        m("bass-ep10-ultra", "Baseus Bass EP10 Ultra", &["bass ep10 ultra", "ep10 ultra"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bass BP1 / EP10"),
        m("bowie-ma10", "Baseus Bowie MA10", &["bowie ma10", "ma10"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie MA series"),
        m("bowie-ma10-pro", "Baseus Bowie MA10 Pro", &["bowie ma10 pro", "ma10 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie MA series"),
        m("bowie-ma10s", "Baseus Bowie MA10s", &["bowie ma10s", "ma10s"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie MA series"),
        m("bowie-ma20", "Baseus Bowie MA20", &["bowie ma20", "ma20"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie MA series"),
        m("bowie-ma20-pro", "Baseus Bowie MA20 Pro", &["bowie ma20 pro", "ma20 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie MA series"),
        m("bowie-m1", "Baseus Bowie M1", &["bowie m1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie M series"),
        m("bowie-m2", "Baseus Bowie M2", &["bowie m2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie M series"),
        m("bowie-m2-plus", "Baseus Bowie M2+", &["bowie m2+", "m2+"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie M series"),
        m("bowie-m2s", "Baseus Bowie M2s", &["bowie m2s", "m2s"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie M series"),
        m("bowie-m2s-pro", "Baseus Bowie M2s Pro", &["bowie m2s pro", "m2s pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie M series"),
        m("bowie-m3", "Baseus Bowie M3", &["bowie m3"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie M series"),
        m("bowie-m3s", "Baseus Bowie M3s", &["bowie m3s", "m3s"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie M series"),
        m("bowie-m4s", "Baseus Bowie M4s", &["bowie m4s", "m4s"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie M series"),
        m("m2s-ultra", "Baseus M2s Ultra", &["m2s ultra"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie M series"),
        m("bass-e12x", "Baseus Bass E12x", &["bass e12x", "e12x"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, true, "tws", "Bowie E series"),
        m("bass-e19s", "Baseus Bass E19s", &["bass e19s", "e19s"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie E series"),
        m("bowie-e10", "Baseus Bowie E10", &["bowie e10", "e10"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie E series"),
        m("bowie-e12", "Baseus Bowie E12", &["bowie e12", "e12"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, true, "tws", "Bowie E series"),
        m("bowie-e13", "Baseus Bowie E13", &["bowie e13", "e13"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, true, "tws", "Bowie E series"),
        m("bowie-e2", "Baseus Bowie E2", &["bowie e2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie E series"),
        m("bowie-e3", "Baseus Bowie E3", &["bowie e3"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, true, "tws", "Bowie E series"),
        m("bowie-e3-2025", "Baseus Bowie E3 2025", &["bowie e3 2025", "e3 2025"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, true, "tws", "Bowie E series"),
        m("bowie-e5", "Baseus Bowie E5", &["bowie e5"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie E series"),
        m("bowie-e5x", "Baseus Bowie E5x", &["bowie e5x", "e5x"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie E series"),
        m("bowie-e8", "Baseus Bowie E8", &["bowie e8"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie E series"),
        m("e9", "Baseus E9", &["e9"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie E series"),
        m("inspire-xc1", "Baseus Inspire XC1", &["inspire xc1", "xc1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Inspire"),
        m("inspire-xh1", "Baseus Inspire XH1", &["inspire xh1", "xh1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "headset", "Inspire"),
        m("inspire-xp1", "Baseus Inspire XP1", &["inspire xp1", "xp1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Inspire"),
        m("as01", "Baseus AS01", &["as01"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "open", "Open-ear"),
        m("as01-air", "Baseus AS01 Air", &["as01 air"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "open", "Open-ear"),
        m("airgo-1-ring", "Baseus AirGo 1 Ring", &["airgo 1 ring", "1 ring"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "open", "Open-ear"),
        m("airgo-ag20", "Baseus AirGo AG20", &["airgo ag20", "ag20"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "open", "Open-ear"),
        m("airgo-as01", "Baseus AirGo AS01", &["airgo as01", "as01"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "open", "Open-ear"),
        m("bowie-mc1", "Baseus Bowie MC1", &["bowie mc1", "mc1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "open", "Open-ear"),
        m("bowie-mc1-pro", "Baseus Bowie MC1 Pro", &["bowie mc1 pro", "mc1 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "open", "Open-ear"),
        m("bowie-mc2", "Baseus Bowie MC2", &["bowie mc2", "mc2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "open", "Open-ear"),
        m("bowie-mc2-air", "Baseus Bowie MC2 Air", &["bowie mc2 air", "mc2 air"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "open", "Open-ear"),
        m("bowie-mc2-nc", "Baseus Bowie MC2 NC", &["bowie mc2 nc", "mc2 nc"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "open", "Open-ear"),
        m("bowie-mc2-s", "Baseus Bowie MC2 S", &["bowie mc2 s", "mc2 s"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "open", "Open-ear"),
        m("bowie-mf1", "Baseus Bowie MF1", &["bowie mf1", "mf1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "open", "Open-ear"),
        m("bass-bc1-lite", "Baseus Bass BC1 Lite", &["bass bc1 lite", "bc1 lite"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bass line"),
        m("bass-bc1", "Baseus Bass BC1 星光版", &["bass bc1", "bc1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bass line"),
        m("bass-bc2", "Baseus Bass BC2", &["bass bc2", "bc2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bass line"),
        m("bass-bd1", "Baseus Bass BD1", &["bass bd1", "bd1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws", "Bass line"),
        m("bass-bf1", "Baseus Bass BF1", &["bass bf1", "bf1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bass line"),
        m("bass-bf1-lite", "Baseus Bass BF1 Lite", &["bass bf1 lite", "bf1 lite"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bass line"),
        m("bass-bs1", "Baseus Bass BS1", &["bass bs1", "bs1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bass line"),
        m("bass-bs1-lite", "Baseus Bass BS1 Lite", &["bass bs1 lite", "bs1 lite"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bass line"),
        m("bass-bs1-nc", "Baseus Bass BS1 NC", &["bass bs1 nc", "bs1 nc"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws", "Bass line"),
        m("bass-bs2-lite", "Baseus Bass BS2 Lite", &["bass bs2 lite", "bs2 lite"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bass line"),
        m("storm-1", "Baseus Storm 1", &["storm 1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bass line"),
        m("storm-3", "Baseus Storm 3", &["storm 3"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bass line"),
        m("storm-5", "Baseus Storm 5", &["storm 5"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bass line"),
        m("bass-w04", "Baseus Bass W04", &["bass w04", "w04"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("bass-wm01s", "Baseus Bass WM01s", &["bass wm01s", "wm01s"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("bass-wm02s", "Baseus Bass WM02s", &["bass wm02s", "wm02s"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("bowie-ez10", "Baseus Bowie EZ10", &["bowie ez10", "ez10"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("bowie-mz10", "Baseus Bowie MZ10", &["bowie mz10", "mz10"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("bowie-w04", "Baseus Bowie W04", &["bowie w04", "w04"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("bowie-w04-plus", "Baseus Bowie W04 Plus", &["bowie w04 plus", "w04 plus"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("bowie-w04-pro", "Baseus Bowie W04 Pro", &["bowie w04 pro", "w04 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie W / WM"),
        m("bowie-wm01", "Baseus Bowie WM01", &["bowie wm01", "wm01"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("bowie-wm01-plus", "Baseus Bowie WM01 Plus", &["bowie wm01 plus", "wm01 plus"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("bowie-wm03", "Baseus Bowie WM03", &["bowie wm03", "wm03"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("bowie-wm05", "Baseus Bowie WM05", &["bowie wm05", "wm05"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("bowie-wx5-pro", "Baseus Bowie WX5 Pro", &["bowie wx5 pro", "wx5 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie W / WM"),
        m("encok-wm01", "Baseus Encok WM01", &["encok wm01", "wm01"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("w04-pro", "Baseus W04 Pro", &["w04 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Bowie W / WM"),
        m("wm02", "Baseus WM02", &["wm02"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("wm02-plus", "Baseus WM02+", &["wm02+"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Bowie W / WM"),
        m("airnora", "Baseus AirNora", &["airnora"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws", "AirNora"),
        m("airnora-2", "Baseus AirNora 2", &["airnora 2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws", "AirNora"),
        m("airnora-3", "Baseus AirNora 3", &["airnora 3"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws", "AirNora"),
        m("eli-10i-fit", "Baseus Eli 10i Fit", &["eli 10i fit", "10i fit"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Eli sport"),
        m("eli-15i-fit", "Baseus Eli 15i Fit", &["eli 15i fit", "15i fit"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Eli sport"),
        m("eli-1i-fit", "Baseus Eli 1i Fit", &["eli 1i fit", "1i fit"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Eli sport"),
        m("eli-fit", "Baseus Eli Fit", &["eli fit", "fit"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "open", "Eli sport"),
        m("eli-sport-1", "Baseus Eli Sport 1", &["eli sport 1", "sport 1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "open", "Eli sport"),
        m("aequr-gh02", "Baseus AeQur GH02", &["aequr gh02", "gh02"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "headset", "Headset"),
        m("bh1-nc-lite", "Baseus BH1 NC Lite", &["bh1 nc lite"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("bass-bh1", "Baseus Bass BH1", &["bass bh1", "bh1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("bass-bh1-air", "Baseus Bass BH1 Air", &["bass bh1 air", "bh1 air"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "headset", "Headset"),
        m("bass-bh1-lite", "Baseus Bass BH1 Lite", &["bass bh1 lite", "bh1 lite"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("bass-bh1-nc", "Baseus Bass BH1 NC", &["bass bh1 nc", "bh1 nc"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("bass-eh10-nc", "Baseus Bass EH10 NC", &["bass eh10 nc", "eh10 nc"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("bowie-10-max", "Baseus Bowie 10 Max", &["bowie 10 max", "10 max"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "headset", "Headset"),
        m("bowie-30-max", "Baseus Bowie 30 Max", &["bowie 30 max", "30 max"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("bowie-35-max", "Baseus Bowie 35 Max", &["bowie 35 max", "35 max"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("bowie-d05", "Baseus Bowie D05", &["bowie d05", "d05"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "headset", "Headset"),
        m("bowie-h1", "Baseus Bowie H1", &["bowie h1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("bowie-h1-pro", "Baseus Bowie H1 Pro", &["bowie h1 pro", "h1 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "headset", "Headset"),
        m("bowie-h1s", "Baseus Bowie H1S", &["bowie h1s", "h1s"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("bowie-h1i", "Baseus Bowie H1i", &["bowie h1i", "h1i"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("bowie-h1s-pro", "Baseus Bowie H1s Pro", &["bowie h1s pro", "h1s pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "headset", "Headset"),
        m("bowie-h2", "Baseus Bowie H2", &["bowie h2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("bowie-mh1", "Baseus Bowie MH1", &["bowie mh1", "mh1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("eh10-nc-lite", "Baseus EH10 NC Lite", &["eh10 nc lite"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "headset", "Headset"),
        m("bowie-p1", "Baseus Bowie P1", &["bowie p1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "neck", "Neckband"),
        m("bowie-u2", "Baseus Bowie U2", &["bowie u2"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "neck", "Neckband"),
        m("bowie-u2-pro", "Baseus Bowie U2 Pro", &["bowie u2 pro", "u2 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "neck", "Neckband"),
        m("p1", "Baseus P1", &["p1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "neck", "Neckband"),
        m("p1-lite", "Baseus P1 Lite", &["p1 lite"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "neck", "Neckband"),
        m("p1x", "Baseus P1x", &["p1x"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "neck", "Neckband"),
        m("aequr-30-air", "Baseus AeQur 30 Air", &["aequr 30 air", "30 air"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws", "AeQur"),
        m("aequr-ds10", "Baseus AeQur DS10", &["aequr ds10", "ds10"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "AeQur"),
        m("aequr-g10", "Baseus AeQur G10", &["aequr g10", "g10"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "AeQur"),
        m("aequr-n10", "Baseus AeQur N10", &["aequr n10", "n10"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "AeQur"),
        m("aequr-vo20", "Baseus AeQur VO20", &["aequr vo20", "vo20"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "AeQur"),
        m("bass-1-plus", "Baseus Bass 1+", &["bass 1+"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws", "Other"),
        m("bowie-30", "Baseus Bowie 30", &["bowie 30"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws", "Other"),
        m("bowie-35", "Baseus Bowie 35", &["bowie 35"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, false, "tws", "Other"),
        m("bowie-mp1", "Baseus Bowie MP1", &["bowie mp1", "mp1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Other"),
        m("bowie-ms1", "Baseus Bowie MS1", &["bowie ms1", "ms1"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Other"),
        m("ef8", "Baseus EF8", &["ef8"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Other"),
        m("ex", "Baseus EX", &["ex"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Other"),
        m("t2-pro", "Baseus T2 Pro", &["t2 pro"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, true, true, true, "tws", "Other"),
        m("w05-lite", "Baseus W05 Lite", &["w05 lite"], SupportLevel::Experimental, ProtocolFamily::Bp1Pro, false, true, false, "tws", "Other"),
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

pub fn looks_like_baseus(ble_name: &str) -> bool {
    if identify(ble_name).is_some() {
        return true;
    }
    let lower = ble_name.to_lowercase();
    ["baseus", "bowie", "bass", "encok", "airgo", "airnora", "aequr", "eli ", "inspire", "storm"]
        .iter()
        .any(|k| lower.contains(k))
}

pub fn catalog_json() -> Vec<ModelInfo> {
    all_models()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bp1_profile_is_verified_and_has_apk_noise_capabilities() {
        let profile = profile_for(Some("bass-bp1-pro"), None, Some("1.0.0"));
        assert!(profile.verified);
        assert_eq!(profile.protocol, ProtocolFamily::Bp1Pro);
        assert_eq!(profile.noise.max_custom_level, 5);
        assert_eq!(profile.noise.environments, vec![101, 102, 103, 108]);
    }

    #[test]
    fn lite_profile_caps_custom_anc_at_three() {
        let profile = profile_for(Some("eh10-nc-lite"), None, None);
        assert_eq!(profile.noise.max_custom_level, 3);
    }

    #[test]
    fn non_anc_profile_does_not_expose_noise_controls() {
        let profile = profile_for(Some("bowie-m1"), None, None);
        assert!(!profile.noise.supports_adaptive);
        assert_eq!(profile.noise.max_custom_level, 0);
        assert!(profile.noise.environments.is_empty());
    }

    #[test]
    fn unknown_profile_is_safe_and_unverified() {
        let profile = profile_for(Some("unknown"), Some("Baseus Unknown"), None);
        assert!(!profile.verified);
        assert_eq!(profile.protocol, ProtocolFamily::Unknown);
        assert_eq!(profile.noise.max_custom_level, 0);
    }
}

