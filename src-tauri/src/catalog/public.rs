//! Public server discovery metadata is independent of reviewed protocol profiles.
//! CDN references stay in the snapshot; discovery never starts a network request.

use serde::{Deserialize, Serialize};

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
    pub region: String,
    pub product_name: String,
    pub image_url: Option<String>,
    pub large_image_url: Option<String>,
    pub category_path: Vec<String>,
    pub colors: Vec<PublicColor>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicColor {
    pub code: i32,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductPresentation {
    pub region: String,
    pub product_name: String,
    pub image_url: Option<String>,
    pub large_image_url: Option<String>,
    pub category_path: Vec<String>,
    pub colors: Vec<PublicColor>,
}

static SNAPSHOT: std::sync::LazyLock<Snapshot> = std::sync::LazyLock::new(|| {
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

/// New headphone discovery excludes products classified only as speakers.
/// A model remains eligible when regional metadata disagrees, avoiding a
/// false exclusion from one region's category label.
pub fn headphone_models() -> impl Iterator<Item = &'static PublicModel> {
    audio_models().filter(|model| !model.is_speaker_only())
}

/// Whether a resolved catalog identity belongs in the new headphone scan list.
/// Unknown and regionally mixed identities remain visible until classified.
pub fn is_headphone_candidate(name: &str) -> bool {
    SNAPSHOT
        .models
        .iter()
        .find(|model| identity_key(&model.model) == identity_key(name))
        .map_or(true, |model| !model.is_speaker_only())
}

/// A display prefix alone does not create a different catalog identity.
pub fn identity_key(name: &str) -> String {
    let lower = name.trim().to_lowercase();
    lower.strip_prefix("baseus ").unwrap_or(&lower).to_owned()
}

impl PublicModel {
    pub fn presentation(&self) -> Option<ProductPresentation> {
        self.variants
            .iter()
            .min_by_key(|variant| region_rank(&variant.region))
            .map(|variant| ProductPresentation {
                region: variant.region.clone(),
                product_name: variant.product_name.clone(),
                image_url: variant.image_url.clone(),
                large_image_url: variant.large_image_url.clone(),
                category_path: variant.category_path.clone(),
                colors: variant.colors.clone(),
            })
    }
    pub fn presentation_name(&self) -> String {
        let mut variants: Vec<_> = self.variants.iter().collect();
        variants.sort_by_key(|variant| region_rank(&variant.region));
        variants
            .into_iter()
            .find(|variant| !variant.product_name.trim().is_empty())
            .map(|variant| variant.product_name.clone())
            .unwrap_or_else(|| self.model.clone())
    }
    /// Prefer the international snapshot and large product art, without inferring device color.
    pub fn product_image(&self) -> Option<String> {
        let mut variants: Vec<_> = self.variants.iter().collect();
        variants.sort_by_key(|variant| region_rank(&variant.region));
        variants.into_iter().find_map(|variant| {
            [&variant.large_image_url, &variant.image_url]
                .into_iter()
                .flatten()
                .find(|url| url.starts_with("https://") && !url.trim().is_empty())
                .cloned()
        })
    }

    fn is_speaker_only(&self) -> bool {
        !self.variants.is_empty()
            && self.variants.iter().all(|variant| {
                variant
                    .category_path
                    .last()
                    .is_some_and(|category| category.to_lowercase().contains("speaker"))
            })
    }

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

fn region_rank(region: &str) -> u8 {
    match region {
        "us" => 0,
        "eu" => 1,
        _ => 2,
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
    fn headphone_discovery_excludes_only_consistently_classified_speakers() {
        let speakers: Vec<_> = audio_models()
            .filter(|model| model.is_speaker_only())
            .collect();
        assert_eq!(speakers.len(), 5);
        assert_eq!(
            headphone_models().count(),
            audio_models().count() - speakers.len()
        );
        assert!(speakers
            .iter()
            .all(|model| !is_headphone_candidate(&model.model)));

        let regionally_mixed = PublicModel {
            id: "mixed-region".into(),
            model: "Mixed Region".into(),
            audio: true,
            variants: vec![
                PublicVariant {
                    region: "us".into(),
                    product_name: "Mixed Region".into(),
                    image_url: None,
                    large_image_url: None,
                    category_path: vec!["Audio".into(), "Speaker series".into()],
                    colors: Vec::new(),
                },
                PublicVariant {
                    region: "eu".into(),
                    product_name: "Mixed Region".into(),
                    image_url: None,
                    large_image_url: None,
                    category_path: vec!["Audio".into(), "In ear series".into()],
                    colors: Vec::new(),
                },
            ],
        };
        assert!(!regionally_mixed.is_speaker_only());
    }

    #[test]
    fn non_audio_server_products_are_not_registered_as_earbuds() {
        let models = crate::protocol::catalog_json();
        for product in SNAPSHOT.models.iter().filter(|model| !model.audio) {
            assert!(!models.iter().any(|model| model.id == product.id));
        }
    }
}
