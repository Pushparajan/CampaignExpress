//! Digital asset management adaptors — connectors for AEM Assets, Bynder, and Aprimo.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DamProvider {
    AemAssets,
    Bynder,
    Aprimo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamConfig {
    pub provider: DamProvider,
    pub api_base_url: String,
    pub api_token: String,
    pub workspace_id: Option<String>,
    pub auto_sync: bool,
    pub sync_interval_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamAsset {
    pub id: String,
    pub provider: DamProvider,
    pub external_id: String,
    pub name: String,
    pub file_type: String,
    pub mime_type: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub file_size_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub tags: Vec<String>,
    pub folder_path: String,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamSyncResult {
    pub provider: DamProvider,
    pub assets_synced: u32,
    pub assets_created: u32,
    pub assets_updated: u32,
    pub assets_deleted: u32,
    pub errors: Vec<String>,
    pub synced_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Adaptor
// ---------------------------------------------------------------------------

pub struct DamAdaptor {
    configs: DashMap<String, DamConfig>,
    assets: DashMap<String, DamAsset>,
    folder_index: DashMap<String, Vec<String>>,
}

impl DamAdaptor {
    pub fn new() -> Self {
        Self {
            configs: DashMap::new(),
            assets: DashMap::new(),
            folder_index: DashMap::new(),
        }
    }

    /// Register a named DAM provider configuration.
    pub fn register_provider(&self, name: &str, config: DamConfig) {
        tracing::info!(provider = name, "Registering DAM provider");
        self.configs.insert(name.to_string(), config);
    }

    /// Simulate syncing assets from the external DAM system.
    /// Creates 10 sample assets per provider with provider-specific URLs.
    pub fn sync_assets(&self, provider_name: &str) -> Option<DamSyncResult> {
        let config = self.configs.get(provider_name)?;
        let provider = config.provider.clone();

        let sample_assets: Vec<(&str, &str, &str, u64, &[&str])> = vec![
            (
                "hero-banner.jpg",
                "jpg",
                "image/jpeg",
                2_500_000,
                &["banner", "hero"],
            ),
            (
                "logo-dark.png",
                "png",
                "image/png",
                150_000,
                &["logo", "branding"],
            ),
            (
                "promo-video.mp4",
                "mp4",
                "video/mp4",
                45_000_000,
                &["video", "promo"],
            ),
            (
                "product-shot-1.jpg",
                "jpg",
                "image/jpeg",
                3_200_000,
                &["product", "photography"],
            ),
            (
                "brand-guidelines.pdf",
                "pdf",
                "application/pdf",
                8_000_000,
                &["guidelines", "brand"],
            ),
            (
                "email-header.png",
                "png",
                "image/png",
                400_000,
                &["email", "header"],
            ),
            (
                "social-ad-square.jpg",
                "jpg",
                "image/jpeg",
                1_800_000,
                &["social", "ad"],
            ),
            (
                "icon-set.svg",
                "svg",
                "image/svg+xml",
                25_000,
                &["icons", "ui"],
            ),
            (
                "landing-page-bg.jpg",
                "jpg",
                "image/jpeg",
                5_000_000,
                &["background", "landing"],
            ),
            (
                "cta-button.png",
                "png",
                "image/png",
                12_000,
                &["button", "cta"],
            ),
        ];

        let folders = ["/campaigns", "/brand", "/products", "/social", "/email"];
        let mut created = 0u32;

        for (i, (name, ftype, mime, size, tags)) in sample_assets.iter().enumerate() {
            let asset_id = Uuid::new_v4().to_string();
            let external_id = format!("ext-{}", Uuid::new_v4());
            let folder = folders[i % folders.len()];

            let (url, download_url) = match provider {
                DamProvider::AemAssets => (
                    format!("/content/dam{}/{}", folder, name),
                    format!(
                        "/content/dam{}/{}/jcr:content/renditions/original",
                        folder, name
                    ),
                ),
                DamProvider::Bynder => (
                    format!("https://media.bynder.com/m/{}", asset_id),
                    format!("https://media.bynder.com/m/{}/download", asset_id),
                ),
                DamProvider::Aprimo => (
                    format!("https://dam.aprimo.com/assets/{}", asset_id),
                    format!("https://dam.aprimo.com/assets/{}/download", asset_id),
                ),
            };

            let (width, height) = if mime.starts_with("image/") {
                (Some(1920), Some(1080))
            } else {
                (None, None)
            };

            let thumbnail_url = if mime.starts_with("image/") || mime.starts_with("video/") {
                Some(format!("{}/thumbnail", url))
            } else {
                None
            };

            let now = Utc::now();
            let asset = DamAsset {
                id: asset_id.clone(),
                provider: provider.clone(),
                external_id,
                name: name.to_string(),
                file_type: ftype.to_string(),
                mime_type: mime.to_string(),
                url,
                thumbnail_url,
                file_size_bytes: *size,
                width,
                height,
                tags: tags.iter().map(|t| t.to_string()).collect(),
                folder_path: folder.to_string(),
                metadata: HashMap::new(),
                created_at: now,
                updated_at: now,
                download_url,
            };

            self.assets.insert(asset_id.clone(), asset);
            self.folder_index
                .entry(folder.to_string())
                .or_default()
                .push(asset_id);
            created += 1;
        }

        tracing::info!(provider = provider_name, created, "Synced DAM assets");

        Some(DamSyncResult {
            provider,
            assets_synced: created,
            assets_created: created,
            assets_updated: 0,
            assets_deleted: 0,
            errors: Vec::new(),
            synced_at: Utc::now(),
        })
    }

    /// Search assets by name or tags, optionally filtering by provider and file type.
    pub fn search_assets(
        &self,
        query: &str,
        provider: Option<DamProvider>,
        file_type: Option<&str>,
    ) -> Vec<DamAsset> {
        let q = query.to_lowercase();
        self.assets
            .iter()
            .filter(|entry| {
                let asset = entry.value();
                let matches_query = asset.name.to_lowercase().contains(&q)
                    || asset.tags.iter().any(|t| t.to_lowercase().contains(&q));
                let matches_provider = provider.as_ref().is_none_or(|p| asset.provider == *p);
                let matches_type = file_type.is_none_or(|ft| asset.file_type == ft);
                matches_query && matches_provider && matches_type
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Retrieve a single asset by ID.
    pub fn get_asset(&self, id: &str) -> Option<DamAsset> {
        self.assets.get(id).map(|a| a.clone())
    }

    /// List all assets in a given folder path.
    pub fn list_folder(&self, folder_path: &str) -> Vec<DamAsset> {
        self.folder_index
            .get(folder_path)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.assets.get(id).map(|a| a.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Simulate importing an asset for use in campaigns (returns asset copy).
    pub fn import_to_campaign(&self, asset_id: &str) -> Option<DamAsset> {
        let asset = self.assets.get(asset_id)?;
        tracing::info!(asset_id, name = %asset.name, "Importing asset to campaign");
        Some(asset.clone())
    }

    /// Return (provider_name, asset_count) for each registered provider.
    pub fn get_provider_stats(&self) -> Vec<(String, u32)> {
        let mut stats: HashMap<String, u32> = HashMap::new();
        for config_entry in self.configs.iter() {
            let provider_name = config_entry.key().clone();
            let provider = &config_entry.value().provider;
            let count = self
                .assets
                .iter()
                .filter(|a| a.value().provider == *provider)
                .count() as u32;
            stats.insert(provider_name, count);
        }
        stats.into_iter().collect()
    }
}

impl Default for DamAdaptor {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn aem_config() -> DamConfig {
        DamConfig {
            provider: DamProvider::AemAssets,
            api_base_url: "https://author.example.com".to_string(),
            api_token: "test-token".to_string(),
            workspace_id: None,
            auto_sync: true,
            sync_interval_minutes: 30,
        }
    }

    fn bynder_config() -> DamConfig {
        DamConfig {
            provider: DamProvider::Bynder,
            api_base_url: "https://mycompany.bynder.com/api/v4".to_string(),
            api_token: "test-token".to_string(),
            workspace_id: Some("ws-1".to_string()),
            auto_sync: false,
            sync_interval_minutes: 60,
        }
    }

    #[test]
    fn test_register_provider() {
        let adaptor = DamAdaptor::new();
        adaptor.register_provider("aem", aem_config());
        adaptor.register_provider("bynder", bynder_config());

        // Registering should not create any assets yet
        assert_eq!(adaptor.get_provider_stats().len(), 2);
    }

    #[test]
    fn test_sync_assets() {
        let adaptor = DamAdaptor::new();
        adaptor.register_provider("aem", aem_config());

        let result = adaptor.sync_assets("aem").expect("sync should succeed");
        assert_eq!(result.assets_synced, 10);
        assert_eq!(result.assets_created, 10);
        assert!(result.errors.is_empty());
        assert_eq!(result.provider, DamProvider::AemAssets);

        // Unknown provider returns None
        assert!(adaptor.sync_assets("unknown").is_none());
    }

    #[test]
    fn test_search_assets() {
        let adaptor = DamAdaptor::new();
        adaptor.register_provider("aem", aem_config());
        adaptor.sync_assets("aem");

        // Search by name
        let results = adaptor.search_assets("banner", None, None);
        assert!(!results.is_empty());
        assert!(results.iter().all(|a| a.name.contains("banner")));

        // Search by tag
        let tagged = adaptor.search_assets("logo", None, None);
        assert!(!tagged.is_empty());

        // Filter by file type
        let pngs = adaptor.search_assets("", None, Some("png"));
        assert!(pngs.iter().all(|a| a.file_type == "png"));

        // Filter by provider (no bynder assets yet)
        let bynder_assets = adaptor.search_assets("banner", Some(DamProvider::Bynder), None);
        assert!(bynder_assets.is_empty());
    }

    #[test]
    fn test_folder_listing() {
        let adaptor = DamAdaptor::new();
        adaptor.register_provider("aem", aem_config());
        adaptor.sync_assets("aem");

        let campaigns = adaptor.list_folder("/campaigns");
        assert!(!campaigns.is_empty());

        let empty = adaptor.list_folder("/nonexistent");
        assert!(empty.is_empty());
    }
}
