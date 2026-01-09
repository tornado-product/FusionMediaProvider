/*!
错误处理模块 - 定义多媒体下载过程中可能出现的错误类型。
*/
use thiserror::Error;

/// 多媒体下载错误枚举
#[derive(Error, Debug)]
pub enum MediaError {
    #[error("API 密钥未设置或为空")]
    ApiKeyIsEmpty,

    #[error("Pixabay 错误: {0}")]
    PixabayError(#[from] pixabay_sdk::PixabayError),

    #[cfg(feature = "pexels")]
    #[error("Pexels 错误: {0}")]
    PexelsError(String),

    #[error("未配置任何提供商")]
    NoProviders,

    #[error("所有提供商均失败，可能是 API 密钥未设置或为空")]
    AllProvidersFailed,

    #[error("下载错误: {0}")]
    DownloadError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP 错误: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("无效的质量选项: {0}")]
    InvalidQuality(String),

    #[error("未知的提供商")]
    UnknownProvider(String),

    #[error("该提供商未启用")]
    ProviderNotEnabled(String),
}

/// 操作结果类型别名
pub type Result<T> = std::result::Result<T, MediaError>;
