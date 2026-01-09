use crate::error::{MediaError, Result};
use crate::media_provider::MediaProvider;
use crate::models::{MediaItem, MediaMetadata, MediaType, MediaUrls, SearchResult, VideoFile};
use async_trait::async_trait;

/// Pixabay 提供商实现
pub struct PixabayProvider {
    client: pixabay_sdk::Pixabay,
}

impl PixabayProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: pixabay_sdk::Pixabay::new(api_key),
        }
    }
    /// 处理查询关键字，支持多种输入格式
    ///
    /// 支持的格式：
    /// - 单个关键字: "nature"
    /// - 空格分隔: "nature landscape"
    /// - 逗号分隔: "nature,landscape"
    /// - 数组形式: &["nature", "landscape"]
    ///
    /// Pixabay API 使用 + 号连接多个关键字（URL编码后的空格）
    /// HTTP 客户端库（如 reqwest）会自动对 URL 参数进行编码，将空格转换为 %20 或 +
    fn process_query(query: &str) -> String {
        // 统一使用空白、逗号、分号、竖线作为分隔符，然后用 + 连接关键字
        query
            .split([' ', ',', ';', '|'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("+")
    }
}

#[async_trait]
impl MediaProvider for PixabayProvider {
    fn name(&self) -> &str {
        "Pixabay"
    }

    async fn search_images(&self, query: &str, limit: u32, page: u32) -> Result<SearchResult> {
        // 处理多关键字查询
        let processed_query = Self::process_query(query);
        let response = self
            .client
            .search_images(&processed_query, Some(limit), Some(page))
            .await?;

        let items: Vec<MediaItem> = response
            .hits
            .into_iter()
            .map(|img| MediaItem {
                id: img.id.to_string(),
                media_type: MediaType::Image,
                title: img.tags.clone(),
                description: img.tags.clone(),
                tags: img.tags.split(',').map(|s| s.trim().to_string()).collect(),
                author: img.user.clone(),
                author_url: format!("https://pixabay.com/users/{}-{}/", img.user, img.user_id),
                source_url: img.page_url.clone(),
                provider: "Pixabay".to_string(),
                urls: MediaUrls {
                    thumbnail: img.preview_url.clone(),
                    medium: Some(img.webformat_url.clone()),
                    large: Some(img.large_image_url.clone()),
                    original: img.image_url.clone(),
                    video_files: None,
                },
                metadata: MediaMetadata {
                    width: img.image_width,
                    height: img.image_height,
                    size: Some(img.image_size),
                    duration: None,
                    views: img.views,
                    downloads: img.downloads,
                    likes: img.likes,
                },
            })
            .collect();

        let total_pages = SearchResult::calculate_total_pages(response.total, limit);

        Ok(SearchResult {
            total: response.total,
            total_hits: response.total_hits,
            page,
            per_page: limit,
            total_pages,
            items,
            provider: "Pixabay".to_string(),
        })
    }

    async fn search_videos(&self, query: &str, limit: u32, page: u32) -> Result<SearchResult> {
        // 处理多关键字查询
        let processed_query = Self::process_query(query);
        let response = self
            .client
            .search_videos(&processed_query, Some(limit), Some(page))
            .await?;

        let items: Vec<MediaItem> = response
            .hits
            .into_iter()
            .map(|vid| {
                let video_files: Vec<VideoFile> = vec![
                    vid.videos.large.as_ref().map(|v| VideoFile {
                        quality: "large".to_string(),
                        url: v.url.clone(),
                        width: v.width,
                        height: v.height,
                        size: v.size,
                        thumbnail: Some(v.thumbnail.clone()),
                    }),
                    vid.videos.medium.as_ref().map(|v| VideoFile {
                        quality: "medium".to_string(),
                        url: v.url.clone(),
                        width: v.width,
                        height: v.height,
                        size: v.size,
                        thumbnail: Some(v.thumbnail.clone()),
                    }),
                    vid.videos.small.as_ref().map(|v| VideoFile {
                        quality: "small".to_string(),
                        url: v.url.clone(),
                        width: v.width,
                        height: v.height,
                        size: v.size,
                        thumbnail: Some(v.thumbnail.clone()),
                    }),
                    vid.videos.tiny.as_ref().map(|v| VideoFile {
                        quality: "tiny".to_string(),
                        url: v.url.clone(),
                        width: v.width,
                        height: v.height,
                        size: v.size,
                        thumbnail: Some(v.thumbnail.clone()),
                    }),
                ]
                .into_iter()
                .flatten()
                .collect();

                let thumbnail = video_files
                    .first()
                    .and_then(|f| f.thumbnail.clone())
                    .unwrap_or_default();

                MediaItem {
                    id: vid.id.to_string(),
                    media_type: MediaType::Video,
                    title: vid.tags.clone(),
                    description: vid.tags.clone(),
                    tags: vid.tags.split(',').map(|s| s.trim().to_string()).collect(),
                    author: vid.user.clone(),
                    author_url: format!("https://pixabay.com/users/{}-{}/", vid.user, vid.user_id),
                    source_url: vid.page_url.clone(),
                    provider: "Pixabay".to_string(),
                    urls: MediaUrls {
                        thumbnail,
                        medium: video_files
                            .iter()
                            .find(|f| f.quality == "medium")
                            .map(|f| f.url.clone()),
                        large: video_files
                            .iter()
                            .find(|f| f.quality == "large")
                            .map(|f| f.url.clone()),
                        original: None,
                        video_files: Some(video_files),
                    },
                    metadata: MediaMetadata {
                        width: vid.videos.large.as_ref().map(|v| v.width).unwrap_or(0),
                        height: vid.videos.large.as_ref().map(|v| v.height).unwrap_or(0),
                        size: vid.videos.large.as_ref().map(|v| v.size),
                        duration: Some(vid.duration),
                        views: vid.views,
                        downloads: vid.downloads,
                        likes: vid.likes,
                    },
                }
            })
            .collect();

        let total_pages = SearchResult::calculate_total_pages(response.total, limit);

        Ok(SearchResult {
            total: response.total,
            total_hits: response.total_hits,
            page,
            per_page: limit,
            total_pages,
            items,
            provider: "Pixabay".to_string(),
        })
    }

    async fn get_media(&self, id: &str, media_type: MediaType) -> Result<MediaItem> {
        let id_num = id.parse::<u64>().map_err(|_| {
            MediaError::PixabayError(pixabay_sdk::PixabayError::ApiError(
                "Invalid ID format".to_string(),
            ))
        })?;

        match media_type {
            MediaType::Image => {
                let img = self.client.get_image(id_num).await?;
                Ok(MediaItem {
                    id: img.id.to_string(),
                    media_type: MediaType::Image,
                    title: img.tags.clone(),
                    description: img.tags.clone(),
                    tags: img.tags.split(',').map(|s| s.trim().to_string()).collect(),
                    author: img.user.clone(),
                    author_url: format!("https://pixabay.com/users/{}-{}/", img.user, img.user_id),
                    source_url: img.page_url.clone(),
                    provider: "Pixabay".to_string(),
                    urls: MediaUrls {
                        thumbnail: img.preview_url.clone(),
                        medium: Some(img.webformat_url.clone()),
                        large: Some(img.large_image_url.clone()),
                        original: img.image_url.clone(),
                        video_files: None,
                    },
                    metadata: MediaMetadata {
                        width: img.image_width,
                        height: img.image_height,
                        size: Some(img.image_size),
                        duration: None,
                        views: img.views,
                        downloads: img.downloads,
                        likes: img.likes,
                    },
                })
            }
            MediaType::Video => {
                let vid = self.client.get_video(id_num).await?;
                let video_files: Vec<VideoFile> = vec![
                    vid.videos.large.as_ref().map(|v| VideoFile {
                        quality: "large".to_string(),
                        url: v.url.clone(),
                        width: v.width,
                        height: v.height,
                        size: v.size,
                        thumbnail: Some(v.thumbnail.clone()),
                    }),
                    vid.videos.medium.as_ref().map(|v| VideoFile {
                        quality: "medium".to_string(),
                        url: v.url.clone(),
                        width: v.width,
                        height: v.height,
                        size: v.size,
                        thumbnail: Some(v.thumbnail.clone()),
                    }),
                    vid.videos.small.as_ref().map(|v| VideoFile {
                        quality: "small".to_string(),
                        url: v.url.clone(),
                        width: v.width,
                        height: v.height,
                        size: v.size,
                        thumbnail: Some(v.thumbnail.clone()),
                    }),
                    vid.videos.tiny.as_ref().map(|v| VideoFile {
                        quality: "tiny".to_string(),
                        url: v.url.clone(),
                        width: v.width,
                        height: v.height,
                        size: v.size,
                        thumbnail: Some(v.thumbnail.clone()),
                    }),
                ]
                .into_iter()
                .flatten()
                .collect();

                let thumbnail = video_files
                    .first()
                    .and_then(|f| f.thumbnail.clone())
                    .unwrap_or_default();

                Ok(MediaItem {
                    id: vid.id.to_string(),
                    media_type: MediaType::Video,
                    title: vid.tags.clone(),
                    description: vid.tags.clone(),
                    tags: vid.tags.split(',').map(|s| s.trim().to_string()).collect(),
                    author: vid.user.clone(),
                    author_url: format!("https://pixabay.com/users/{}-{}/", vid.user, vid.user_id),
                    source_url: vid.page_url.clone(),
                    provider: "Pixabay".to_string(),
                    urls: MediaUrls {
                        thumbnail,
                        medium: video_files
                            .iter()
                            .find(|f| f.quality == "medium")
                            .map(|f| f.url.clone()),
                        large: video_files
                            .iter()
                            .find(|f| f.quality == "large")
                            .map(|f| f.url.clone()),
                        original: None,
                        video_files: Some(video_files),
                    },
                    metadata: MediaMetadata {
                        width: vid.videos.large.as_ref().map(|v| v.width).unwrap_or(0),
                        height: vid.videos.large.as_ref().map(|v| v.height).unwrap_or(0),
                        size: vid.videos.large.as_ref().map(|v| v.size),
                        duration: Some(vid.duration),
                        views: vid.views,
                        downloads: vid.downloads,
                        likes: vid.likes,
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
        // 测试空格分隔 - 空格会被保留并转换为 +
        assert_eq!(
            PixabayProvider::process_query("nature landscape"),
            "nature+landscape"
        );

        // 测试逗号分隔
        assert_eq!(
            PixabayProvider::process_query("nature,landscape,mountain"),
            "nature+landscape+mountain"
        );

        // 测试分号分隔
        assert_eq!(
            PixabayProvider::process_query("nature;landscape"),
            "nature+landscape"
        );

        // 测试竖线分隔
        assert_eq!(
            PixabayProvider::process_query("nature|landscape"),
            "nature+landscape"
        );

        // 测试混合分隔符
        assert_eq!(
            PixabayProvider::process_query("nature, landscape; mountain"),
            "nature+landscape+mountain"
        );

        // 测试多余空格
        assert_eq!(
            PixabayProvider::process_query("  nature  ,  landscape  "),
            "nature+landscape"
        );

        // 测试单个关键字
        assert_eq!(PixabayProvider::process_query("nature"), "nature");

        // 测试空字符串
        assert_eq!(PixabayProvider::process_query(""), "");

        // 测试带空格的短语
        assert_eq!(
            PixabayProvider::process_query("yellow flowers"),
            "yellow+flowers"
        );
    }
}
