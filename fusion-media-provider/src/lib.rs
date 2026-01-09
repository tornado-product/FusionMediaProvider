/*!
Poly Media Downloader - 多媒体下载库，支持从多个提供商（Pexels, Pixabay）搜索和下载图片及视频。
*/
mod create_provider;
mod downloader;
mod error;
mod media_provider;
mod models;
mod pexels_provider;
mod pixabay_provider;

pub use downloader::{DownloadConfig, MediaDownloader, SearchParams};
pub use error::{MediaError, Result};
pub use models::{
    AggregatedSearchResult, BatchDownloadProgress, DownloadProgress, DownloadState, ImageQuality,
    MediaItem, MediaMetadata, MediaType, MediaUrls, ProgressCallback, SearchResult, VideoFile,
    VideoQuality,
};
pub use pixabay_provider::PixabayProvider;

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
