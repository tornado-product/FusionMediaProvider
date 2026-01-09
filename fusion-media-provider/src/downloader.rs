use crate::create_provider::create_provider;
use crate::error::{MediaError, Result};
use crate::media_provider::MediaProvider;
use crate::models::{
    AggregatedSearchResult, BatchDownloadProgress, DownloadProgress, DownloadState, ImageQuality,
    MediaItem, MediaType, ProgressCallback, SearchResult, VideoQuality,
};
use futures::future::join_all;
use log::error;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// 媒体下载配置
#[derive(Clone)]
pub struct DownloadConfig {
    /// 首选图片质量
    pub image_quality: ImageQuality,
    /// 首选视频质量
    pub video_quality: VideoQuality,
    /// 下载目录
    pub output_dir: String,
    /// 是否使用原始文件名
    pub use_original_names: bool,
    /// 最大并发下载数
    pub max_concurrent: usize,
    /// 进度回调（可选）
    pub progress_callback: Option<ProgressCallback>,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            image_quality: ImageQuality::Large,
            video_quality: VideoQuality::Large,
            output_dir: "./downloads".to_string(),
            use_original_names: false,
            max_concurrent: 5,
            progress_callback: None,
        }
    }
}

impl std::fmt::Debug for DownloadConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadConfig")
            .field("image_quality", &self.image_quality)
            .field("video_quality", &self.video_quality)
            .field("output_dir", &self.output_dir)
            .field("use_original_names", &self.use_original_names)
            .field("max_concurrent", &self.max_concurrent)
            .field("progress_callback", &self.progress_callback.is_some())
            .finish()
    }
}

/// 搜索参数
#[derive(Debug, Clone)]
pub struct SearchParams {
    pub query: String, //查询关键字
    pub limit: u32,    //每页记录数
    pub page: u32,     //第几页
    pub media_type: MediaType,
}

impl SearchParams {
    pub fn new(query: impl Into<String>, media_type: MediaType) -> Self {
        Self {
            query: query.into(),
            limit: 20,
            page: 1,
            media_type,
        }
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.limit = per_page;
        self
    }
}

/// 聚合多个提供商的主媒体下载器
pub struct MediaDownloader {
    providers: Vec<Arc<dyn MediaProvider>>,
    config: DownloadConfig,
    http_client: reqwest::Client,
}

impl MediaDownloader {
    /// 创建新的媒体下载器
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            config: DownloadConfig::default(),
            http_client: reqwest::Client::new(),
        }
    }

    /// 设置下载配置
    pub fn with_config(mut self, config: DownloadConfig) -> Self {
        self.config = config;
        self
    }

    /// 添加提供商
    pub fn add_provider(mut self, provider: Arc<dyn MediaProvider>) -> Self {
        self.providers.push(provider);
        self
    }

    /// 根据名称添加提供商
    pub fn add_provider_by_name_and_apikey(mut self, provider_name: &str, api_key: &str) -> Self {
        let provider_res = create_provider(provider_name, api_key);
        match provider_res {
            Ok(provider) => {
                self.providers.push(provider);
            }
            Err(e) => {
                error!("Error creating provider: {}", e);
            }
        }
        self
    }

    /// 获取所有提供商
    pub fn providers(&self) -> &[Arc<dyn MediaProvider>] {
        &self.providers
    }

    /// 从所有提供商搜索媒体
    ///
    /// 返回所有提供商的聚合结果，包含组合的分页信息
    pub async fn search(&self, params: SearchParams) -> Result<AggregatedSearchResult> {
        if self.providers.is_empty() {
            return Err(MediaError::NoProviders);
        }

        let futures: Vec<_> = self
            .providers
            .iter()
            .map(|provider| {
                let provider = Arc::clone(provider);
                let params = params.clone();

                async move {
                    match params.media_type {
                        MediaType::Image => {
                            provider
                                .search_images(&params.query, params.limit, params.page)
                                .await
                        }
                        MediaType::Video => {
                            provider
                                .search_videos(&params.query, params.limit, params.page)
                                .await
                        }
                    }
                }
            })
            .collect();

        let results = join_all(futures).await;

        let mut provider_results = Vec::new();
        let mut all_items = Vec::new();
        let mut total_sum = 0u32;
        let mut total_hits_sum = 0u32;
        let mut total_pages_sum = 0u32;
        let mut has_success = false;

        for result in results {
            match result {
                Ok(search_result) => {
                    has_success = true;

                    // 聚合所有提供商的总数
                    total_sum += search_result.total;
                    total_hits_sum += search_result.total_hits;
                    total_pages_sum += search_result.total_pages;

                    // 收集项目
                    all_items.extend(search_result.items.clone());

                    // 存储提供商特定的结果
                    provider_results.push(search_result);
                }
                Err(e) => {
                    eprintln!("提供商失败: {}", e);
                }
            }
        }

        if !has_success {
            return Err(MediaError::AllProvidersFailed);
        }

        Ok(AggregatedSearchResult {
            provider: provider_results
                .first()
                .map(|r| r.provider.clone())
                .unwrap_or_else(|| "all".to_string()),
            total: total_sum,
            total_hits: total_hits_sum,
            page: params.page,
            per_page: params.limit,
            total_pages: total_pages_sum,
            items: all_items,
            provider_results,
        })
    }

    /// 从特定提供商搜索媒体
    pub async fn search_from_provider(
        &self,
        provider_name: &str,
        params: SearchParams,
    ) -> Result<SearchResult> {
        let provider = self
            .providers
            .iter()
            .find(|p| p.name() == provider_name)
            .ok_or_else(|| MediaError::DownloadError(format!("未找到提供商 {}", provider_name)))?;

        match params.media_type {
            MediaType::Image => {
                provider
                    .search_images(&params.query, params.limit, params.page)
                    .await
            }
            MediaType::Video => {
                provider
                    .search_videos(&params.query, params.limit, params.page)
                    .await
            }
        }
    }

    /// 下载单个媒体项并跟踪进度
    pub async fn download_item(&self, item: &MediaItem) -> Result<String> {
        let start_time = Instant::now();
        let mut progress = DownloadProgress::new(item);

        // 通知: 开始
        progress.state = DownloadState::Starting;
        self.notify_progress(&progress);

        // 根据质量偏好确定 URL
        let url = match item.media_type {
            MediaType::Image => self.get_image_url(item)?,
            MediaType::Video => self.get_video_url(item)?,
        };

        // 生成文件名
        let filename = self.generate_filename(item);
        let output_path = Path::new(&self.config.output_dir).join(&filename);

        // 确保输出目录存在
        tokio::fs::create_dir_all(&self.config.output_dir).await?;

        // 开始下载
        progress.state = DownloadState::Downloading;
        self.notify_progress(&progress);

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            progress.state = DownloadState::Failed(format!("HTTP {}", response.status()));
            self.notify_progress(&progress);
            return Err(MediaError::DownloadError(format!(
                "HTTP {}: 下载失败",
                response.status()
            )));
        }

        // 从 Content-Length 头获取总大小
        progress.total_bytes = response.content_length();

        // 下载并跟踪进度
        let mut downloaded: u64 = 0;
        let mut last_update = Instant::now();
        let mut file = File::create(&output_path).await?;
        let mut stream = response.bytes_stream();

        use futures::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            let chunk_len = chunk.len() as u64;

            // 写入块
            file.write_all(&chunk).await?;
            downloaded += chunk_len;

            // 更新进度
            let elapsed = start_time.elapsed().as_secs_f64();
            progress.downloaded_bytes = downloaded;
            progress.elapsed_secs = elapsed;
            progress.speed_bps = if elapsed > 0.0 {
                (downloaded as f64 / elapsed) as u64
            } else {
                0
            };
            progress.calculate_percentage();
            progress.calculate_eta();

            // 节流更新（每 100ms）
            if last_update.elapsed().as_millis() >= 100 {
                self.notify_progress(&progress);
                last_update = Instant::now();
            }
        }

        // 最终更新
        progress.downloaded_bytes = downloaded;
        progress.elapsed_secs = start_time.elapsed().as_secs_f64();
        progress.calculate_percentage();

        // 写入磁盘
        progress.state = DownloadState::Writing;
        self.notify_progress(&progress);

        file.flush().await?;
        drop(file);

        // 完成
        progress.state = DownloadState::Completed;
        self.notify_progress(&progress);

        Ok(output_path.to_string_lossy().to_string())
    }

    /// 如果配置了进度回调，则通知进度
    fn notify_progress(&self, progress: &DownloadProgress) {
        if let Some(callback) = &self.config.progress_callback {
            callback(progress.clone());
        }
    }

    /// 并发批量下载多个媒体项，并跟踪整体进度
    pub async fn download_items(&self, items: &[MediaItem]) -> Vec<Result<String>> {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_concurrent));

        let futures: Vec<_> = items
            .iter()
            .map(|item| {
                let semaphore = Arc::clone(&semaphore);
                let item = item.clone();
                let downloader = self.clone();

                async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    downloader.download_item(&item).await
                }
            })
            .collect();

        join_all(futures).await
    }

    /// Download multiple items with overall batch progress callback
    pub async fn download_items_with_batch_progress<F>(
        &self,
        items: &[MediaItem],
        batch_callback: F,
    ) -> Vec<Result<String>>
    where
        F: Fn(BatchDownloadProgress) + Send + Sync + 'static,
    {
        use std::sync::Mutex;

        let batch_progress = Arc::new(Mutex::new(BatchDownloadProgress::new(items.len())));
        let batch_callback = Arc::new(batch_callback);

        // Clone downloader with custom progress callback
        let mut config = self.config.clone();
        let batch_progress_clone = Arc::clone(&batch_progress);
        let batch_callback_clone = Arc::clone(&batch_callback);

        config.progress_callback = Some(Arc::new(move |progress: DownloadProgress| {
            let mut batch = batch_progress_clone.lock().unwrap();

            // Update item progress
            if let Some(pos) = batch
                .item_progress
                .iter()
                .position(|p| p.item_id == progress.item_id)
            {
                batch.item_progress[pos] = progress.clone();
            } else {
                batch.item_progress.push(progress.clone());
            }

            // Update counters
            match progress.state {
                DownloadState::Downloading => {
                    batch.downloading_items = batch
                        .item_progress
                        .iter()
                        .filter(|p| matches!(p.state, DownloadState::Downloading))
                        .count();
                }
                DownloadState::Completed => {
                    batch.completed_items = batch
                        .item_progress
                        .iter()
                        .filter(|p| matches!(p.state, DownloadState::Completed))
                        .count();
                }
                DownloadState::Failed(_) => {
                    batch.failed_items = batch
                        .item_progress
                        .iter()
                        .filter(|p| matches!(p.state, DownloadState::Failed(_)))
                        .count();
                }
                _ => {}
            }

            batch.calculate_overall_percentage();
            batch_callback_clone(batch.clone());
        }));

        let downloader_with_callback = MediaDownloader {
            providers: self.providers.clone(),
            config,
            http_client: self.http_client.clone(),
        };

        downloader_with_callback.download_items(items).await
    }

    /// 根据 ID 下载媒体
    pub async fn download_by_id(&self, id: &str, media_type: MediaType) -> Result<String> {
        // 遍历所有提供商尝试获取媒体
        for provider in &self.providers {
            match provider.get_media(id, media_type.clone()).await {
                Ok(item) => {
                    // 找到媒体项，下载它
                    return self.download_item(&item).await;
                }
                Err(_) => {
                    // 当前提供商没有找到，继续尝试下一个
                    continue;
                }
            }
        }

        // 所有提供商都没有找到该媒体
        Err(MediaError::DownloadError(format!(
            "未找到 ID 为 {} 的媒体",
            id
        )))
    }

    /// 批量下载媒体项
    pub async fn download_batch(
        &self,
        items: &[&MediaItem],
        batch_callback: Option<ProgressCallback>,
    ) -> Result<Vec<String>> {
        let items_vec: Vec<MediaItem> = items.iter().map(|&item| item.clone()).collect();
        let results = self
            .download_items_with_batch_progress(&items_vec, move |progress| {
                if let Some(callback) = &batch_callback {
                    let progress = DownloadProgress {
                        item_id: "batch".to_string(),
                        item_title: format!(
                            "批量下载 ({}/{})",
                            progress.completed_items, progress.total_items
                        ),
                        provider: "聚合".to_string(),
                        state: DownloadState::Completed,
                        downloaded_bytes: 0,
                        total_bytes: None,
                        speed_bps: 0,
                        percentage: progress.overall_percentage,
                        elapsed_secs: 0.0,
                        eta_secs: None,
                    };
                    callback(progress);
                }
            })
            .await;

        let successful_downloads: Vec<String> = results
            .into_iter()
            .filter_map(|result| result.ok())
            .collect();

        Ok(successful_downloads)
    }

    /// 根据质量偏好获取图片 URL
    fn get_image_url(&self, item: &MediaItem) -> Result<String> {
        match self.config.image_quality {
            ImageQuality::Thumbnail => Ok(item.urls.thumbnail.clone()),
            ImageQuality::Medium => item
                .urls
                .medium
                .clone()
                .or_else(|| item.urls.large.clone())
                .or_else(|| Some(item.urls.thumbnail.clone()))
                .ok_or_else(|| MediaError::InvalidQuality("没有可用的中等质量".to_string())),
            ImageQuality::Large => item
                .urls
                .large
                .clone()
                .or_else(|| item.urls.medium.clone())
                .or_else(|| Some(item.urls.thumbnail.clone()))
                .ok_or_else(|| MediaError::InvalidQuality("没有可用的大尺寸质量".to_string())),
            ImageQuality::Original => item
                .urls
                .original
                .clone()
                .or_else(|| item.urls.large.clone())
                .or_else(|| item.urls.medium.clone())
                .or_else(|| Some(item.urls.thumbnail.clone()))
                .ok_or_else(|| MediaError::InvalidQuality("没有可用的原始质量".to_string())),
        }
    }

    /// 根据质量偏好获取视频 URL
    fn get_video_url(&self, item: &MediaItem) -> Result<String> {
        let video_files = item
            .urls
            .video_files
            .as_ref()
            .ok_or_else(|| MediaError::InvalidQuality("没有可用的视频文件".to_string()))?;

        let quality_str = self.config.video_quality.as_str();

        // 尝试查找精确的质量匹配
        if let Some(file) = video_files.iter().find(|f| f.quality == quality_str) {
            return Ok(file.url.clone());
        }

        // 尝试按分辨率查找
        let min_width = self.config.video_quality.min_width();
        if let Some(file) = video_files
            .iter()
            .filter(|f| f.width >= min_width)
            .min_by_key(|f| f.width)
        {
            return Ok(file.url.clone());
        }

        // 回退到最大可用
        video_files
            .iter()
            .max_by_key(|f| f.width)
            .map(|f| f.url.clone())
            .ok_or_else(|| MediaError::InvalidQuality("未找到合适的视频质量".to_string()))
    }

    /// 为媒体项生成文件名
    fn generate_filename(&self, item: &MediaItem) -> String {
        let extension = match item.media_type {
            MediaType::Image => "jpg",
            MediaType::Video => "mp4",
        };

        if self.config.use_original_names {
            format!("{}_{}.{}", item.provider.to_lowercase(), item.id, extension)
        } else {
            let sanitized_title = item
                .title
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect::<String>();

            let sanitized = if sanitized_title.is_empty() {
                item.id.clone()
            } else {
                sanitized_title
            };

            format!(
                "{}_{}_{}.{}",
                item.provider.to_lowercase(),
                sanitized,
                item.id,
                extension
            )
        }
    }
}

impl Clone for MediaDownloader {
    fn clone(&self) -> Self {
        Self {
            providers: self.providers.clone(),
            config: self.config.clone(),
            http_client: self.http_client.clone(),
        }
    }
}

impl Default for MediaDownloader {
    fn default() -> Self {
        Self::new()
    }
}
