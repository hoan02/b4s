//! Reviewed value and feature schemas; independent of packet encoding.
use super::router::FeatureCommand;
use super::{DeviceProfile, ProtocolFamily};

pub(super) fn validate_feature(
    profile: &DeviceProfile,
    command: &FeatureCommand,
) -> Result<(), String> {
    if let FeatureCommand::SetCustomEq {
        bands,
        dict_sort,
        anc,
    } = command
    {
        let eq = profile
            .model_id
            .as_deref()
            .and_then(crate::catalog::profile_for)
            .and_then(|profile| profile.eq)
            .ok_or("No reviewed custom EQ schema")?;
        if profile.protocol != ProtocolFamily::Bp1Pro
            || *dict_sort != eq.custom_write.slot
            || *anc != eq.custom_write.anc_bank
        {
            return Err("Custom EQ slot/ANC selector is not reviewed for this model".into());
        }
        if bands.len() != eq.bands.len()
            || bands
                .iter()
                .zip(&eq.bands)
                .enumerate()
                .any(|(index, (band, frequency))| {
                    band.frequency != *frequency
                        || band.q_value != eq.q_values[index]
                        || band.filter != 1
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
    match command {
        FeatureCommand::SetBassBoost(level) => {
            let sound = profile.sound.as_ref().ok_or("No reviewed sound schema")?;
            if *level > sound.max_bass_level {
                return Err("Bass level is outside the reviewed model range".into());
            }
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
        FeatureCommand::SetWindNoise(_) => {
            profile
                .model_id
                .as_deref()
                .and_then(crate::catalog::profile_for)
                .and_then(|model| model.wind_noise)
                .ok_or("No reviewed windNoise schema")?;
        }
        _ => {}
    }

    Ok(())
}
