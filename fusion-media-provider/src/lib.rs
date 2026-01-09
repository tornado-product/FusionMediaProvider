/*!
Poly Media Downloader - 多媒体下载库，支持从多个提供商（Pexels, Pixabay）搜索和下载图片及视频。
*/
mod pexels_provider;
mod downloader;
mod models;
mod error;
mod media_provider;
mod pixabay_provider;
mod create_provider;

pub use pixabay_provider::{PixabayProvider};
pub use downloader::{MediaDownloader, DownloadConfig, SearchParams};
pub use models::{AggregatedSearchResult, MediaItem, MediaType, MediaUrls, VideoFile, MediaMetadata, SearchResult, ImageQuality, VideoQuality, DownloadProgress, DownloadState, BatchDownloadProgress, ProgressCallback};
pub use error::{MediaError, Result};

#[cfg(feature = "pexels")]
pub use pexels_provider::PexelsProvider;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_downloader_creation() {
        let downloader = MediaDownloader::new();
        assert_eq!(downloader.providers().len(), 0);
    }
}