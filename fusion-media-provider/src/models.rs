use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaType {
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "video")]
    Video,
}
impl FromStr for MediaType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "image" => Ok(MediaType::Image),
            "video" => Ok(MediaType::Video),
            _ => Err(format!("Invalid media type: {}", s)),
        }
    }
}
impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaType::Image => write!(f, "image"),
            MediaType::Video => write!(f, "video"),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoFile {
    pub quality: String,
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub thumbnail: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaUrls {
    /// 缩略图 URL（最小）
    pub thumbnail: String,
    /// 中等尺寸 URL
    pub medium: Option<String>,
    /// 大尺寸 URL
    pub large: Option<String>,
    /// 原始/完整尺寸 URL（可能需要完整 API 访问权限）
    pub original: Option<String>,
    /// 对于视频：不同分辨率选项
    pub video_files: Option<Vec<VideoFile>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
    pub width: u32,
    pub height: u32,
    pub size: Option<u64>,
    pub duration: Option<u32>, // 对于视频，单位为秒
    pub views: u32,
    pub downloads: u32,
    pub likes: u32,
}
/// 统一的媒体项，表示图片或视频
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: String,
    pub media_type: MediaType,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub author: String,
    pub author_url: String,
    pub source_url: String,
    pub provider: String,
    pub urls: MediaUrls,
    pub metadata: MediaMetadata,
}

/// 进度回调类型
pub type ProgressCallback = Arc<dyn Fn(DownloadProgress) + Send + Sync>;

/// 下载状态
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DownloadState {
    /// 开始下载
    Starting,
    /// 下载中
    Downloading,
    /// 写入磁盘
    Writing,
    /// 完成
    Completed,
    /// 失败（带错误信息）
    Failed(String),
}

/// 下载进度信息
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    /// 正在下载的项目 ID
    pub item_id: String,
    /// 项目标题
    pub item_title: String,
    /// 提供商名称
    pub provider: String,
    /// 当前状态
    pub state: DownloadState,
    /// 已下载字节数
    pub downloaded_bytes: u64,
    /// 总字节数（如果已知）
    pub total_bytes: Option<u64>,
    /// 下载速度（字节/秒）
    pub speed_bps: u64,
    /// 进度百分比 (0-100)
    pub percentage: f64,
    /// 已用时间（秒）
    pub elapsed_secs: f64,
    /// 预计剩余时间（秒）
    pub eta_secs: Option<f64>,
}
impl DownloadProgress {
    pub fn new(item: &MediaItem) -> Self {
        Self {
            item_id: item.id.clone(),
            item_title: item.title.clone(),
            provider: item.provider.clone(),
            state: DownloadState::Starting,
            downloaded_bytes: 0,
            total_bytes: None,
            speed_bps: 0,
            percentage: 0.0,
            elapsed_secs: 0.0,
            eta_secs: None,
        }
    }

    pub fn calculate_percentage(&mut self) {
        if let Some(total) = self.total_bytes {
            if total > 0 {
                self.percentage = (self.downloaded_bytes as f64 / total as f64) * 100.0;
            }
        }
    }

    pub fn calculate_eta(&mut self) {
        if let Some(total) = self.total_bytes {
            if self.speed_bps > 0 && self.downloaded_bytes < total {
                let remaining_bytes = total - self.downloaded_bytes;
                self.eta_secs = Some(remaining_bytes as f64 / self.speed_bps as f64);
            }
        }
    }

    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_idx = 0;

        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_idx])
    }

    pub fn format_speed(&self) -> String {
        Self::format_bytes(self.speed_bps) + "/s"
    }

    pub fn format_eta(&self) -> String {
        match self.eta_secs {
            Some(secs) if secs.is_finite() => {
                if secs < 60.0 {
                    format!("{:.0}秒", secs)
                } else if secs < 3600.0 {
                    format!("{:.0}分 {:.0}秒", secs / 60.0, secs % 60.0)
                } else {
                    format!("{:.0}小时 {:.0}分", secs / 3600.0, (secs % 3600.0) / 60.0)
                }
            }
            _ => "未知".to_string(),
        }
    }
}

/// 批量下载进度
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDownloadProgress {
    /// 总项目数
    pub total_items: usize,
    /// 已完成项目数
    pub completed_items: usize,
    /// 失败项目数
    pub failed_items: usize,
    /// 正在下载的项目数
    pub downloading_items: usize,
    /// 总体进度百分比 (0-100)
    pub overall_percentage: f64,
    /// 各项目的详细进度
    pub item_progress: Vec<DownloadProgress>,
}

impl BatchDownloadProgress {
    pub fn new(total_items: usize) -> Self {
        Self {
            total_items,
            completed_items: 0,
            failed_items: 0,
            downloading_items: 0,
            overall_percentage: 0.0,
            item_progress: Vec::new(),
        }
    }

    pub fn calculate_overall_percentage(&mut self) {
        if self.total_items > 0 {
            let completed_percentage =
                (self.completed_items as f64 / self.total_items as f64) * 100.0;
            let in_progress_percentage = self
                .item_progress
                .iter()
                .filter(|p| matches!(p.state, DownloadState::Downloading))
                .map(|p| p.percentage / self.total_items as f64)
                .sum::<f64>();

            self.overall_percentage = completed_percentage + in_progress_percentage;
        }
    }
}

/// 带分页信息的搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    /// 所有页面可用的结果总数
    pub total: u32,
    /// 当前响应中的结果数
    pub total_hits: u32,
    /// 当前页码
    pub page: u32,
    /// 每页结果数
    pub per_page: u32,
    /// 总页数
    pub total_pages: u32,
    /// 本页的媒体项
    pub items: Vec<MediaItem>,
    /// 提供商名称
    pub provider: String,
}

impl SearchResult {
    /// 从总结果数和每页数量计算总页数
    pub fn calculate_total_pages(total: u32, per_page: u32) -> u32 {
        if per_page == 0 {
            return 0;
        }
        (total + per_page - 1) / per_page // 向上取整
    }
}

/// 来自多个提供商的聚合搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregatedSearchResult {
    /// 搜索的提供商名称
    pub provider: String,
    /// 所有提供商的总结果数
    pub total: u32,
    /// 所有提供商的总命中数（可能有上限）
    pub total_hits: u32,
    /// 当前页码
    pub page: u32,
    /// 每页结果数
    pub per_page: u32,
    /// 所有提供商的总页数
    pub total_pages: u32,
    /// 来自所有提供商的媒体项
    pub items: Vec<MediaItem>,
    /// 各提供商的详细结果
    pub provider_results: Vec<SearchResult>,
}

/// 图片质量偏好
#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ImageQuality {
    Thumbnail,
    Medium,
    Large,
    Original,
}

impl ImageQuality {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageQuality::Thumbnail => "thumbnail",
            ImageQuality::Medium => "medium",
            ImageQuality::Large => "large",
            ImageQuality::Original => "original",
        }
    }
}

/// 视频质量偏好
#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum VideoQuality {
    Tiny,   // 360p
    Small,  // 540p
    Medium, // 720p
    Large,  // 1080p
    Original,
}

impl VideoQuality {
    pub fn as_str(&self) -> &'static str {
        match self {
            VideoQuality::Tiny => "tiny",
            VideoQuality::Small => "small",
            VideoQuality::Medium => "medium",
            VideoQuality::Large => "large",
            VideoQuality::Original => "original",
        }
    }

    pub fn min_width(&self) -> u32 {
        match self {
            VideoQuality::Tiny => 640,
            VideoQuality::Small => 960,
            VideoQuality::Medium => 1280,
            VideoQuality::Large => 1920,
            VideoQuality::Original => 0,
        }
    }
}
