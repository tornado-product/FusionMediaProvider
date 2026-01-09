use crate::error::Result;
use crate::models::{MediaItem, MediaType, SearchResult};
use async_trait::async_trait;

/// 媒体提供商的 Trait（Pixabay, Pexels 等）
#[async_trait]
pub trait MediaProvider: Send + Sync {
    /// 获取提供商名称
    fn name(&self) -> &str;

    /// 搜索图片
    async fn search_images(&self, query: &str, limit: u32, page: u32) -> Result<SearchResult>;

    /// 搜索视频
    async fn search_videos(&self, query: &str, limit: u32, page: u32) -> Result<SearchResult>;

    /// 通过 ID 获取媒体项
    async fn get_media(&self, id: &str, media_type: MediaType) -> Result<MediaItem>;
}
