use super::{encode_command, Command, DeviceProfile, ProtocolFamily, AncMode, Bp1ProAnc};
use super::{EqBand, EqPreset, SpatialMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListeningCommand {
    Normal,
    TransparencyFull,
    TransparencyVoice,
    CustomLevel(u8),
    AdaptiveEnvironment(u16),
}

#[derive(Debug, Clone)]
pub enum FeatureCommand {
    SetEq(EqPreset),
    SetEqIndex(u8),
    SetCustomEq { dict_sort: u8, anc: bool, bands: Vec<EqBand> },
    SetGameMode(bool),
    SetSpatial(SpatialMode),
    SetBassBoost(u8),
    SetLdac(bool),
    SetHearingProtection { enabled: bool, level: u8 },
    FindBuds(bool),
}

pub fn encode_feature(profile: &DeviceProfile, command: FeatureCommand) -> Result<Vec<u8>, String> {
    if profile.protocol == ProtocolFamily::Unknown {
        return Err("No protocol is verified for this model".into());
    }

    match (profile.protocol, command) {
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetEq(preset)) => Ok(Bp1ProAnc::cmd_set_eq(preset)),
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetEqIndex(index)) => Ok(encode_command(Command::SetEqIndex(index))),
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetGameMode(on)) => Ok(Bp1ProAnc::cmd_set_game_mode(on)),
        (ProtocolFamily::Bp1Pro, FeatureCommand::FindBuds(start)) => Ok(Bp1ProAnc::cmd_find_buds(start)),
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetCustomEq { dict_sort, anc, bands }) => {
            Ok(Bp1ProAnc::cmd_set_custom_eq(dict_sort, anc, &bands))
        }
        (_, FeatureCommand::SetEq(preset)) => Ok(encode_command(Command::SetEq(preset))),
        (_, FeatureCommand::SetEqIndex(index)) => Ok(encode_command(Command::SetEqIndex(index))),
        (_, FeatureCommand::SetGameMode(on)) => Ok(encode_command(Command::SetGameMode(on))),
        (_, FeatureCommand::SetSpatial(mode)) => Ok(encode_command(Command::SetSpatial(mode))),
        (_, FeatureCommand::SetBassBoost(level)) => Ok(encode_command(Command::SetBassBoost(level))),
        (_, FeatureCommand::SetLdac(enabled)) => Ok(encode_command(Command::SetLdac(enabled))),
        (_, FeatureCommand::SetHearingProtection { enabled, level }) => {
            Ok(encode_command(Command::SetHearingProtection { enabled, level }))
        }
        (_, FeatureCommand::FindBuds(start)) => Ok(encode_command(Command::FindBuds(start))),
        (_, FeatureCommand::SetCustomEq { .. }) => Err("Custom EQ protocol is not verified for this model".into()),
    }
}

pub fn encode_listening(profile: &DeviceProfile, command: ListeningCommand) -> Result<Vec<u8>, String> {
    let (mode, parameter) = match command {
        ListeningCommand::Normal => (AncMode::Off, 0xFF),
        ListeningCommand::TransparencyFull => (AncMode::Transparency, 0xFF),
        ListeningCommand::TransparencyVoice => {
            if !profile.noise.supports_transparency_voice {
                return Err("Transparency voice mode is not supported by this model".into());
            }
            (AncMode::Transparency, 0x01)
        }
        ListeningCommand::CustomLevel(level) => {
            let max = profile.noise.max_custom_level;
            if max == 0 || level == 0 || level > max {
                return Err(format!("Custom ANC level {level} is not supported by this model"));
            }
            (AncMode::Anc, level)
        }
        ListeningCommand::AdaptiveEnvironment(environment) => {
            if !profile.noise.supports_adaptive
                || !profile.noise.environments.contains(&environment)
            {
                return Err(format!("Adaptive environment {environment} is not supported by this model"));
            }
            (AncMode::Anc, u8::try_from(environment).map_err(|_| "Invalid environment".to_string())?)
        }
    };

    match profile.protocol {
        ProtocolFamily::Bp1Pro => Ok(Bp1ProAnc::cmd_set_noise(mode, parameter)),
        ProtocolFamily::BaseusAaBaExperimental => Ok(encode_command(Command::SetNoise { mode, parameter })),
        ProtocolFamily::Unknown => Err("No protocol is verified for this model".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::profile_for;

    #[test]
    fn bp1_profile_routes_apk_semantics_to_ba34() {
        let profile = profile_for(Some("bass-bp1-pro"), None, None);
        assert_eq!(
            encode_listening(&profile, ListeningCommand::Normal).unwrap(),
            vec![0xBA, 0x34, 0x00, 0xFF]
        );
        assert_eq!(
            encode_listening(&profile, ListeningCommand::AdaptiveEnvironment(108)).unwrap(),
            vec![0xBA, 0x34, 0x01, 0x6C]
        );
    }

    #[test]
    fn lite_profile_rejects_level_four() {
        let profile = profile_for(Some("eh10-nc-lite"), None, None);
        assert!(encode_listening(&profile, ListeningCommand::CustomLevel(4)).is_err());
    }

    #[test]
    fn unknown_profile_never_falls_back_to_bp1() {
        let profile = profile_for(Some("unknown"), None, None);
        assert!(encode_listening(&profile, ListeningCommand::Normal).is_err());
    }

    #[test]
    fn bp1_profile_routes_common_feature_commands() {
        let profile = profile_for(Some("bass-bp1-pro"), None, None);
        assert_eq!(
            encode_feature(&profile, FeatureCommand::SetBassBoost(2)).unwrap(),
            vec![0xBA, 0x54, 0x01, 0x02]
        );
        assert_eq!(
            encode_feature(&profile, FeatureCommand::FindBuds(true)).unwrap(),
            vec![0xBA, 0x10, 0x02, 0x01]
        );
    }

    #[test]
    fn unknown_profile_rejects_common_feature_commands() {
        let profile = profile_for(Some("unknown"), None, None);
        assert!(encode_feature(&profile, FeatureCommand::SetEq(crate::protocol::EqPreset::BassBoost)).is_err());
    }

    #[test]
    fn bp1_custom_eq_uses_ba31_and_eight_band_payload() {
        let profile = profile_for(Some("bass-bp1-pro"), None, None);
        let bands = vec![EqBand { frequency: 100, q_value: 1.0, gain: 0.0, filter: 1 }; 8];
        let packet = encode_feature(&profile, FeatureCommand::SetCustomEq { dict_sort: 101, anc: false, bands }).unwrap();
        assert_eq!(&packet[..4], &[0xBA, 0x31, 0x65, 0x00]);
        assert_eq!(packet.len(), 4 + (8 * 8));
    }
}
