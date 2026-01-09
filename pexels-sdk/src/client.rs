use reqwest::{header, Client, StatusCode};
use std::time::Duration;
use url::Url;

use crate::models::{CollectionsPage, MediaPage, Photo, PhotosPage, Video, VideosPage};
use crate::search::{PaginationParams, SearchParams, VideoSearchParams};
use crate::PexelsError;

/// Pexels API 的主要客户端
///
/// 此客户端提供与 Pexels API 所有端点交互的方法，
/// 并处理认证、请求构建和响应解析。
pub struct PexelsClient {
    /// 用于 Pexels API 认证的 API 密钥
    api_key: String,

    /// 具有连接池和可配置超时的 HTTP 客户端
    client: Client,

    /// Pexels API 的基础 URL
    base_url: String,
}

impl PexelsClient {
    /// 使用提供的 API 密钥创建新的 PexelsClient
    ///
    /// # 参数
    ///
    /// * `api_key` - Pexels API 密钥
    ///
    /// # 返回
    ///
    /// PexelsClient 的新实例
    ///
    /// # 示例
    ///
    /// ```
    /// use pexels_sdk::PexelsClient;
    ///
    /// let client = PexelsClient::new("your_api_key");
    /// ```
    pub fn new<S: Into<String>>(api_key: S) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_default();

        Self { api_key: api_key.into(), client, base_url: "https://api.pexels.com/v1".to_string() }
    }

    /// 使用自定义配置创建新的 PexelsClient
    ///
    /// # 参数
    ///
    /// * `api_key` - Pexels API 密钥
    /// * `timeout` - 请求超时时间（秒）
    /// * `max_idle_connections` - 每个主机的最大空闲连接数
    ///
    /// # 返回
    ///
    /// PexelsClient 的新实例
    pub fn with_config<S: Into<String>>(
        api_key: S,
        timeout: u64,
        max_idle_connections: usize,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .pool_max_idle_per_host(max_idle_connections)
            .build()
            .unwrap_or_default();

        Self { api_key: api_key.into(), client, base_url: "https://api.pexels.com/v1".to_string() }
    }

    /// 为 Pexels API 设置自定义基础 URL
    ///
    /// # 参数
    ///
    /// * `base_url` - 自定义基础 URL
    ///
    /// # 返回
    ///
    /// 用于方法链的 Self
    pub fn with_base_url<S: Into<String>>(mut self, base_url: S) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// 搜索与指定查询和参数匹配的照片
    ///
    /// # 参数
    ///
    /// * `query` - 搜索查询
    /// * `params` - 其他搜索参数（分页、过滤器等）
    ///
    /// # 返回
    ///
    /// 包含照片搜索响应或错误的结果
    ///
    /// # 示例
    ///
    /// ```
    /// use pexels_sdk::{PexelsClient, SearchParams, Size};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = PexelsClient::new("your_api_key");
    ///     let params = SearchParams::new()
    ///         .page(1)
    ///         .per_page(15)
    ///         .size(Size::Large);
    ///
    ///     match client.search_photos("nature", &params).await {
    ///         Ok(photos) => println!("Found {} photos", photos.total_results),
    ///         Err(e) => println!("搜索失败: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn search_photos(
        &self,
        query: &str,
        params: &SearchParams,
    ) -> Result<PhotosPage, PexelsError> {
        let mut url = Url::parse(&format!("{}/search", self.base_url))?;

        // 添加查询参数
        url.query_pairs_mut().append_pair("query", query);

        // 添加所有搜索参数
        for (key, value) in params.to_query_params() {
            url.query_pairs_mut().append_pair(&key, &value);
        }

        let response = self.send_request(url).await?;

        match response.status() {
            StatusCode::OK => {
                let photos_page: PhotosPage = response.json().await?;
                Ok(photos_page)
            }
            StatusCode::UNAUTHORIZED => Err(PexelsError::AuthError("Invalid API key".to_string())),
            StatusCode::TOO_MANY_REQUESTS => Err(PexelsError::RateLimitError),
            status => {
                Err(PexelsError::ApiError(format!("Search photos failed with status: {status}")))
            }
        }
    }

    /// 获取精选/推荐照片
    ///
    /// # 参数
    ///
    /// * `params` - 分页参数
    ///
    /// # 返回
    ///
    /// 包含精选照片响应或错误的结果
    pub async fn curated_photos(
        &self,
        params: &PaginationParams,
    ) -> Result<PhotosPage, PexelsError> {
        let mut url = Url::parse(&format!("{}/curated", self.base_url))?;

        // 添加分页参数
        if let Some(page) = params.page {
            url.query_pairs_mut().append_pair("page", &page.to_string());
        }

        if let Some(per_page) = params.per_page {
            url.query_pairs_mut().append_pair("per_page", &per_page.to_string());
        }

        let response = self.send_request(url).await?;

        match response.status() {
            StatusCode::OK => {
                let photos_page: PhotosPage = response.json().await?;
                Ok(photos_page)
            }
            StatusCode::UNAUTHORIZED => Err(PexelsError::AuthError("Invalid API key".to_string())),
            StatusCode::TOO_MANY_REQUESTS => Err(PexelsError::RateLimitError),
            status => {
                Err(PexelsError::ApiError(format!("Curated photos failed with status: {status}")))
            }
        }
    }

    /// 根据 ID 获取特定照片
    ///
    /// # 参数
    ///
    /// * `id` - 照片 ID
    ///
    /// # 返回
    ///
    /// 包含照片或错误的结果
    pub async fn get_photo(&self, id: u64) -> Result<Photo, PexelsError> {
        let url = Url::parse(&format!("{}/photos/{}", self.base_url, id))?;

        let response = self.send_request(url).await?;

        match response.status() {
            StatusCode::OK => {
                let photo: Photo = response.json().await?;
                Ok(photo)
            }
            StatusCode::NOT_FOUND => {
                Err(PexelsError::NotFound(format!("Photo with ID {id} not found")))
            }
            StatusCode::UNAUTHORIZED => Err(PexelsError::AuthError("Invalid API key".to_string())),
            StatusCode::TOO_MANY_REQUESTS => Err(PexelsError::RateLimitError),
            status => {
                Err(PexelsError::ApiError(format!("Get photo failed with status: {status}")))
            }
        }
    }

    /// 搜索与指定查询和参数匹配的视频
    ///
    /// # 参数
    ///
    /// * `query` - 搜索查询
    /// * `params` - 其他搜索参数（分页、过滤器等）
    ///
    /// # 返回
    ///
    /// 包含视频搜索响应或错误的结果
    pub async fn search_videos(
        &self,
        query: &str,
        params: &VideoSearchParams,
    ) -> Result<VideosPage, PexelsError> {
        let mut url = Url::parse(&format!("{}/videos/search", self.base_url))?;

        // 添加查询参数
        url.query_pairs_mut().append_pair("query", query);

        // 添加所有来自 VideoSearchParams 的搜索参数
        if let Some(page) = params.page {
            url.query_pairs_mut().append_pair("page", &page.to_string());
        }

        if let Some(per_page) = params.per_page {
            url.query_pairs_mut().append_pair("per_page", &per_page.to_string());
        }

        if let Some(ref orientation) = params.orientation {
            url.query_pairs_mut().append_pair("orientation", orientation.as_str());
        }

        if let Some(ref size) = params.size {
            url.query_pairs_mut().append_pair("size", size.as_str());
        }

        if let Some(ref locale) = params.locale {
            url.query_pairs_mut().append_pair("locale", locale);
        }

        let response = self.send_request(url).await?;

        match response.status() {
            StatusCode::OK => {
                let videos_page: VideosPage = response.json().await?;
                Ok(videos_page)
            }
            StatusCode::UNAUTHORIZED => Err(PexelsError::AuthError("Invalid API key".to_string())),
            StatusCode::TOO_MANY_REQUESTS => Err(PexelsError::RateLimitError),
            status => {
                Err(PexelsError::ApiError(format!("Search videos failed with status: {status}")))
            }
        }
    }

    /// 获取热门视频
    ///
    /// # 参数
    ///
    /// * `params` - 分页参数
    ///
    /// # 返回
    ///
    /// 包含热门视频响应或错误的结果
    pub async fn popular_videos(
        &self,
        params: &PaginationParams,
    ) -> Result<VideosPage, PexelsError> {
        let mut url = Url::parse(&format!("{}/videos/popular", self.base_url))?;

        // 添加分页参数
        if let Some(page) = params.page {
            url.query_pairs_mut().append_pair("page", &page.to_string());
        }

        if let Some(per_page) = params.per_page {
            url.query_pairs_mut().append_pair("per_page", &per_page.to_string());
        }

        let response = self.send_request(url).await?;

        match response.status() {
            StatusCode::OK => {
                let videos_page: VideosPage = response.json().await?;
                Ok(videos_page)
            }
            StatusCode::UNAUTHORIZED => Err(PexelsError::AuthError("Invalid API key".to_string())),
            StatusCode::TOO_MANY_REQUESTS => Err(PexelsError::RateLimitError),
            status => {
                Err(PexelsError::ApiError(format!("Popular videos failed with status: {status}")))
            }
        }
    }

    /// 根据 ID 获取特定视频
    ///
    /// # 参数
    ///
    /// * `id` - 视频 ID
    ///
    /// # 返回
    ///
    /// 包含视频或错误的结果
    pub async fn get_video(&self, id: u64) -> Result<Video, PexelsError> {
        let url = Url::parse(&format!("{}/videos/videos/{}", self.base_url, id))?;

        let response = self.send_request(url).await?;

        match response.status() {
            StatusCode::OK => {
                let video: Video = response.json().await?;
                Ok(video)
            }
            StatusCode::NOT_FOUND => {
                Err(PexelsError::NotFound(format!("Video with ID {id} not found")))
            }
            StatusCode::UNAUTHORIZED => Err(PexelsError::AuthError("Invalid API key".to_string())),
            StatusCode::TOO_MANY_REQUESTS => Err(PexelsError::RateLimitError),
            status => {
                Err(PexelsError::ApiError(format!("Get video failed with status: {status}")))
            }
        }
    }

    /// 获取收藏列表
    ///
    /// # 参数
    ///
    /// * `params` - 分页参数
    ///
    /// # 返回
    ///
    /// 包含收藏响应或错误的结果
    pub async fn get_collections(
        &self,
        params: &PaginationParams,
    ) -> Result<CollectionsPage, PexelsError> {
        let mut url = Url::parse(&format!("{}/collections", self.base_url))?;

        // 添加分页参数
        if let Some(page) = params.page {
            url.query_pairs_mut().append_pair("page", &page.to_string());
        }

        if let Some(per_page) = params.per_page {
            url.query_pairs_mut().append_pair("per_page", &per_page.to_string());
        }

        let response = self.send_request(url).await?;

        match response.status() {
            StatusCode::OK => {
                let collections_page: CollectionsPage = response.json().await?;
                Ok(collections_page)
            }
            StatusCode::UNAUTHORIZED => Err(PexelsError::AuthError("Invalid API key".to_string())),
            StatusCode::TOO_MANY_REQUESTS => Err(PexelsError::RateLimitError),
            status => Err(PexelsError::ApiError(format!(
                "Get collections failed with status: {status}"
            ))),
        }
    }

    /// 获取收藏中的媒体项目（照片和视频）
    ///
    /// # 参数
    ///
    /// * `id` - 收藏 ID
    /// * `params` - 分页参数
    ///
    /// # 返回
    ///
    /// 包含媒体响应或错误的结果
    pub async fn get_collection_media(
        &self,
        id: &str,
        params: &PaginationParams,
    ) -> Result<MediaPage, PexelsError> {
        let mut url = Url::parse(&format!("{}/collections/{}", self.base_url, id))?;

        // 添加分页参数
        if let Some(page) = params.page {
            url.query_pairs_mut().append_pair("page", &page.to_string());
        }

        if let Some(per_page) = params.per_page {
            url.query_pairs_mut().append_pair("per_page", &per_page.to_string());
        }

        let response = self.send_request(url).await?;

        match response.status() {
            StatusCode::OK => {
                let media_page: MediaPage = response.json().await?;
                Ok(media_page)
            }
            StatusCode::NOT_FOUND => {
                Err(PexelsError::NotFound(format!("Collection with ID {id} not found")))
            }
            StatusCode::UNAUTHORIZED => Err(PexelsError::AuthError("Invalid API key".to_string())),
            StatusCode::TOO_MANY_REQUESTS => Err(PexelsError::RateLimitError),
            status => Err(PexelsError::ApiError(format!(
                "Get collection media failed with status: {status}"
            ))),
        }
    }

    /// 辅助方法，用于向 Pexels API 发送认证请求
    ///
    /// # 参数
    ///
    /// * `url` - 要发送请求的完整构造 URL
    ///
    /// # 返回
    ///
    /// 包含 HTTP 响应或错误的结果
    async fn send_request(&self, url: Url) -> Result<reqwest::Response, PexelsError> {
        let response =
            self.client.get(url).header(header::AUTHORIZATION, &self.api_key).send().await?;

        Ok(response)
    }
}
