use crate::error::{MediaError, Result};
use crate::models::{MediaItem, MediaType, MediaUrls, MediaMetadata, VideoFile, SearchResult};
use async_trait::async_trait;
use pexels_sdk::{SearchBuilder, VideoSearchBuilder};
use crate::media_provider::MediaProvider;

#[cfg(feature = "pexels")]
pub struct PexelsProvider {
    client: pexels_sdk::Pexels,
}

#[cfg(feature = "pexels")]
impl PexelsProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: pexels_sdk::Pexels::new(api_key),
        }
    }
    /// 处理查询关键字，支持多种输入格式
    ///
    /// Pexels API 支持自然语言查询，可以直接使用空格分隔的关键字
    /// 支持的格式：
    /// - 单个关键字: "nature"
    /// - 空格分隔: "nature landscape"
    /// - 逗号分隔: "nature,landscape"
    /// - 短语: "group of people working"
    ///
    /// 所有格式都会转换为空格分隔的自然语言查询
    fn process_query(query: &str) -> String {
        // 将逗号、分号等分隔符统一替换为空格，保持自然语言风格
        query
            .split(|c: char| c == ',' || c == ';' || c == '|')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(feature = "pexels")]
#[async_trait]
impl MediaProvider for PexelsProvider {
    fn name(&self) -> &str {
        "Pexels"
    }

    async fn search_images(&self, query: &str, limit: u32, page: u32) -> Result<SearchResult> {
        // 处理多关键字查询
        let processed_query = Self::process_query(query);
        let search_builder:SearchBuilder = SearchBuilder::new().query(&processed_query).per_page(limit as usize).page(page as usize);
        let response = self.client.search_photos(search_builder).await
            .map_err(|e| MediaError::PexelsError(e.to_string()))?;

        let items: Vec<MediaItem> = response.photos.into_iter().map(|photo| {
            MediaItem {
                id: photo.id.to_string(),
                media_type: MediaType::Image,
                title: photo.alt.clone(),
                description: photo.alt.clone(),
                tags: vec![],
                author: photo.photographer.clone(),
                author_url: photo.photographer_url.clone(),
                source_url: photo.url.clone(),
                provider: "Pexels".to_string(),
                urls: MediaUrls {
                    thumbnail: photo.src.tiny.clone(),
                    medium: Some(photo.src.medium.clone()),
                    large: Some(photo.src.large.clone()),
                    original: Some(photo.src.original.clone()),
                    video_files: None,
                },
                metadata: MediaMetadata {
                    width: photo.width,
                    height: photo.height,
                    size: None,
                    duration: None,
                    views: 0,
                    downloads: 0,
                    likes: 0,
                },
            }
        }).collect();

        let total_pages = SearchResult::calculate_total_pages(response.total_results, limit);

        Ok(SearchResult {
            total: response.total_results,
            total_hits: items.len() as u32,
            page,
            per_page: limit,
            total_pages,
            items,
            provider: "Pexels".to_string(),
        })
    }

    async fn search_videos(&self, query: &str, limit: u32, page: u32) -> Result<SearchResult> {
        // 处理多关键字查询
        let processed_query = Self::process_query(query);
        let search_builder:VideoSearchBuilder = VideoSearchBuilder::new().query(&processed_query).per_page(limit as usize).page(page as usize);
        let response = self.client.search_videos(search_builder).await
            .map_err(|e| MediaError::PexelsError(e.to_string()))?;

        let items: Vec<MediaItem> = response.videos.into_iter().map(|video| {
            let video_files: Vec<VideoFile> = video.video_files.iter().map(|vf| {
                VideoFile {
                    quality: vf.quality.clone().unwrap_or_else(|| "".to_string()),
                    url: vf.file_link.clone(),
                    width: vf.width,
                    height: vf.height,
                    size: 0,
                    thumbnail: None,
                }
            }).collect();

            MediaItem {
                id: video.id.to_string(),
                media_type: MediaType::Video,
                title: "Video".to_string(),
                description: String::new(),
                tags: vec![],
                author: video.user.name.clone(),
                author_url: video.user.user_url.clone(),
                source_url: video.video_url.clone(),
                provider: "Pexels".to_string(),
                urls: MediaUrls {
                    thumbnail: video.image_url.clone(),
                    medium: video_files.iter().find(|f| f.quality.to_lowercase().contains("hd")).map(|f| f.url.clone()),
                    large: video_files.iter().find(|f| f.quality.to_lowercase().contains("hd")).map(|f| f.url.clone()),
                    original: None,
                    video_files: Some(video_files),
                },
                metadata: MediaMetadata {
                    width: video.width,
                    height: video.height,
                    size: None,
                    duration: video.duration,
                    views: 0,
                    downloads: 0,
                    likes: 0,
                },
            }
        }).collect();

        let total_pages = SearchResult::calculate_total_pages(response.total_results, limit);

        Ok(SearchResult {
            total: response.total_results,
            total_hits: items.len() as u32,
            page,
            per_page: limit,
            total_pages,
            items,
            provider: "Pexels".to_string(),
        })
    }

    async fn get_media(&self, id: &str, media_type: MediaType) -> Result<MediaItem> {
        let id_num = id.parse::<i32>().map_err(|_| {
            MediaError::PexelsError("Invalid ID format".to_string())
        })?;

        match media_type {
            MediaType::Image => {
                let photo = self.client.get_photo(id_num as usize).await
                    .map_err(|e| MediaError::PexelsError(e.to_string()))?;

                Ok(MediaItem {
                    id: photo.id.to_string(),
                    media_type: MediaType::Image,
                    title: photo.alt.clone(),
                    description: photo.alt.clone(),
                    tags: vec![],
                    author: photo.photographer.clone(),
                    author_url: photo.photographer_url.clone(),
                    source_url: photo.url.clone(),
                    provider: "Pexels".to_string(),
                    urls: MediaUrls {
                        thumbnail: photo.src.tiny.clone(),
                        medium: Some(photo.src.medium.clone()),
                        large: Some(photo.src.large.clone()),
                        original: Some(photo.src.original.clone()),
                        video_files: None,
                    },
                    metadata: MediaMetadata {
                        width: photo.width,
                        height: photo.height,
                        size: None,
                        duration: None,
                        views: 0,
                        downloads: 0,
                        likes: 0,
                    },
                })
            }
            MediaType::Video => {
                let video = self.client.get_video(id_num as usize).await
                    .map_err(|e| MediaError::PexelsError(e.to_string()))?;

                let video_files: Vec<VideoFile> = video.video_files.iter().map(|vf| {
                    VideoFile {
                        quality: vf.quality.clone().unwrap_or_else(|| "".to_string()),
                        url: vf.file_link.clone(),
                        width: vf.width,
                        height: vf.height,
                        size: 0,
                        thumbnail: None,
                    }
                }).collect();

                Ok(MediaItem {
                    id: video.id.to_string(),
                    media_type: MediaType::Video,
                    title: "Video".to_string(),
                    description: String::new(),
                    tags: vec![],
                    author: video.user.name.clone(),
                    author_url: video.user.user_url.clone(),
                    source_url: video.video_url.clone(),
                    provider: "Pexels".to_string(),
                    urls: MediaUrls {
                        thumbnail: video.image_url.clone(),
                        medium: video_files.iter().find(|f| f.quality.to_lowercase().contains("hd")).map(|f| f.url.clone()),
                        large: video_files.iter().find(|f| f.quality.to_lowercase().contains("hd")).map(|f| f.url.clone()),
                        original: None,
                        video_files: Some(video_files),
                    },
                    metadata: MediaMetadata {
                        width: video.width,
                        height: video.height,
                        size: None,
                        duration: video.duration,
                        views: 0,
                        downloads: 0,
                        likes: 0,
                    },
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_query() {
        // 测试空格分隔 - 保持原样
        assert_eq!(PexelsProvider::process_query("nature landscape"), "nature landscape");

        // 测试逗号分隔 - 转换为空格
        assert_eq!(PexelsProvider::process_query("nature,landscape,mountain"), "nature landscape mountain");

        // 测试分号分隔
        assert_eq!(PexelsProvider::process_query("nature;landscape"), "nature landscape");

        // 测试竖线分隔
        assert_eq!(PexelsProvider::process_query("nature|landscape"), "nature landscape");

        // 测试混合分隔符
        assert_eq!(PexelsProvider::process_query("nature, landscape; mountain"), "nature landscape mountain");

        // 测试多余空格
        assert_eq!(PexelsProvider::process_query("  nature  ,  landscape  "), "nature landscape");

        // 测试单个关键字
        assert_eq!(PexelsProvider::process_query("nature"), "nature");

        // 测试空字符串
        assert_eq!(PexelsProvider::process_query(""), "");

        // 测试短语查询
        assert_eq!(PexelsProvider::process_query("group of people working"), "group of people working");

        // 测试组合短语和关键字
        assert_eq!(PexelsProvider::process_query("yellow flowers, mountain sunset"), "yellow flowers mountain sunset");
    }
}