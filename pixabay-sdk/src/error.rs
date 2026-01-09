use thiserror::Error;

#[derive(Error, Debug)]
pub enum PixabayError {
    #[error("发送 HTTP 请求失败: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("解析 JSON 响应失败: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("解析 URL 失败: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("API 错误: {0}")]
    ApiError(String),

    #[error("超过速率限制")]
    RateLimitExceeded,

    #[error("无效的 API 密钥")]
    InvalidApiKey,
}

pub type Result<T> = std::result::Result<T, PixabayError>;