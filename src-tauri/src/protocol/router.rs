use super::{encode_command, AncMode, Bp1ProAnc, Command, DeviceProfile, ProtocolFamily};
use super::{EqBand, EqPreset, SpatialMode};

/// Route replies as well as writes; unknown families must never inherit BP1 decoding.
pub fn decode_frame(
    family: ProtocolFamily,
    frame: &super::Frame,
    last_anc: Option<AncMode>,
) -> Result<super::DeviceEvent, super::DecodeError> {
    match family {
        ProtocolFamily::Bp1Pro => Bp1ProAnc::decode_frame(frame, last_anc),
        ProtocolFamily::Unknown => Ok(super::DeviceEvent::Unknown {
            cmd: frame.cmd,
            payload: frame.payload.clone(),
        }),
    }
}

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
    SetCustomEq {
        dict_sort: u8,
        anc: bool,
        bands: Vec<EqBand>,
    },
    SetGameMode(bool),
    SetSpatial(SpatialMode),
    SetBassBoost(u8),
    SetLdac(bool),
    SetHearingProtection {
        enabled: bool,
        level: u8,
    },
    FindBuds(bool),
    SetGesture {
        layout: u8,
        left: Option<u8>,
        right: Option<u8>,
    },
    SetInEar(bool),
    SetMultipoint(bool),
    RestoreDefaults,
    SetAdaptiveLr(bool),
}

pub fn encode_feature(profile: &DeviceProfile, command: FeatureCommand) -> Result<Vec<u8>, String> {
    if profile.protocol == ProtocolFamily::Unknown {
        return Err("No protocol is verified for this model".into());
    }
    use crate::device::capability::{authorize, Feature};
    let feature = match &command {
        FeatureCommand::SetEq(_) | FeatureCommand::SetEqIndex(_) => Feature::Eq,
        FeatureCommand::SetCustomEq { .. } => Feature::CustomEq,
        FeatureCommand::SetGameMode(_) => Feature::Game,
        FeatureCommand::SetSpatial(_) => Feature::Spatial,
        FeatureCommand::SetBassBoost(_) => Feature::Bass,
        FeatureCommand::SetLdac(_) => Feature::Ldac,
        FeatureCommand::SetHearingProtection { .. } => Feature::Hearing,
        FeatureCommand::FindBuds(_) => Feature::Find,
        FeatureCommand::SetGesture { .. } => Feature::Gesture,
        FeatureCommand::SetInEar(_) => Feature::InEar,
        FeatureCommand::SetMultipoint(_) => Feature::Multipoint,
        FeatureCommand::RestoreDefaults => Feature::RestoreDefaults,
        FeatureCommand::SetAdaptiveLr(_) => Feature::AdaptiveLr,
    };
    authorize(profile, feature)?;
    if let FeatureCommand::SetCustomEq {
        bands,
        dict_sort,
        anc,
    } = &command
    {
        if profile.protocol != ProtocolFamily::Bp1Pro || *dict_sort != 101 || *anc {
            return Err("Custom EQ slot/ANC selector is not reviewed for this model".into());
        }
        if bands
            .iter()
            .any(|band| band.q_value != 1.0 || band.filter != 1)
        {
            return Err("BP1 Pro custom EQ requires Q=1 and peak filters".into());
        }
        let eq = profile
            .model_id
            .as_deref()
            .and_then(crate::catalog::profile_for)
            .and_then(|profile| profile.eq)
            .ok_or("No reviewed custom EQ schema")?;
        if bands.len() != eq.bands.len()
            || bands.iter().zip(&eq.bands).any(|(band, frequency)| {
                band.frequency != *frequency
                    || !band.q_value.is_finite()
                    || band.q_value <= 0.0
                    || !band.gain.is_finite()
                    || band.gain < eq.min_gain
                    || band.gain > eq.max_gain
            })
        {
            return Err("Custom EQ values do not match the reviewed model schema".into());
        }
    }
    // Reject values outside the protocol's reviewed range instead of silently
    // clamping intent into a different command.
    match &command {
        FeatureCommand::SetBassBoost(level) if *level > 1 => {
            return Err("Bass level is outside the current protocol range".into());
        }
        FeatureCommand::SetHearingProtection { level, .. } => {
            let hearing = profile
                .model_id
                .as_deref()
                .and_then(crate::catalog::profile_for)
                .and_then(|model| model.hearing)
                .ok_or("No reviewed hearing threshold schema")?;
            if !hearing.thresholds.contains(level)
                && !(*level == 0xFF && hearing.preserve_threshold_sentinel)
            {
                return Err("Hearing threshold is outside the reviewed model schema".into());
            }
        }
        FeatureCommand::SetGesture {
            layout,
            left,
            right,
        } => {
            let gesture = profile
                .model_id
                .as_deref()
                .and_then(crate::catalog::profile_for)
                .and_then(|model| model.gesture)
                .ok_or("No reviewed gesture schema")?;
            let entry = gesture
                .layouts
                .iter()
                .find(|entry| entry.layout == *layout)
                .ok_or("Gesture layout is not reviewed for this model")?;
            for function in [left, right].into_iter().flatten() {
                if !entry.functions.contains(function) {
                    return Err("Gesture function is not allowed for this layout".into());
                }
            }
        }
        FeatureCommand::SetInEar(_) => {
            profile
                .model_id
                .as_deref()
                .and_then(crate::catalog::profile_for)
                .and_then(|model| model.in_ear)
                .ok_or("No reviewed in-ear schema")?;
        }
        FeatureCommand::SetMultipoint(_) => {
            profile
                .model_id
                .as_deref()
                .and_then(crate::catalog::profile_for)
                .and_then(|model| model.multipoint)
                .ok_or("No reviewed multipoint schema")?;
        }
        FeatureCommand::RestoreDefaults => {
            profile
                .model_id
                .as_deref()
                .and_then(crate::catalog::profile_for)
                .and_then(|model| model.restore_defaults)
                .ok_or("No reviewed restore-defaults schema")?;
        }
        FeatureCommand::SetAdaptiveLr(_) => {
            profile
                .model_id
                .as_deref()
                .and_then(crate::catalog::profile_for)
                .and_then(|model| model.adaptive_lr)
                .ok_or("No reviewed adaptiveLr schema")?;
        }
        _ => {}
    }

    match (profile.protocol, command) {
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetEq(preset)) => {
            encode_profile_eq(profile, preset.to_byte())
        }
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetEqIndex(index)) => {
            encode_profile_eq(profile, index)
        }
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetGameMode(on)) => {
            Ok(Bp1ProAnc::cmd_set_game_mode(on))
        }
        (ProtocolFamily::Bp1Pro, FeatureCommand::FindBuds(start)) => {
            Ok(Bp1ProAnc::cmd_find_buds(start))
        }
        (
            ProtocolFamily::Bp1Pro,
            FeatureCommand::SetCustomEq {
                dict_sort,
                anc,
                bands,
            },
        ) => Ok(Bp1ProAnc::cmd_set_custom_eq(dict_sort, anc, &bands)),
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetSpatial(mode)) => {
            Ok(encode_command(Command::SetSpatial(mode)))
        }
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetBassBoost(level)) => {
            Ok(encode_command(Command::SetBassBoost(level)))
        }
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetLdac(enabled)) => {
            Ok(encode_command(Command::SetLdac(enabled)))
        }
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetHearingProtection { enabled, level }) => {
            Ok(encode_command(Command::SetHearingProtection {
                enabled,
                level,
            }))
        }
        (
            ProtocolFamily::Bp1Pro,
            FeatureCommand::SetGesture {
                layout,
                left,
                right,
            },
        ) => Ok(encode_command(Command::SetGesture {
            layout,
            left,
            right,
        })),
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetInEar(enabled)) => {
            Ok(encode_command(Command::SetInEar(enabled)))
        }
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetMultipoint(enabled)) => {
            Ok(encode_command(Command::SetMultipoint(enabled)))
        }
        (ProtocolFamily::Bp1Pro, FeatureCommand::RestoreDefaults) => {
            Ok(encode_command(Command::RestoreDefaults))
        }
        (ProtocolFamily::Bp1Pro, FeatureCommand::SetAdaptiveLr(enabled)) => {
            Ok(encode_command(Command::SetAdaptiveLr(enabled)))
        }
        (ProtocolFamily::Unknown, _) => Err("No protocol is verified for this model".into()),
    }
}

fn encode_profile_eq(profile: &DeviceProfile, index: u8) -> Result<Vec<u8>, String> {
    let eq = profile
        .model_id
        .as_deref()
        .and_then(crate::catalog::profile_for)
        .and_then(|profile| profile.eq)
        .ok_or("No model EQ schema")?;
    let preset = eq
        .presets
        .iter()
        .find(|preset| preset.dict_sort == index)
        .ok_or("Preset index is absent from the model schema")?;
    if preset.filters.is_empty() {
        return Err("Preset has no source-traced filter payload".into());
    }
    Ok(Bp1ProAnc::cmd_set_eq_filters(index, &preset.filters))
}

pub fn encode_listening(
    profile: &DeviceProfile,
    command: ListeningCommand,
) -> Result<Vec<u8>, String> {
    crate::device::capability::authorize(profile, crate::device::capability::Feature::Listening)?;
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
                return Err(format!(
                    "Custom ANC level {level} is not supported by this model"
                ));
            }
            (AncMode::Anc, level)
        }
        ListeningCommand::AdaptiveEnvironment(environment) => {
            if !profile.noise.supports_adaptive
                || !profile.noise.environments.contains(&environment)
            {
                return Err(format!(
                    "Adaptive environment {environment} is not supported by this model"
                ));
            }
            (
                AncMode::Anc,
                u8::try_from(environment).map_err(|_| "Invalid environment".to_string())?,
            )
        }
    };

    match profile.protocol {
        ProtocolFamily::Bp1Pro => Ok(Bp1ProAnc::cmd_set_noise(mode, parameter)),
        ProtocolFamily::Unknown => Err("No protocol is verified for this model".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::profile_for;

    #[test]
    fn custom_eq_rejects_wrong_layout_and_nonfinite_values_before_encoding() {
        let profile = profile_for(Some("bass-bp1-pro"), None, None);
        let bands: Vec<_> = crate::catalog::profile_for("bass-bp1-pro")
            .unwrap()
            .eq
            .unwrap()
            .bands
            .into_iter()
            .map(|frequency| EqBand {
                frequency,
                q_value: 1.0,
                gain: 0.0,
                filter: 1,
            })
            .collect();
        for invalid in 0..3 {
            let mut changed = bands.clone();
            match invalid {
                0 => changed[0].frequency = 1,
                1 => changed[0].gain = f32::NAN,
                _ => changed[0].q_value = 0.0,
            }
            assert!(encode_feature(
                &profile,
                FeatureCommand::SetCustomEq {
                    dict_sort: 101,
                    anc: false,
                    bands: changed,
                }
            )
            .is_err());
        }
    }

    #[test]
    fn notification_routing_preserves_bp1_and_keeps_unknown_raw() {
        let frame = super::super::Frame::decode_notify(&[0xAA, 0x23, 0x01]).unwrap();
        assert_eq!(
            decode_frame(ProtocolFamily::Bp1Pro, &frame, None).unwrap(),
            super::super::DeviceEvent::GameMode(true)
        );
        assert!(matches!(
            decode_frame(ProtocolFamily::Unknown, &frame, None).unwrap(),
            super::super::DeviceEvent::Unknown { cmd: 0x23, .. }
        ));
    }

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
            encode_feature(&profile, FeatureCommand::SetBassBoost(1)).unwrap(),
            vec![0xBA, 0x54, 0x01]
        );
        assert_eq!(
            encode_feature(&profile, FeatureCommand::FindBuds(true)).unwrap(),
            vec![0xBA, 0x10, 0x02, 0x01]
        );
    }

    #[test]
    fn unknown_profile_rejects_common_feature_commands() {
        let profile = profile_for(Some("unknown"), None, None);
        assert!(encode_feature(
            &profile,
            FeatureCommand::SetEq(crate::protocol::EqPreset::BassBoost)
        )
        .is_err());
    }

    #[test]
    fn bp1_custom_eq_uses_ba31_and_eight_band_payload() {
        let profile = profile_for(Some("bass-bp1-pro"), None, None);
        let bands = crate::catalog::profile_for("bass-bp1-pro")
            .unwrap()
            .eq
            .unwrap()
            .bands
            .into_iter()
            .map(|frequency| EqBand {
                frequency,
                q_value: 1.0,
                gain: 0.0,
                filter: 1,
            })
            .collect();
        let packet = encode_feature(
            &profile,
            FeatureCommand::SetCustomEq {
                dict_sort: 101,
                anc: false,
                bands,
            },
        )
        .unwrap();
        assert_eq!(&packet[..5], &[0xBA, 0x31, 0x65, 100, 0]);
        assert_eq!(packet.len(), 3 + (8 * 8));
    }
    #[test]
    fn bp1_custom_rejects_unreviewed_slot_selector_and_filter() {
        let profile = profile_for(Some("bass-bp1-pro"), None, None);
        let bands: Vec<_> = crate::catalog::profile_for("bass-bp1-pro")
            .unwrap()
            .eq
            .unwrap()
            .bands
            .into_iter()
            .map(|frequency| EqBand {
                frequency,
                q_value: 1.0,
                gain: 0.0,
                filter: 1,
            })
            .collect();
        for (dict_sort, anc) in [(100, false), (102, false), (101, true)] {
            assert!(encode_feature(
                &profile,
                FeatureCommand::SetCustomEq {
                    dict_sort,
                    anc,
                    bands: bands.clone(),
                }
            )
            .is_err());
        }
        let mut invalid = bands;
        invalid[0].filter = 2;
        assert!(encode_feature(
            &profile,
            FeatureCommand::SetCustomEq {
                dict_sort: 101,
                anc: false,
                bands: invalid,
            }
        )
        .is_err());
    }

    #[test]
    fn gesture_and_in_ear_require_capability_reviewed_schema_and_allowed_functions() {
        let mut profile = profile_for(Some("bass-bp1-pro"), None, None);
        // The reviewed profile enables the capabilities but marks them
        // experimental-only, so a user without the opt-in cannot dispatch.
        assert!(encode_feature(
            &profile,
            FeatureCommand::SetGesture {
                layout: 0,
                left: Some(1),
                right: Some(1),
            }
        )
        .is_err());
        assert!(encode_feature(&profile, FeatureCommand::SetInEar(true)).is_err());
        assert!(encode_feature(&profile, FeatureCommand::SetMultipoint(true)).is_err());
        assert!(encode_feature(&profile, FeatureCommand::RestoreDefaults).is_err());
        assert!(encode_feature(&profile, FeatureCommand::SetAdaptiveLr(true)).is_err());

        // Isolate the schema/allowlist behaviour from the Experimental gate.
        profile.experimental_features.clear();
        assert_eq!(
            encode_feature(
                &profile,
                FeatureCommand::SetGesture {
                    layout: 0,
                    left: Some(1),
                    right: Some(6),
                }
            )
            .unwrap(),
            vec![0xBA, 0x22, 0x00, 0x01, 0x06]
        );
        // Layout 4 (single press) is not in the reviewed BP1 Pro schema.
        assert!(encode_feature(
            &profile,
            FeatureCommand::SetGesture {
                layout: 4,
                left: Some(1),
                right: None,
            }
        )
        .is_err());
        // Single click (layout 3) only allows play/pause and none.
        assert!(encode_feature(
            &profile,
            FeatureCommand::SetGesture {
                layout: 3,
                left: Some(2),
                right: None,
            }
        )
        .is_err());
        assert_eq!(
            encode_feature(&profile, FeatureCommand::SetInEar(true)).unwrap(),
            vec![0xBA, 0x26, 0x01]
        );
        assert_eq!(
            encode_feature(&profile, FeatureCommand::SetMultipoint(true)).unwrap(),
            vec![0xBA, 0x58, 0x01]
        );
        assert_eq!(
            encode_feature(&profile, FeatureCommand::RestoreDefaults).unwrap(),
            vec![0xBA, 0x37]
        );
        assert_eq!(
            encode_feature(&profile, FeatureCommand::SetAdaptiveLr(true)).unwrap(),
            vec![0xBA, 0x4A, 0x01]
        );
    }
}
