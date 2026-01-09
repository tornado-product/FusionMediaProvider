/*!
`pexels-sdk` crate 提供了 Pexels API 的封装库。基于 [Pexels API 文档](https://www.pexels.com/api/documentation/)。

要获取 API 密钥，您需要从 [申请 API 访问 - Pexels](https://www.pexels.com/api/new/) 申请。

本库依赖 [serde-json](https://github.com/serde-rs/json) crate 来处理结果。因此，您需要阅读 [serde_json - Rust](https://docs.serde.rs/serde_json/index.html) 的文档，特别是 [serde_json::Value - Rust](https://docs.serde.rs/serde_json/enum.Value.html)。

# 配置

在 `Cargo.toml` 文件中的 `[dependencies]` 部分添加以下行：

```toml
pexels-sdk = "*"
```

并在您的 crate 根文件（如 `main.rs`）中添加：

```rust
use pexels_sdk;
```

完成！现在您可以使用此 API 封装库。

# 示例

此示例展示了如何获取*山脉*照片列表。

```rust
use dotenvy::dotenv;
use std::env;
use pexels_sdk::{PexelsClient, SearchParams};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
    let pexels_client = PexelsClient::new(api_key);
    let params = SearchParams::new()
        .page(1)
        .per_page(15);
    let response = pexels_client.search_photos("mountains", &params).await.expect("Failed to get photos");
    println!("{:?}", response);
}
```

您可以使用 `cargo run` 来运行它！就是如此简单。

# 随机照片

如果您想获取随机照片，可以使用 `curated_photos` 函数并将 `per_page` 设置为 1，`page` 设置为 1 到 1000 之间的随机数，以获取漂亮的随机照片。如果您想要获取特定主题的随机照片，也可以对热门搜索使用相同的方法。

# 图片格式

* original - 原始图片的大小由宽度和高度属性给出。
* large - 此图片最大宽度为 940 像素，最大高度为 650 像素。它保持原始图片的宽高比。
* large2x - 此图片最大宽度为 1880 像素，最大高度为 1300 像素。它保持原始图片的宽高比。
* medium - 此图片高度为 350 像素，宽度灵活。它保持原始图片的宽高比。
* small - 此图片高度为 130 像素，宽度灵活。它保持原始图片的宽高比。
* portrait - 此图片宽度为 800 像素，高度为 1200 像素。
* landscape - 此图片宽度为 1200 像素，高度为 627 像素。
* tiny - 此图片宽度为 280 像素，高度为 200 像素。
*/

mod client;
mod collections;
mod domain;
mod download;
mod models;
mod photos;
mod search;
mod videos;

/// collections 模块
pub use collections::featured::Featured;
pub use collections::featured::FeaturedBuilder;
pub use collections::items::Collections;
pub use collections::items::CollectionsBuilder;
pub use collections::media::Media;
pub use collections::media::MediaBuilder;
/// domain 模块
pub use domain::models::Collection;
pub use domain::models::CollectionsResponse;
pub use domain::models::MediaPhoto;
pub use domain::models::MediaResponse;
pub use domain::models::MediaType as MediaTypeResponse;
pub use domain::models::MediaVideo;
pub use domain::models::Photo;
pub use domain::models::PhotoSrc;
pub use domain::models::PhotosResponse;
pub use domain::models::User;
pub use domain::models::Video;
pub use domain::models::VideoFile;
pub use domain::models::VideoPicture;
pub use domain::models::VideoResponse;
/// photos 模块
pub use photos::curated::Curated;
pub use photos::curated::CuratedBuilder;
pub use photos::photo::FetchPhoto;
pub use photos::photo::FetchPhotoBuilder;
pub use photos::search::Color;
pub use photos::search::Hex;
pub use photos::search::Search;
pub use photos::search::SearchBuilder;
/// videos 模块
pub use videos::popular::Popular;
pub use videos::popular::PopularBuilder;
pub use videos::search::Search as VideoSearch;
pub use videos::search::SearchBuilder as VideoSearchBuilder;
pub use videos::video::FetchVideo;
pub use videos::video::FetchVideoBuilder;

pub use client::PexelsClient;
pub use search::SearchParams;

pub use download::DownloadManager;
pub use download::ProgressCallback;

/// 导入依赖包
use reqwest::Client;
use reqwest::Error as ReqwestError;
use serde_json::Error as JSONError;
use serde_json::Value;
use std::env::VarError;
use std::fmt::Display;
use std::str::FromStr;
use thiserror::Error;
use url::ParseError;

/// Pexels API 版本
const PEXELS_VERSION: &str = "v1";

/// 视频路径
const PEXELS_VIDEO_PATH: &str = "videos";

/// 收藏路径
const PEXELS_COLLECTIONS_PATH: &str = "collections";

/// Pexels API URL
const PEXELS_API: &str = "https://api.pexels.com";

/// 期望的照片方向。
/// 支持的值：`landscape`、`portrait`、`square`。
/// 默认值：`landscape`。
///
/// # 示例
/// ```rust
/// use pexels_sdk::Orientation;
/// use std::str::FromStr;
///
/// let orientation = Orientation::from_str("landscape").unwrap();
/// assert_eq!(orientation, Orientation::Landscape);
/// ```
#[derive(PartialEq, Debug, Clone)]
pub enum Orientation {
    Landscape,
    Portrait,
    Square,
}

impl Orientation {
    fn as_str(&self) -> &str {
        match self {
            Orientation::Landscape => "landscape",
            Orientation::Portrait => "portrait",
            Orientation::Square => "square",
        }
    }
}

impl FromStr for Orientation {
    type Err = PexelsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "landscape" => Ok(Orientation::Landscape),
            "portrait" => Ok(Orientation::Portrait),
            "square" => Ok(Orientation::Square),
            _ => Err(PexelsError::ParseMediaSortError),
        }
    }
}

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Orientation::Landscape => "landscape".to_string(),
            Orientation::Portrait => "portrait".to_string(),
            Orientation::Square => "square".to_string(),
        };
        write!(f, "{str}")
    }
}

/// 指定媒体集合中的项目顺序。
/// 支持的值：`asc`、`desc`。默认值：`asc`。
///
/// # 示例
/// ```rust
/// use pexels_sdk::MediaSort;
/// use std::str::FromStr;
///
/// let sort = MediaSort::from_str("asc").unwrap();
/// assert_eq!(sort, MediaSort::Asc);
/// ```
#[derive(PartialEq, Debug)]
pub enum MediaSort {
    Asc,
    Desc,
}

impl MediaSort {
    fn as_str(&self) -> &str {
        match self {
            MediaSort::Asc => "asc",
            MediaSort::Desc => "desc",
        }
    }
}

impl FromStr for MediaSort {
    type Err = PexelsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asc" => Ok(MediaSort::Asc),
            "desc" => Ok(MediaSort::Desc),
            _ => Err(PexelsError::ParseMediaSortError),
        }
    }
}

/// 指定要请求的媒体类型。
/// 如果未提供或无效，将返回所有媒体类型。
/// 支持的值：`photos`、`videos`。
///
/// # 示例
/// ```rust
/// use pexels_sdk::MediaType;
/// use std::str::FromStr;
///
/// let media_type = MediaType::from_str("photos");
/// match media_type {
///     Ok(mt) => assert_eq!(mt, MediaType::Photo),
///     Err(e) => eprintln!("Error parsing media type: {:?}", e),
/// }
/// ```
#[derive(PartialEq, Debug)]
pub enum MediaType {
    Photo,
    Video,
    Empty,
}

impl MediaType {
    fn as_str(&self) -> &str {
        match self {
            MediaType::Photo => "photos",
            MediaType::Video => "videos",
            MediaType::Empty => "",
        }
    }
}

impl FromStr for MediaType {
    type Err = PexelsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "photo" => Ok(MediaType::Photo),
            "video" => Ok(MediaType::Video),
            "" => Ok(MediaType::Empty),
            _ => Err(PexelsError::ParseMediaTypeError),
        }
    }
}

/// 指定搜索查询的语言环境。
/// 支持的值：`en-US`、`pt-BR`、`es-ES`、`ca-ES`、`de-DE`、`it-IT`、`fr-FR`、`sv-SE`、`id-ID`、`pl-PL`、`ja-JP`、`zh-TW`、`zh-CN`、`ko-KR`、`th-TH`、`nl-NL`、`hu-HU`、`vi-VN`、`cs-CZ`、`da-DK`、`fi-FI`、`uk-UA`、`el-GR`、`ro-RO`、`nb-NO`、`sk-SK`、`tr-TR`、`ru-RU`。
/// 默认值：`en-US`。
///
/// # 示例
/// ```rust
/// use pexels_sdk::Locale;
/// use std::str::FromStr;
///
/// let locale = Locale::from_str("en-US").unwrap();
/// assert_eq!(locale, Locale::en_US);
/// ```
#[allow(non_camel_case_types)]
#[derive(PartialEq, Debug)]
pub enum Locale {
    en_US,
    pt_BR,
    es_ES,
    ca_ES,
    de_DE,
    it_IT,
    fr_FR,
    sv_SE,
    id_ID,
    pl_PL,
    ja_JP,
    zh_TW,
    zh_CN,
    ko_KR,
    th_TH,
    nl_NL,
    hu_HU,
    vi_VN,
    cs_CZ,
    da_DK,
    fi_FI,
    uk_UA,
    el_GR,
    ro_RO,
    nb_NO,
    sk_SK,
    tr_TR,
    ru_RU,
}

impl Locale {
    fn as_str(&self) -> &str {
        match self {
            Locale::en_US => "en-US",
            Locale::pt_BR => "pt-BR",
            Locale::es_ES => "es-ES",
            Locale::ca_ES => "ca-ES",
            Locale::de_DE => "de-DE",
            Locale::it_IT => "it-IT",
            Locale::fr_FR => "fr-FR",
            Locale::sv_SE => "sv-SE",
            Locale::id_ID => "id-ID",
            Locale::pl_PL => "pl-PL",
            Locale::ja_JP => "ja-JP",
            Locale::zh_TW => "zh-TW",
            Locale::zh_CN => "zh-CN",
            Locale::ko_KR => "ko-KR",
            Locale::th_TH => "th-TH",
            Locale::nl_NL => "nl-NL",
            Locale::hu_HU => "hu-HU",
            Locale::vi_VN => "vi-VN",
            Locale::cs_CZ => "cs-CZ",
            Locale::da_DK => "da-DK",
            Locale::fi_FI => "fi-FI",
            Locale::uk_UA => "uk-UA",
            Locale::el_GR => "el-GR",
            Locale::ro_RO => "ro-RO",
            Locale::nb_NO => "nb-NO",
            Locale::sk_SK => "sk-SK",
            Locale::tr_TR => "tr-TR",
            Locale::ru_RU => "-ES",
        }
    }
}

impl FromStr for Locale {
    type Err = PexelsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en-us" => Ok(Locale::en_US),
            "pt-br" => Ok(Locale::pt_BR),
            "es-es" => Ok(Locale::es_ES),
            "ca-es" => Ok(Locale::ca_ES),
            "de-de" => Ok(Locale::de_DE),
            "it-it" => Ok(Locale::it_IT),
            "fr-fr" => Ok(Locale::fr_FR),
            "sv-se" => Ok(Locale::sv_SE),
            "id-id" => Ok(Locale::id_ID),
            "pl-pl" => Ok(Locale::pl_PL),
            "ja-jp" => Ok(Locale::ja_JP),
            "zh-tw" => Ok(Locale::zh_TW),
            "zh-cn" => Ok(Locale::zh_CN),
            "ko-kr" => Ok(Locale::ko_KR),
            "th-th" => Ok(Locale::th_TH),
            "nl-nl" => Ok(Locale::nl_NL),
            "hu-hu" => Ok(Locale::hu_HU),
            "vi-vn" => Ok(Locale::vi_VN),
            "cs-cz" => Ok(Locale::cs_CZ),
            "da-dk" => Ok(Locale::da_DK),
            "fi-fi" => Ok(Locale::fi_FI),
            "uk-ua" => Ok(Locale::uk_UA),
            "el-gr" => Ok(Locale::el_GR),
            "ro-ro" => Ok(Locale::ro_RO),
            "nb-no" => Ok(Locale::nb_NO),
            "sk-sk" => Ok(Locale::sk_SK),
            "tr-tr" => Ok(Locale::tr_TR),
            "ru-ru" => Ok(Locale::ru_RU),
            _ => Err(PexelsError::ParseLocaleError),
        }
    }
}

/// 指定视频或照片的最小尺寸。
/// 支持的值：`large`、`medium`、`small`。
///
/// # 示例
/// ```rust
/// use pexels_sdk::Size;
/// use std::str::FromStr;
///
/// let size = Size::from_str("large").unwrap();
/// assert_eq!(size, Size::Large);
/// ```
#[derive(PartialEq, Debug, Clone)]
pub enum Size {
    Large,
    Medium,
    Small,
}

impl Size {
    fn as_str(&self) -> &str {
        match self {
            Size::Large => "large",
            Size::Medium => "medium",
            Size::Small => "small",
        }
    }
}

impl FromStr for Size {
    type Err = PexelsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "large" => Ok(Size::Large),
            "medium" => Ok(Size::Medium),
            "small" => Ok(Size::Small),
            _ => Err(PexelsError::ParseSizeError),
        }
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Size::Large => "large".to_string(),
            Size::Medium => "medium".to_string(),
            Size::Small => "small".to_string(),
        };
        write!(f, "{str}")
    }
}

/// 构建器返回的结果的类型别名。
pub(crate) type BuilderResult = Result<String, PexelsError>;

/// 与 Pexels API 交互时可能发生的错误。
/// 此枚举作为与 API 交互的函数的返回类型。
///
/// # 示例
/// ```rust
/// use pexels_sdk::PexelsError;
///
/// let error = PexelsError::ParseMediaTypeError;
/// assert_eq!(error.to_string(), "解析媒体类型失败: 无效的值");
/// ```
#[derive(Debug, Error)]
pub enum PexelsError {
    #[error("发送 HTTP 请求失败: {0}")]
    RequestError(#[from] ReqwestError),
    #[error("解析 JSON 响应失败: {0}")]
    JsonParseError(#[from] JSONError),
    #[error("环境变量中未找到 API 密钥: {0}")]
    EnvVarError(#[from] VarError),
    #[error("环境变量中未找到 API 密钥")]
    ApiKeyNotFound,
    #[error("解析 URL 失败: {0}")]
    ParseError(#[from] ParseError),
    #[error("无效的十六进制颜色码: {0}")]
    HexColorCodeError(String),
    #[error("解析媒体类型失败: 无效的值")]
    ParseMediaTypeError,
    #[error("解析媒体排序失败: 无效的值")]
    ParseMediaSortError,
    #[error("解析方向失败: 无效的值")]
    ParseOrientationError,
    #[error("解析尺寸失败: 无效的值")]
    ParseSizeError,
    #[error("解析语言环境失败: 无效的值")]
    ParseLocaleError,
    #[error("下载错误: {0}")]
    DownloadError(String),
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    #[error("API 错误: {0}")]
    ApiError(String),
    #[error("超出速率限制")]
    RateLimitError,
    #[error("认证错误: {0}")]
    AuthError(String),
    #[error("无效的参数: {0}")]
    InvalidParameter(String),
    #[error("未找到资源: {0}")]
    NotFound(String),
    #[error("异步任务错误")]
    AsyncError,
    #[error("未知错误: {0}")]
    Unknown(String),
}

// Manual implementation PartialEq
impl PartialEq for PexelsError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare RequestError
            (PexelsError::RequestError(e1), PexelsError::RequestError(e2)) => {
                e1.to_string() == e2.to_string()
            }
            // Compare JsonParseError
            (PexelsError::JsonParseError(e1), PexelsError::JsonParseError(e2)) => {
                e1.to_string() == e2.to_string()
            }
            // Compare ApiKeyNotFound
            (PexelsError::ApiKeyNotFound, PexelsError::ApiKeyNotFound) => true,
            // Compare ParseError
            (PexelsError::ParseError(e1), PexelsError::ParseError(e2)) => {
                e1.to_string() == e2.to_string()
            }
            // Compare HexColorCodeError
            (PexelsError::HexColorCodeError(msg1), PexelsError::HexColorCodeError(msg2)) => {
                msg1 == msg2
            }
            // Other things are not equal
            _ => false,
        }
    }
}

/// 用于与 Pexels API 交互的客户端
///
/// # 示例
/// ```rust
/// use dotenvy::dotenv;
/// use pexels_sdk::PexelsClient;
/// use std::env;
///
/// #[tokio::main]
/// async fn main() {
///    dotenv().ok();
///   let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
///  let client = PexelsClient::new(api_key);
/// }
/// ```
///
/// # 错误
/// 如果请求失败或响应无法解析为 JSON，则返回 `PexelsError`。
///
/// # 示例
/// ```rust
/// use dotenvy::dotenv;
/// use pexels_sdk::PexelsClient;
/// use pexels_sdk::SearchParams;
/// use std::env;
///
/// #[tokio::main]
/// async fn main() {
///     dotenv().ok();
///     let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
///     let client = PexelsClient::new(api_key);
///     let params = SearchParams::new().page(1).per_page(15);
///     let response = client.search_photos("mountains", &params).await.expect("Failed to get photos");
///     println!("{:?}", response);
/// }
/// ```
pub struct Pexels {
    client: Client,
    api_key: String,
}

impl Pexels {
    /// 创建新的 Pexels 客户端。
    ///
    /// # 参数
    /// * `api_key` - Pexels API 的 API 密钥。
    ///
    /// # 示例
    /// ```rust
    /// use dotenvy::dotenv;
    /// use pexels_sdk::PexelsClient;
    /// use std::env;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv().ok();
    ///     let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
    ///     let client = PexelsClient::new(api_key);
    /// }
    /// ```
    pub fn new(api_key: String) -> Self {
        Pexels {
            client: Client::new(),
            api_key,
        }
    }

    /// 向指定 URL 发送 HTTP GET 请求并返回 JSON 响应。
    /// 使用 `reqwest` crate 发送 HTTP 请求。
    ///
    /// # 错误
    /// 如果请求失败或响应无法解析为 JSON，则返回 `PexelsError`。
    async fn make_request(&self, url: &str) -> Result<Value, PexelsError> {
        let json_response = self
            .client
            .get(url)
            .header("Authorization", &self.api_key)
            .send()
            .await?
            .json::<Value>()
            .await?;
        Ok(json_response)
    }

    /// 根据搜索条件从 Pexels API 检索照片列表。
    ///
    /// # 参数
    /// * `builder` - 带有搜索参数的 `SearchBuilder` 实例。
    ///
    /// # 错误
    /// 如果请求失败或响应无法解析为 JSON，则返回 `PexelsError`。
    ///
    /// # 示例
    /// ```rust
    /// use dotenvy::dotenv;
    /// use pexels_sdk::PexelsClient;
    /// use pexels_sdk::SearchParams;
    /// use std::env;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv().ok();
    ///     let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
    ///     let client = PexelsClient::new(api_key);
    ///     let params = SearchParams::new().page(1).per_page(15);
    ///     let response = client.search_photos("mountains", &params).await.expect("Failed to get photos");
    ///     println!("{:?}", response);
    /// }
    /// ```
    pub async fn search_photos(
        &self,
        builder: SearchBuilder<'_>,
    ) -> Result<PhotosResponse, PexelsError> {
        builder.build().fetch(self).await
    }

    /// 根据 ID 从 Pexels API 检索照片。
    ///
    /// # 参数
    /// * `id` - 要检索的照片的 ID。
    ///
    /// # 错误
    /// 如果请求失败或响应无法解析为 JSON，则返回 `PexelsError`。
    ///
    /// # 示例
    /// ```rust
    /// use dotenvy::dotenv;
    /// use pexels_sdk::PexelsClient;
    /// use std::env;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv().ok();
    ///     let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
    ///     let client = PexelsClient::new(api_key);
    ///     let response = client.get_photo(10967).await.expect("Failed to get photo");
    ///     println!("{:?}", response);
    /// }
    /// ```
    pub async fn get_photo(&self, id: usize) -> Result<Photo, PexelsError> {
        FetchPhotoBuilder::new().id(id).build().fetch(self).await
    }

    /// 从 Pexels API 检索随机照片。
    ///
    /// # 参数
    /// * `builder` - 带有搜索参数的 `CuratedBuilder` 实例。
    ///
    /// # 错误
    /// 如果请求失败或响应无法解析为 JSON，则返回 `PexelsError`。  
    ///
    /// # Example
    /// ```rust
    /// use dotenvy::dotenv;
    /// use pexels_sdk::Pexels;
    /// use pexels_sdk::CuratedBuilder;
    /// use std::env;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv().ok();
    ///     let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
    ///     let client = Pexels::new(api_key);
    ///     let response = client.curated_photo(CuratedBuilder::new().per_page(1).page(1)).await.expect("Failed to get random photo");
    ///     println!("{:?}", response);
    /// }
    /// ```                 
    pub async fn curated_photo(
        &self,
        builder: CuratedBuilder,
    ) -> Result<PhotosResponse, PexelsError> {
        builder.build().fetch(self).await
    }

    /// Retrieves a list of videos from the Pexels API based on the search criteria.
    ///
    /// # Arguments
    /// * `builder` - A `VideoSearchBuilder` instance with the search parameters.
    ///
    /// # Errors
    /// Returns a `PexelsError` if the request fails or the response cannot be parsed as JSON.
    ///
    /// # Example
    /// ```rust
    /// use dotenvy::dotenv;
    /// use pexels_sdk::Pexels;
    /// use pexels_sdk::VideoSearchBuilder;
    /// use std::env;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv().ok();
    ///     let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
    ///     let client = Pexels::new(api_key);
    ///     let response = client.search_videos(VideoSearchBuilder::new().query("nature").per_page(15).page(1)).await.expect("Failed to get videos");
    ///     println!("{:?}", response);
    /// }
    /// ```                 
    pub async fn search_videos(
        &self,
        builder: VideoSearchBuilder<'_>,
    ) -> Result<VideoResponse, PexelsError> {
        builder.build().fetch(self).await
    }

    /// Retrieves a list of popular videos from the Pexels API.
    ///
    /// # Arguments
    /// * `builder` - A `PopularBuilder` instance with the search parameters.
    ///
    /// # Errors
    /// Returns a `PexelsError` if the request fails or the response cannot be parsed as JSON.
    ///
    /// # Example
    /// ```rust
    /// use dotenvy::dotenv;
    /// use pexels_sdk::Pexels;
    /// use pexels_sdk::PopularBuilder;
    /// use std::env;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv().ok();
    ///     let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
    ///     let client = Pexels::new(api_key);
    ///     let response = client.popular_videos(PopularBuilder::new().per_page(15).page(1)).await.expect("Failed to get popular videos");
    ///     println!("{:?}", response);
    /// }
    /// ```                
    pub async fn popular_videos(
        &self,
        builder: PopularBuilder,
    ) -> Result<VideoResponse, PexelsError> {
        builder.build().fetch(self).await
    }

    /// Retrieves a video by its ID from the Pexels API.
    ///
    /// # Arguments
    /// * `id` - The ID of the video to retrieve.
    ///
    /// # Errors
    /// Returns a `PexelsError` if the request fails or the response cannot be parsed as JSON.
    ///
    /// # Example
    /// ```rust
    /// use dotenvy::dotenv;
    /// use pexels_sdk::Pexels;
    /// use std::env;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv().ok();
    ///     let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
    ///     let client = Pexels::new(api_key);
    ///     let response = client.get_video(25460961).await.expect("Failed to get video");
    ///     println!("{:?}", response);
    /// }
    /// ```
    pub async fn get_video(&self, id: usize) -> Result<Video, PexelsError> {
        FetchVideoBuilder::new().id(id).build().fetch(self).await
    }

    /// Retrieves a list of collections from the Pexels API.
    ///
    /// # Arguments
    /// * `per_page` - The number of collections to retrieve per page.
    /// * `page` - The page number to retrieve.
    ///
    /// # Errors
    /// Returns a `PexelsError` if the request fails or the response cannot be parsed as JSON.
    ///
    /// # Example
    /// ```rust
    /// use dotenvy::dotenv;
    /// use pexels_sdk::Pexels;
    /// use std::env;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv().ok();
    ///     let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
    ///     let client = Pexels::new(api_key);
    ///     let response = client.search_collections(15, 1).await.expect("Failed to get collections");
    ///     println!("{:?}", response);
    /// }      
    /// ```          
    pub async fn search_collections(
        &self,
        per_page: usize,
        page: usize,
    ) -> Result<CollectionsResponse, PexelsError> {
        CollectionsBuilder::new()
            .per_page(per_page)
            .page(page)
            .build()
            .fetch(self)
            .await
    }

    /// Retrieves a list of featured collections from the Pexels API.
    ///
    /// # Arguments
    /// * `per_page` - The number of collections to retrieve per page.
    /// * `page` - The page number to retrieve.
    ///
    /// # Errors
    /// Returns a `PexelsError` if the request fails or the response cannot be parsed as JSON.
    ///
    /// # Example
    /// ```rust
    /// use dotenvy::dotenv;
    /// use pexels_sdk::Pexels;
    /// use std::env;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv().ok();
    ///     let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
    ///     let client = Pexels::new(api_key);
    ///     let response = client.featured_collections(15, 1).await.expect("Failed to get collections");
    ///     println!("{:?}", response);
    /// }
    /// ```
    pub async fn featured_collections(
        &self,
        per_page: usize,
        page: usize,
    ) -> Result<CollectionsResponse, PexelsError> {
        FeaturedBuilder::new()
            .per_page(per_page)
            .page(page)
            .build()
            .fetch(self)
            .await
    }

    /// Retrieves all media (photos and videos) within a single collection.
    ///
    /// # Arguments
    /// * `builder` - A `MediaBuilder` instance with the search parameters.
    ///
    /// # Errors
    /// Returns a `PexelsError` if the request fails or the response cannot be parsed as JSON.
    ///
    /// # Example
    /// ```rust,no_run
    /// use dotenvy::dotenv;
    /// use pexels_sdk::Pexels;
    /// use pexels_sdk::MediaBuilder;
    /// use std::env;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     dotenv().ok();
    ///     let api_key = env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
    ///     let client = Pexels::new(api_key);
    ///     let builder = MediaBuilder::new().id("your_collection_id".to_string()).per_page(15).page(1);
    ///     let response = client.search_media(builder).await.expect("Failed to get media");
    ///     println!("Found {} media items", response.total_results);
    /// }                 
    pub async fn search_media(&self, builder: MediaBuilder) -> Result<MediaResponse, PexelsError> {
        builder.build().fetch(self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;

    #[test]
    fn test_pexels_error_partial_eq() {
        let err1 = PexelsError::ApiKeyNotFound;
        let err2 = PexelsError::ApiKeyNotFound;
        assert_eq!(err1, err2);

        let err3 = PexelsError::HexColorCodeError(String::from("Invalid color"));
        let err4 = PexelsError::HexColorCodeError(String::from("Invalid color"));
        assert_eq!(err3, err4);

        let err9 = PexelsError::ParseError(ParseError::EmptyHost);
        let err10 = PexelsError::ParseError(ParseError::EmptyHost);
        assert_eq!(err9, err10);

        // 测试不相等的情况
        let err11 = PexelsError::ApiKeyNotFound;
        let err12 = PexelsError::HexColorCodeError(String::from("Invalid color"));
        assert_ne!(err11, err12);
    }

    #[test]
    fn test_parse_photo() {
        let input = "photo";
        let media_type = input.parse::<MediaType>();
        assert_eq!(media_type, Ok(MediaType::Photo));
    }

    #[test]
    fn test_parse_video() {
        let input = "video";
        let media_type = input.parse::<MediaType>();
        assert_eq!(media_type, Ok(MediaType::Video));
    }

    #[test]
    fn test_parse_invalid() {
        let input = "audio";
        let media_type = input.parse::<MediaType>();
        assert!(matches!(media_type, Err(PexelsError::ParseMediaTypeError)));
    }

    #[tokio::test]
    #[ignore]
    async fn test_make_request() {
        dotenv().ok();
        let api_key = std::env::var("PEXELS_API_KEY").expect("PEXELS_API_KEY not set");
        let client = Pexels::new(api_key);
        let url = "https://api.pexels.com/v1/curated";
        let response = client.make_request(url).await;
        assert!(response.is_ok());
    }
}
