//! Public server discovery metadata is independent of reviewed protocol profiles.
//! CDN references stay in the snapshot; discovery never starts a network request.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    schema_version: u8,
    models: Vec<PublicModel>,
}

#[derive(Debug, Deserialize)]
pub struct PublicModel {
    pub id: String,
    pub model: String,
    pub audio: bool,
    pub variants: Vec<PublicVariant>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicVariant {
    pub category_path: Vec<String>,
    pub colors: Vec<PublicColor>,
}

#[derive(Debug, Deserialize)]
pub struct PublicColor {
    pub code: i32,
}

static SNAPSHOT: once_cell::sync::Lazy<Snapshot> = once_cell::sync::Lazy::new(|| {
    let snapshot: Snapshot = serde_json::from_str(include_str!("../../catalog/baseus-public.json"))
        .expect("valid public Baseus catalog");
    assert_eq!(
        snapshot.schema_version, 1,
        "supported public catalog schema"
    );
    snapshot
});

pub fn audio_models() -> impl Iterator<Item = &'static PublicModel> {
    SNAPSHOT.models.iter().filter(|model| model.audio)
}

/// Prefix differences in the app registry are not different product identities.
pub fn identity_key(name: &str) -> String {
    let lower = name.trim().to_lowercase();
    lower.strip_prefix("baseus ").unwrap_or(&lower).to_owned()
}

impl PublicModel {
    pub fn name_patterns(&self) -> Vec<String> {
        let mut names = vec![self.model.to_lowercase(), identity_key(&self.model)];
        names.sort();
        names.dedup();
        names
    }

    pub fn group(&self) -> String {
        self.variants
            .first()
            .and_then(|variant| variant.category_path.last())
            .cloned()
            .unwrap_or_else(|| "Audio".into())
    }

    pub fn color_codes(&self) -> Vec<String> {
        let mut codes: Vec<_> = self
            .variants
            .iter()
            .flat_map(|v| &v.colors)
            .map(|c| c.code)
            .collect();
        codes.sort();
        codes.dedup();
        codes.into_iter().map(|code| code.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_has_unique_nonempty_identities_and_region_variants() {
        let mut ids = std::collections::HashSet::new();
        let mut names = std::collections::HashSet::new();
        assert!(!SNAPSHOT.models.is_empty());
        for model in &SNAPSHOT.models {
            assert!(!model.id.is_empty());
            assert!(!model.model.is_empty());
            assert!(ids.insert(&model.id));
            assert!(names.insert(model.model.to_lowercase()));
            assert!(!model.variants.is_empty());
        }
    }

    #[test]
    fn model_identity_only_ignores_the_brand_prefix() {
        assert_eq!(
            identity_key("Baseus Bowie MA10"),
            identity_key("Bowie MA10")
        );
        assert_ne!(
            identity_key("Baseus Bowie MA10"),
            identity_key("Baseus Bowie MA10 Pro")
        );
    }

    #[test]
    fn non_audio_server_products_are_not_registered_as_earbuds() {
        let models = crate::protocol::catalog_json();
        for product in SNAPSHOT.models.iter().filter(|model| !model.audio) {
            assert!(!models.iter().any(|model| model.id == product.id));
        }
    }
}
