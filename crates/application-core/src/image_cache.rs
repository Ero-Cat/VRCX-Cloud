use std::sync::Arc;

use async_trait::async_trait;
use vrcx_0_contracts::UgcCategory;

use crate::Result;

#[async_trait]
pub trait ImageCachePort: Send + Sync {
    async fn get_image(&self, url: &str, file_id: &str, version: &str) -> Result<String>;

    async fn save_image_to_file(&self, url: &str, path: &str) -> Result<()>;

    async fn save_ugc_image_to_file(
        &self,
        url: &str,
        ugc_folder_path: &str,
        category: UgcCategory,
        month_folder: &str,
        file_name: &str,
    ) -> Result<String>;
}

#[derive(Clone)]
pub struct ImageCache {
    port: Arc<dyn ImageCachePort>,
}

impl ImageCache {
    pub fn new(port: Arc<dyn ImageCachePort>) -> Self {
        Self { port }
    }

    pub async fn get_image(&self, url: &str, file_id: &str, version: &str) -> Result<String> {
        self.port.get_image(url, file_id, version).await
    }

    pub async fn save_image_to_file(&self, url: &str, path: &str) -> Result<()> {
        self.port.save_image_to_file(url, path).await
    }
}

pub async fn save_ugc_image_to_file(
    image_cache: &ImageCache,
    url: &str,
    ugc_folder_path: &str,
    category: UgcCategory,
    month_folder: &str,
    file_name: &str,
) -> Result<String> {
    image_cache
        .port
        .save_ugc_image_to_file(url, ugc_folder_path, category, month_folder, file_name)
        .await
}
