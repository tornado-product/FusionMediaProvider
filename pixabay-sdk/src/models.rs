use serde::{Deserialize, Serialize};

/// 图片搜索响应
///
/// 包含图片搜索结果的总数量和图片列表。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResponse {
    /// 结果总数
    pub total: u32,
    #[serde(rename = "totalHits")]
    /// 匹配结果总数
    pub total_hits: u32,
    /// 图片列表
    pub hits: Vec<Image>,
}

/// Pixabay 图片数据结构
///
/// 包含图片的所有信息，包括预览图、缩略图、高清图链接等。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    /// 图片唯一标识符
    pub id: u64,
    #[serde(rename = "pageURL")]
    /// 图片在 Pixabay 上的页面 URL
    pub page_url: String,
    #[serde(rename = "type")]
    /// 图片类型（如 photo、illustration、vector）
    pub image_type: String,
    /// 图片标签
    pub tags: String,
    #[serde(rename = "previewURL")]
    /// 预览图 URL（小尺寸）
    pub preview_url: String,
    #[serde(rename = "previewWidth")]
    /// 预览图宽度
    pub preview_width: u32,
    #[serde(rename = "previewHeight")]
    /// 预览图高度
    pub preview_height: u32,
    #[serde(rename = "webformatURL")]
    /// 缩略图 URL（中等尺寸）
    pub webformat_url: String,
    #[serde(rename = "webformatWidth")]
    /// 缩略图宽度
    pub webformat_width: u32,
    #[serde(rename = "webformatHeight")]
    /// 缩略图高度
    pub webformat_height: u32,
    #[serde(rename = "largeImageURL")]
    /// 大图 URL（高清分辨率）
    pub large_image_url: String,
    #[serde(rename = "fullHDURL", skip_serializing_if = "Option::is_none")]
    /// 全高清图 URL
    pub full_hd_url: Option<String>,
    #[serde(rename = "imageURL", skip_serializing_if = "Option::is_none")]
    /// 原始图片 URL
    pub image_url: Option<String>,
    #[serde(rename = "vectorURL", skip_serializing_if = "Option::is_none")]
    /// 矢量图 URL
    pub vector_url: Option<String>,
    #[serde(rename = "imageWidth")]
    /// 图片宽度
    pub image_width: u32,
    #[serde(rename = "imageHeight")]
    /// 图片高度
    pub image_height: u32,
    #[serde(rename = "imageSize")]
    /// 图片大小（字节）
    pub image_size: u64,
    /// 浏览次数
    pub views: u32,
    /// 下载次数
    pub downloads: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// 收藏次数
    pub collections: Option<u32>,
    /// 点赞次数
    pub likes: u32,
    /// 评论次数
    pub comments: u32,
    #[serde(rename = "user_id")]
    /// 上传用户 ID
    pub user_id: u64,
    /// 上传用户名
    pub user: String,
    #[serde(rename = "userImageURL")]
    /// 用户头像 URL
    pub user_image_url: String,
}

/// 视频搜索响应
///
/// 包含视频搜索结果的总数量和视频列表。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoResponse {
    /// 结果总数
    pub total: u32,
    #[serde(rename = "totalHits")]
    /// 匹配结果总数
    pub total_hits: u32,
    /// 视频列表
    pub hits: Vec<Video>,
}

/// Pixabay 视频数据结构
///
/// 包含视频的所有信息，包括不同分辨率的视频文件链接、时长、预览图等。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    /// 视频唯一标识符
    pub id: u64,
    #[serde(rename = "pageURL")]
    /// 视频在 Pixabay 上的页面 URL
    pub page_url: String,
    #[serde(rename = "type")]
    /// 视频类型（如 film、animation）
    pub video_type: String,
    /// 视频标签
    pub tags: String,
    /// 视频时长（秒）
    pub duration: u32,
    /// 视频文件链接（包含不同分辨率）
    pub videos: VideoFiles,
    /// 浏览次数
    pub views: u32,
    /// 下载次数
    pub downloads: u32,
    /// 点赞次数
    pub likes: u32,
    /// 评论次数
    pub comments: u32,
    #[serde(rename = "user_id")]
    /// 上传用户 ID
    pub user_id: u64,
    /// 上传用户名
    pub user: String,
    #[serde(rename = "userImageURL")]
    /// 用户头像 URL
    pub user_image_url: String,
}

/// 视频文件集合
///
/// 包含不同分辨率的视频文件链接。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFiles {
    /// 高分辨率视频（最大尺寸）
    pub large: Option<VideoFile>,
    /// 中等分辨率视频
    pub medium: Option<VideoFile>,
    /// 小分辨率视频
    pub small: Option<VideoFile>,
    /// 最小分辨率视频
    pub tiny: Option<VideoFile>,
}

/// 单个视频文件信息
///
/// 包含视频文件的具体信息，包括 URL、分辨率、文件大小和预览图。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFile {
    /// 视频文件 URL
    pub url: String,
    /// 视频宽度
    pub width: u32,
    /// 视频高度
    pub height: u32,
    /// 文件大小（字节）
    pub size: u64,
    /// 视频预览图
    pub thumbnail: String,
}

/// 图片类型枚举
///
/// 用于筛选搜索结果的图片类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageType {
    /// 所有类型
    All,
    /// 照片
    Photo,
    /// 插画
    Illustration,
    /// 矢量图
    Vector,
}

impl std::fmt::Display for ImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ImageType::All => "all",
            ImageType::Photo => "photo",
            ImageType::Illustration => "illustration",
            ImageType::Vector => "vector",
        };
        f.write_str(s)
    }
}

/// 视频类型枚举
///
/// 用于筛选搜索结果的视频类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoType {
    /// 所有类型
    All,
    /// 影片
    Film,
    /// 动画
    Animation,
}

impl std::fmt::Display for VideoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            VideoType::All => "all",
            VideoType::Film => "film",
            VideoType::Animation => "animation",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    All,
    Horizontal,
    Vertical,
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Orientation::All => "all",
            Orientation::Horizontal => "horizontal",
            Orientation::Vertical => "vertical",
        };
        f.write_str(s)
    }
}

/// 图片分类枚举
///
/// 用于筛选搜索结果的图片分类。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// 背景
    Backgrounds,
    /// 时尚
    Fashion,
    /// 自然
    Nature,
    /// 科学
    Science,
    /// 教育
    Education,
    /// 情感
    Feelings,
    /// 健康
    Health,
    /// 人物
    People,
    /// 宗教
    Religion,
    /// 地点
    Places,
    /// 动物
    Animals,
    /// 工业
    Industry,
    /// 计算机
    Computer,
    /// 食物
    Food,
    /// 体育
    Sports,
    /// 交通
    Transportation,
    /// 旅行
    Travel,
    /// 建筑
    Buildings,
    /// 商业
    Business,
    /// 音乐
    Music,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Category::Backgrounds => "backgrounds",
            Category::Fashion => "fashion",
            Category::Nature => "nature",
            Category::Science => "science",
            Category::Education => "education",
            Category::Feelings => "feelings",
            Category::Health => "health",
            Category::People => "people",
            Category::Religion => "religion",
            Category::Places => "places",
            Category::Animals => "animals",
            Category::Industry => "industry",
            Category::Computer => "computer",
            Category::Food => "food",
            Category::Sports => "sports",
            Category::Transportation => "transportation",
            Category::Travel => "travel",
            Category::Buildings => "buildings",
            Category::Business => "business",
            Category::Music => "music",
        };
        f.write_str(s)
    }
}

/// 结果排序枚举
///
/// 用于设置搜索结果的排序方式。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    /// 热门优先
    Popular,
    /// 最新优先
    Latest,
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Order::Popular => "popular",
            Order::Latest => "latest",
        };
        f.write_str(s)
    }
}

/// 搜索语言枚举
///
/// 用于设置搜索请求的语言（影响结果的语言偏好）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    /// 捷克语
    Cs,
    /// 丹麦语
    Da,
    /// 德语
    De,
    /// 英语
    En,
    /// 西班牙语
    Es,
    /// 法语
    Fr,
    /// 印尼语
    Id,
    /// 意大利语
    It,
    /// 匈牙利语
    Hu,
    /// 荷兰语
    Nl,
    /// 挪威语
    No,
    /// 波兰语
    Pl,
    /// 葡萄牙语
    Pt,
    /// 罗马尼亚语
    Ro,
    /// 斯洛伐克语
    Sk,
    /// 芬兰语
    Fi,
    /// 瑞典语
    Sv,
    /// 土耳其语
    Tr,
    /// 越南语
    Vi,
    /// 泰语
    Th,
    /// 保加利亚语
    Bg,
    /// 俄语
    Ru,
    /// 希腊语
    El,
    /// 日语
    Ja,
    /// 韩语
    Ko,
    /// 中文
    Zh,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Language::Cs => "cs",
            Language::Da => "da",
            Language::De => "de",
            Language::En => "en",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::Id => "id",
            Language::It => "it",
            Language::Hu => "hu",
            Language::Nl => "nl",
            Language::No => "no",
            Language::Pl => "pl",
            Language::Pt => "pt",
            Language::Ro => "ro",
            Language::Sk => "sk",
            Language::Fi => "fi",
            Language::Sv => "sv",
            Language::Tr => "tr",
            Language::Vi => "vi",
            Language::Th => "th",
            Language::Bg => "bg",
            Language::Ru => "ru",
            Language::El => "el",
            Language::Ja => "ja",
            Language::Ko => "ko",
            Language::Zh => "zh",
        };
        f.write_str(s)
    }
}
