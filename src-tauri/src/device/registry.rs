use crate::protocol::{DeviceProfile, ModelInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceIdentity {
    pub address: String,
    pub name: String,
    pub advertised_service: Option<String>,
    pub manufacturer_data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionSource {
    CatalogName,
    ExplicitModel,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ResolvedDevice {
    #[allow(dead_code)]
    pub identity: DeviceIdentity,
    pub model: Option<ModelInfo>,
    pub profile: DeviceProfile,
    pub source: ResolutionSource,
}

pub struct DeviceRegistry;

impl DeviceRegistry {
    pub fn resolve(identity: DeviceIdentity) -> ResolvedDevice {
        let model = crate::protocol::identify_model(&identity.name);
        Self::finish(identity, model, ResolutionSource::CatalogName)
    }

    pub fn resolve_with_model(identity: DeviceIdentity, model_id: &str) -> ResolvedDevice {
        let model = crate::protocol::catalog_json()
            .into_iter()
            .find(|item| item.id == model_id);
        let source = if model.is_some() {
            ResolutionSource::ExplicitModel
        } else {
            ResolutionSource::Unknown
        };
        Self::finish(identity, model, source)
    }

    fn finish(
        identity: DeviceIdentity,
        model: Option<ModelInfo>,
        source: ResolutionSource,
    ) -> ResolvedDevice {
        let profile = model
            .as_ref()
            .map(|item| crate::protocol::profile_for(Some(&item.id), Some(&item.display_name), None))
            .unwrap_or_else(|| crate::protocol::profile_for(None, Some(&identity.name), None));
        let source = if model.is_none() {
            ResolutionSource::Unknown
        } else {
            source
        };

        ResolvedDevice {
            identity,
            model,
            profile,
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{ProtocolFamily, SupportLevel};

    fn identity(name: &str) -> DeviceIdentity {
        DeviceIdentity {
            address: "AA:BB:CC:DD:EE:FF".into(),
            name: name.into(),
            advertised_service: None,
            manufacturer_data: Vec::new(),
        }
    }

    #[test]
    fn resolves_bp1_as_verified_profile() {
        let resolved = DeviceRegistry::resolve(identity("Baseus Bass BP1 Pro"));

        assert_eq!(resolved.source, ResolutionSource::CatalogName);
        assert_eq!(resolved.profile.protocol, ProtocolFamily::Bp1Pro);
        assert!(resolved.profile.verified);
        assert_eq!(resolved.model.as_ref().unwrap().support, SupportLevel::Verified);
    }

    #[test]
    fn resolves_other_catalog_model_without_claiming_bp1_support() {
        let resolved = DeviceRegistry::resolve(identity("Baseus Bowie MA10"));

        assert_eq!(resolved.source, ResolutionSource::CatalogName);
        assert!(!resolved.profile.verified);
        assert_eq!(resolved.model.as_ref().unwrap().support, SupportLevel::Experimental);
    }

    #[test]
    fn unknown_model_has_no_protocol_fallback() {
        let resolved = DeviceRegistry::resolve(identity("Random Earbuds"));

        assert_eq!(resolved.source, ResolutionSource::Unknown);
        assert!(resolved.model.is_none());
        assert_eq!(resolved.profile.protocol, ProtocolFamily::Unknown);
        assert!(!resolved.profile.verified);
    }

    #[test]
    fn explicit_catalog_model_overrides_ambiguous_bluetooth_name() {
        let resolved = DeviceRegistry::resolve_with_model(identity("Wireless Audio"), "bass-bp1-pro");

        assert_eq!(resolved.source, ResolutionSource::ExplicitModel);
        assert_eq!(resolved.model.as_ref().unwrap().id, "bass-bp1-pro");
        assert!(resolved.profile.verified);
    }
}
