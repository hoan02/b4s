use super::{ControlTransport, ModelProfile, WireFraming};

pub(super) fn validate_profiles(profiles: &[ModelProfile]) -> Result<(), String> {
    let mut ids = std::collections::HashSet::new();
    for profile in profiles {
        if profile.id.starts_with("server-")
            || profile.id.starts_with('-')
            || profile.id.ends_with('-')
            || profile.id.contains("--")
            || !profile
                .id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(format!(
                "model ID must be canonical lowercase kebab-case: {}",
                profile.id
            ));
        }
        validate_feature_contracts(profile)?;
        if profile.capabilities.hearing_protection && profile.hearing.is_none() {
            return Err(format!("missing hearing constraints in {}", profile.id));
        }
        if let Some(hearing) = &profile.hearing {
            let unique: std::collections::HashSet<_> = hearing.thresholds.iter().collect();
            if hearing.provenance.trim().is_empty()
                || hearing.thresholds.is_empty()
                || unique.len() != hearing.thresholds.len()
                || hearing
                    .thresholds
                    .iter()
                    .any(|value| ![75, 80, 85, 90, 95, 100].contains(value))
            {
                return Err(format!("invalid hearing constraints in {}", profile.id));
            }
        }
        if profile.capabilities.gesture && profile.gesture.is_none() {
            return Err(format!("missing gesture constraints in {}", profile.id));
        }
        if let Some(gesture) = &profile.gesture {
            if gesture.provenance.trim().is_empty() || gesture.layouts.is_empty() {
                return Err(format!("invalid gesture constraints in {}", profile.id));
            }
            let mut layouts = std::collections::HashSet::new();
            for layout in &gesture.layouts {
                if layout.layout > 5 || !layouts.insert(layout.layout) {
                    return Err(format!("invalid gesture layout in {}", profile.id));
                }
                if layout.functions.is_empty()
                    || layout
                        .functions
                        .iter()
                        .any(|function| !matches!(function, 0..=19 | 27 | 28))
                {
                    return Err(format!("invalid gesture functions in {}", profile.id));
                }
            }
        }
        if profile.capabilities.in_ear && profile.in_ear.is_none() {
            return Err(format!("missing in-ear provenance in {}", profile.id));
        }
        if let Some(in_ear) = &profile.in_ear {
            if in_ear.provenance.trim().is_empty() {
                return Err(format!("invalid in-ear provenance in {}", profile.id));
            }
        }
        if profile.capabilities.multipoint && profile.multipoint.is_none() {
            return Err(format!("missing multipoint provenance in {}", profile.id));
        }
        if let Some(multipoint) = &profile.multipoint {
            if multipoint.provenance.trim().is_empty() {
                return Err(format!("invalid multipoint provenance in {}", profile.id));
            }
        }
        if profile.capabilities.restore_defaults && profile.restore_defaults.is_none() {
            return Err(format!(
                "missing restore-defaults provenance in {}",
                profile.id
            ));
        }
        if let Some(restore) = &profile.restore_defaults {
            if restore.provenance.trim().is_empty() {
                return Err(format!(
                    "invalid restore-defaults provenance in {}",
                    profile.id
                ));
            }
        }
        if profile.capabilities.adaptive_lr && profile.adaptive_lr.is_none() {
            return Err(format!("missing adaptiveLr provenance in {}", profile.id));
        }
        if let Some(adaptive) = &profile.adaptive_lr {
            if adaptive.provenance.trim().is_empty() {
                return Err(format!("invalid adaptiveLr provenance in {}", profile.id));
            }
        }
        if profile.capabilities.wind_noise && profile.wind_noise.is_none() {
            return Err(format!("missing windNoise provenance in {}", profile.id));
        }
        if let Some(wind) = &profile.wind_noise {
            if wind.provenance.trim().is_empty() {
                return Err(format!("invalid windNoise provenance in {}", profile.id));
            }
        }
        let mut experimental = std::collections::HashSet::new();
        for feature in &profile.experimental_features {
            let capability_enabled = match feature.as_str() {
                "gesture" => profile.capabilities.gesture && profile.gesture.is_some(),
                "inEar" => profile.capabilities.in_ear && profile.in_ear.is_some(),
                "multipoint" => profile.capabilities.multipoint && profile.multipoint.is_some(),
                "restoreDefaults" => {
                    profile.capabilities.restore_defaults && profile.restore_defaults.is_some()
                }
                "adaptiveLr" => profile.capabilities.adaptive_lr && profile.adaptive_lr.is_some(),
                "windNoise" => profile.capabilities.wind_noise && profile.wind_noise.is_some(),
                _ => false,
            };
            if !capability_enabled || !experimental.insert(feature) {
                return Err(format!(
                    "invalid experimental feature {feature} in {}",
                    profile.id
                ));
            }
        }
        if profile.schema_version != 3 {
            return Err(format!("unsupported profile schema in {}", profile.id));
        }
        if profile.connection.is_none() {
            return Err(format!("missing connection profile in {}", profile.id));
        }
        if let Some(connection) = &profile.connection {
            if connection.provenance.trim().is_empty() {
                return Err(format!("missing transport provenance in {}", profile.id));
            }
            for uuid in [
                &connection.service_uuid,
                &connection.write_uuid,
                &connection.notify_uuid,
            ]
            .into_iter()
            .flatten()
            {
                uuid::Uuid::parse_str(uuid)
                    .map_err(|_| format!("invalid transport UUID in {}", profile.id))?;
            }
            if connection.transport == ControlTransport::BleGatt
                && (connection.service_uuid.is_none()
                    || connection.write_uuid.is_none()
                    || connection.notify_uuid.is_none()
                    || connection.framing == WireFraming::Unresolved)
            {
                return Err(format!(
                    "incomplete BLE connection profile in {}",
                    profile.id
                ));
            }
            if connection.transport == ControlTransport::Unresolved
                && (profile.support != "scanOnly"
                    || !connection.handshake.is_empty()
                    || connection.init_state_query)
            {
                return Err(format!(
                    "unresolved transport must remain passive in {}",
                    profile.id
                ));
            }
        }
        if !ids.insert(profile.id.clone()) {
            return Err(format!("duplicate model profile: {}", profile.id));
        }
        if !matches!(
            profile.protocol_family.as_str(),
            "bp1" | "bp1Ultra" | "unknown"
        ) {
            return Err(format!("unknown protocol family in {}", profile.id));
        }
        if !matches!(
            profile.support.as_str(),
            "verified" | "experimental" | "scanOnly"
        ) {
            return Err(format!("invalid support level in {}", profile.id));
        }
        if profile.id.is_empty() || profile.aliases.is_empty() {
            return Err("profiles must define an id and aliases".into());
        }
        if let Some(eq) = &profile.eq {
            let unique_bands: std::collections::HashSet<_> = eq.bands.iter().collect();
            if unique_bands.len() != eq.bands.len()
                || eq.bands.contains(&0)
                || !eq.min_gain.is_finite()
                || !eq.max_gain.is_finite()
                || (profile.capabilities.custom_eq && eq.bands.len() != 8)
            {
                return Err(format!(
                    "EQ layout exceeds the implemented adapter contract in {}",
                    profile.id
                ));
            }
            if eq.q_values.len() != eq.bands.len()
                || eq.q_values.iter().any(|q| !q.is_finite() || *q <= 0.0)
            {
                return Err(format!("invalid EQ Q values in {}", profile.id));
            }
            if eq.bands.is_empty() || eq.min_gain >= eq.max_gain {
                return Err(format!("invalid EQ bands or gain limits in {}", profile.id));
            }
            let mut sorts = std::collections::HashSet::new();
            for preset in &eq.presets {
                if !preset.curve.is_empty() && preset.curve.len() != eq.bands.len() {
                    return Err(format!("EQ curve length mismatch in {}", profile.id));
                }
                if preset.filters.len() > 16
                    || preset.filters.iter().any(|filter| {
                        filter.frequency == 0
                            || !filter.q_value.is_finite()
                            || filter.q_value <= 0.0
                            || !filter.gain.is_finite()
                            || filter.gain < eq.min_gain
                            || filter.gain > eq.max_gain
                            || filter.filter > 2
                    })
                {
                    return Err(format!("invalid EQ filter payload in {}", profile.id));
                }
                if !sorts.insert(preset.dict_sort) {
                    return Err(format!("duplicate EQ dictSort in {}", profile.id));
                }
            }
        }
    }
    Ok(())
}

fn validate_feature_contracts(profile: &ModelProfile) -> Result<(), String> {
    use super::evidence::{EvidenceStatus, FEATURE_KEYS};
    if (profile.capabilities.eq && profile.eq.is_none())
        || (profile.capabilities.custom_eq && !profile.capabilities.eq)
    {
        return Err(format!("missing EQ feature contract in {}", profile.id));
    }
    if profile.feature_evidence.len() != FEATURE_KEYS.len()
        || FEATURE_KEYS
            .iter()
            .any(|key| !profile.feature_evidence.contains_key(*key))
    {
        return Err(format!(
            "incomplete feature evidence schema in {}",
            profile.id
        ));
    }
    for (key, evidence) in &profile.feature_evidence {
        if evidence.status != EvidenceStatus::Unknown && evidence.provenance.trim().is_empty() {
            return Err(format!(
                "missing evidence provenance for {key} in {}",
                profile.id
            ));
        }
        if profile.capabilities.enabled(key) == Some(true)
            && !matches!(
                evidence.status,
                EvidenceStatus::Implemented | EvidenceStatus::HardwareVerified
            )
        {
            return Err(format!(
                "enabled feature {key} has no implemented contract in {}",
                profile.id
            ));
        }
    }
    if profile.capabilities.bass_boost && profile.sound.is_none() {
        return Err(format!("missing sound constraints in {}", profile.id));
    }
    if let Some(sound) = &profile.sound {
        let adapter_max = match profile.protocol_family.as_str() {
            "bp1" => 1,
            "bp1Ultra" => 5,
            _ => 0,
        };
        if sound.max_bass_level == 0
            || sound.max_bass_level > adapter_max
            || sound.provenance.trim().is_empty()
        {
            return Err(format!(
                "sound constraints exceed adapter contract in {}",
                profile.id
            ));
        }
    }
    if let Some(eq) = &profile.eq {
        if eq.custom_write.slot != 101
            || eq.custom_write.anc_bank
            || profile.protocol_family != "bp1"
        {
            return Err(format!(
                "custom EQ contract is not implemented in {}",
                profile.id
            ));
        }
    }
    if profile.capabilities.gesture
        && profile
            .gesture
            .as_ref()
            .is_some_and(|gesture| gesture.protocol != super::types::GestureProtocol::Legacy)
    {
        return Err(format!(
            "gesture protocol has no implemented adapter in {}",
            profile.id
        ));
    }
    Ok(())
}
