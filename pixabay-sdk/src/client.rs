use crate::error::{PixabayError, Result};
use crate::models::*;
use reqwest::Client;
use url::Url;

const BASE_URL: &str = "https://pixabay.com/api/";
const VIDEO_BASE_URL: &str = "https://pixabay.com/api/videos/";

#[derive(Debug, Clone)]
pub struct Pixabay {
    pub api_key: String,
    client: Client,
}

impl Pixabay {
    /// 创建一个新的 Pixabay 客户端
    ///
    /// # 参数
    ///
    /// * `api_key` - 你的 Pixabay API 密钥
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    /// 在 Pixabay 上搜索图片
    ///
    /// # 参数
    ///
    /// * `query` - 搜索查询词（会自动进行 URL 编码，最多 100 字符）
    /// * `per_page` - 每页结果数（3-200，默认：20）
    /// * `page` - 页码（默认：1）
    ///
    /// # 可选参数（使用 SearchImageParams 构建器模式）
    ///
    /// 对于带有更多参数的高级搜索，请使用 `search_images_advanced()` 方法。
    ///
    /// # 速率限制
    ///
    /// 默认：每 60 秒 100 个请求（与 API 密钥关联，而非 IP 地址）
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use pixabay_sdk::Pixabay;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Pixabay::new("your_api_key".to_string());
    /// let images = client.search_images("yellow flowers", Some(10), Some(1)).await?;
    /// println!("找到 {} 张图片", images.total_hits);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_images(
        &self,
        query: &str,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<ImageResponse> {
        // 验证 per_page 范围（根据 API 文档为 3-200）
        let per_page = per_page.unwrap_or(20).clamp(3, 200);
        let page = page.unwrap_or(1);

        let mut url = Url::parse(BASE_URL)?;

        url.query_pairs_mut()
            .append_pair("key", &self.api_key)
            .append_pair("q", query)
            .append_pair("per_page", &per_page.to_string())
            .append_pair("page", &page.to_string());

        let response = self.client.get(url).send().await?;

        self.handle_response(response).await
    }

    /// 处理 API 响应并提取相应的错误
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            Ok(response.json().await?)
        } else if status.as_u16() == 429 {
            Err(PixabayError::RateLimitExceeded)
        } else if status.as_u16() == 400 {
            let error_text = response.text().await?;
            Err(PixabayError::ApiError(format!("错误请求: {}", error_text)))
        } else if status.as_u16() == 401 || status.as_u16() == 403 {
            Err(PixabayError::InvalidApiKey)
        } else {
            let error_text = response.text().await?;
            Err(PixabayError::ApiError(format!("HTTP {}: {}", status.as_u16(), error_text)))
        }
    }

    /// 使用高级参数搜索图片
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use pixabay_sdk::{Pixabay, SearchImageParams, ImageType, Orientation, Category, Order};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Pixabay::new("your_api_key".to_string());
    ///
    /// let params = SearchImageParams::new()
    ///     .query("mountains")
    ///     .per_page(50)
    ///     .image_type(ImageType::Photo)
    ///     .orientation(Orientation::Horizontal)
    ///     .category(Category::Nature)
    ///     .min_width(1920)
    ///     .min_height(1080)
    ///     .editors_choice(true)
    ///     .safesearch(true)
    ///     .order(Order::Latest);
    ///
    /// let images = client.search_images_advanced(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_images_advanced(
        &self,
        params: SearchImageParams,
    ) -> Result<ImageResponse> {
        // 验证 per_page 范围
        let per_page = params.per_page.unwrap_or(20).clamp(3, 200);

        let mut url = Url::parse(BASE_URL)?;

        let mut query = url.query_pairs_mut();
        query.append_pair("key", &self.api_key);

        if let Some(q) = &params.query {
            // 验证查询长度（根据 API 文档最多 100 字符）
            if q.len() > 100 {
                drop(query);
                return Err(PixabayError::ApiError(
                    "查询字符串不能超过 100 个字符".to_string()
                ));
            }
            query.append_pair("q", q);
        }

        query.append_pair("per_page", &per_page.to_string());
        query.append_pair("page", &params.page.unwrap_or(1).to_string());

        if let Some(image_type) = &params.image_type {
            query.append_pair("image_type", &image_type.to_string());
        }
        if let Some(orientation) = &params.orientation {
            query.append_pair("orientation", &orientation.to_string());
        }
        if let Some(category) = &params.category {
            query.append_pair("category", &category.to_string());
        }
        if let Some(min_width) = params.min_width {
            query.append_pair("min_width", &min_width.to_string());
        }
        if let Some(min_height) = params.min_height {
            query.append_pair("min_height", &min_height.to_string());
        }
        if let Some(colors) = &params.colors {
            query.append_pair("colors", colors);
        }
        if let Some(editors_choice) = params.editors_choice {
            query.append_pair("editors_choice", &editors_choice.to_string());
        }
        if let Some(safesearch) = params.safesearch {
            query.append_pair("safesearch", &safesearch.to_string());
        }
        if let Some(order) = &params.order {
            query.append_pair("order", &order.to_string());
        }
        if let Some(lang) = &params.lang {
            query.append_pair("lang", &lang.to_string());
        }

        drop(query);

        let response = self.client.get(url).send().await?;
        self.handle_response(response).await
    }

    /// 通过 ID 获取特定图片
    ///
    /// # 参数
    ///
    /// * `id` - Pixabay 图片 ID
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use pixabay_sdk::Pixabay;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Pixabay::new("your_api_key".to_string());
    /// let image = client.get_image(195893).await?;
    /// println!("图片: {}", image.page_url);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_image(&self, id: u64) -> Result<Image> {
        let mut url = Url::parse(BASE_URL)?;

        url.query_pairs_mut()
            .append_pair("key", &self.api_key)
            .append_pair("id", &id.to_string());

        let response = self.client.get(url).send().await?;

        let image_response: ImageResponse = self.handle_response(response).await?;
        image_response.hits.into_iter().next()
            .ok_or_else(|| PixabayError::ApiError(format!("未找到 ID 为 {} 的图片", id)))
    }

    /// 在 Pixabay 上搜索视频
    ///
    /// # 参数
    ///
    /// * `query` - 搜索查询词（会自动进行 URL 编码，最多 100 字符）
    /// * `per_page` - 每页结果数（3-200，默认：20）
    /// * `page` - 页码（默认：1）
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use pixabay_sdk::Pixabay;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Pixabay::new("your_api_key".to_string());
    /// let videos = client.search_videos("ocean waves", Some(10), Some(1)).await?;
    /// println!("找到 {} 个视频", videos.total_hits);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_videos(
        &self,
        query: &str,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> Result<VideoResponse> {
        // 验证 per_page 范围（根据 API 文档为 3-200）
        let per_page = per_page.unwrap_or(20).clamp(3, 200);
        let page = page.unwrap_or(1);

        let mut url = Url::parse(VIDEO_BASE_URL)?;

        url.query_pairs_mut()
            .append_pair("key", &self.api_key)
            .append_pair("q", query)
            .append_pair("per_page", &per_page.to_string())
            .append_pair("page", &page.to_string());

        let response = self.client.get(url).send().await?;
        self.handle_response(response).await
    }

    /// 使用高级参数搜索视频
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use pixabay_sdk::{Pixabay, SearchVideoParams, VideoType, Category, Order};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Pixabay::new("your_api_key".to_string());
    ///
    /// let params = SearchVideoParams::new()
    ///     .query("sunset")
    ///     .per_page(20)
    ///     .video_type(VideoType::Film)
    ///     .category(Category::Nature)
    ///     .min_width(1920)
    ///     .min_height(1080)
    ///     .editors_choice(true)
    ///     .order(Order::Latest);
    ///
    /// let videos = client.search_videos_advanced(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_videos_advanced(
        &self,
        params: SearchVideoParams,
    ) -> Result<VideoResponse> {
        // 验证 per_page 范围
        let per_page = params.per_page.unwrap_or(20).clamp(3, 200);

        let mut url = Url::parse(VIDEO_BASE_URL)?;

        let mut query = url.query_pairs_mut();
        query.append_pair("key", &self.api_key);

        if let Some(q) = &params.query {
            // 验证查询长度（根据 API 文档最多 100 字符）
            if q.len() > 100 {
                drop(query);
                return Err(PixabayError::ApiError(
                    "查询字符串不能超过 100 个字符".to_string()
                ));
            }
            query.append_pair("q", q);
        }

        query.append_pair("per_page", &per_page.to_string());
        query.append_pair("page", &params.page.unwrap_or(1).to_string());

        if let Some(video_type) = &params.video_type {
            query.append_pair("video_type", &video_type.to_string());
        }
        if let Some(category) = &params.category {
            query.append_pair("category", &category.to_string());
        }
        if let Some(min_width) = params.min_width {
            query.append_pair("min_width", &min_width.to_string());
        }
        if let Some(min_height) = params.min_height {
            query.append_pair("min_height", &min_height.to_string());
        }
        if let Some(editors_choice) = params.editors_choice {
            query.append_pair("editors_choice", &editors_choice.to_string());
        }
        if let Some(safesearch) = params.safesearch {
            query.append_pair("safesearch", &safesearch.to_string());
        }
        if let Some(order) = &params.order {
            query.append_pair("order", &order.to_string());
        }
        if let Some(lang) = &params.lang {
            query.append_pair("lang", &lang.to_string());
        }

        drop(query);

        let response = self.client.get(url).send().await?;
        self.handle_response(response).await
    }

    /// 通过 ID 获取特定视频
    ///
    /// # 参数
    ///
    /// * `id` - Pixabay 视频 ID
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use pixabay_sdk::Pixabay;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Pixabay::new("your_api_key".to_string());
    /// let video = client.get_video(31377).await?;
    /// println!("视频时长: {}秒", video.duration);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_video(&self, id: u64) -> Result<Video> {
        let mut url = Url::parse(VIDEO_BASE_URL)?;

        url.query_pairs_mut()
            .append_pair("key", &self.api_key)
            .append_pair("id", &id.to_string());

        let response = self.client.get(url).send().await?;

        let video_response: VideoResponse = self.handle_response(response).await?;
        video_response.hits.into_iter().next()
            .ok_or_else(|| PixabayError::ApiError(format!("未找到 ID 为 {} 的视频", id)))
    }
}

/// 高级图片搜索参数结构体
///
/// 使用构建器模式来配置图片搜索的高级参数。
///
/// # 示例
///
/// ```
/// use pixabay_sdk::{SearchImageParams, ImageType, Orientation, Category, Order};
///
/// let params = SearchImageParams::new()
///     .query("sunset")
///     .per_page(20)
///     .image_type(ImageType::Photo)
///     .orientation(Orientation::Horizontal)
///     .category(Category::Nature)
///     .safesearch(true);
/// ```
#[derive(Debug, Clone, Default)]
pub struct SearchImageParams {
    /// 搜索查询词
    pub query: Option<String>,
    /// 每页结果数（3-200）
    pub per_page: Option<u32>,
    /// 页码
    pub page: Option<u32>,
    /// 图片类型筛选
    pub image_type: Option<ImageType>,
    /// 图片方向筛选
    pub orientation: Option<Orientation>,
    /// 图片分类筛选
    pub category: Option<Category>,
    /// 最小宽度
    pub min_width: Option<u32>,
    /// 最小高度
    pub min_height: Option<u32>,
    /// 颜色筛选（十六进制颜色值，如 "ffffff"）
    pub colors: Option<String>,
    /// 编辑精选结果
    pub editors_choice: Option<bool>,
    /// 安全搜索
    pub safesearch: Option<bool>,
    /// 结果排序方式
    pub order: Option<Order>,
    /// 搜索语言
    pub lang: Option<Language>,
}

impl SearchImageParams {
    /// 创建一个新的 SearchImageParams 实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置搜索查询词
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// 设置每页结果数（3-200）
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// 设置页码
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// 设置图片类型
    pub fn image_type(mut self, image_type: ImageType) -> Self {
        self.image_type = Some(image_type);
        self
    }

    /// 设置图片方向
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    /// 设置图片分类
    pub fn category(mut self, category: Category) -> Self {
        self.category = Some(category);
        self
    }

    /// 设置最小宽度
    pub fn min_width(mut self, min_width: u32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    /// 设置最小高度
    pub fn min_height(mut self, min_height: u32) -> Self {
        self.min_height = Some(min_height);
        self
    }

    /// 设置颜色筛选
    pub fn colors(mut self, colors: impl Into<String>) -> Self {
        self.colors = Some(colors.into());
        self
    }

    /// 设置是否只返回编辑精选结果
    pub fn editors_choice(mut self, editors_choice: bool) -> Self {
        self.editors_choice = Some(editors_choice);
        self
    }

    /// 设置是否启用安全搜索
    pub fn safesearch(mut self, safesearch: bool) -> Self {
        self.safesearch = Some(safesearch);
        self
    }

    /// 设置结果排序方式
    pub fn order(mut self, order: Order) -> Self {
        self.order = Some(order);
        self
    }

    /// 设置搜索语言
    pub fn lang(mut self, lang: Language) -> Self {
        self.lang = Some(lang);
        self
    }
}

/// 高级视频搜索参数结构体
///
/// 使用构建器模式来配置视频搜索的高级参数。
///
/// # 示例
///
/// ```
/// use pixabay_sdk::{SearchVideoParams, VideoType, Category, Order};
///
/// let params = SearchVideoParams::new()
///     .query("ocean")
///     .per_page(20)
///     .video_type(VideoType::Film)
///     .category(Category::Nature)
///     .safesearch(true);
/// ```
#[derive(Debug, Clone, Default)]
pub struct SearchVideoParams {
    /// 搜索查询词
    pub query: Option<String>,
    /// 每页结果数（3-200）
    pub per_page: Option<u32>,
    /// 页码
    pub page: Option<u32>,
    /// 视频类型筛选
    pub video_type: Option<VideoType>,
    /// 视频分类筛选
    pub category: Option<Category>,
    /// 最小宽度
    pub min_width: Option<u32>,
    /// 最小高度
    pub min_height: Option<u32>,
    /// 编辑精选结果
    pub editors_choice: Option<bool>,
    /// 安全搜索
    pub safesearch: Option<bool>,
    /// 结果排序方式
    pub order: Option<Order>,
    /// 搜索语言
    pub lang: Option<Language>,
}

impl SearchVideoParams {
    /// 创建一个新的 SearchVideoParams 实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置搜索查询词
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// 设置每页结果数（3-200）
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// 设置页码
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// 设置视频类型
    pub fn video_type(mut self, video_type: VideoType) -> Self {
        self.video_type = Some(video_type);
        self
    }

    /// 设置视频分类
    pub fn category(mut self, category: Category) -> Self {
        self.category = Some(category);
        self
    }

    /// 设置最小宽度
    pub fn min_width(mut self, min_width: u32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    /// 设置最小高度
    pub fn min_height(mut self, min_height: u32) -> Self {
        self.min_height = Some(min_height);
        self
    }

    /// 设置是否只返回编辑精选结果
    pub fn editors_choice(mut self, editors_choice: bool) -> Self {
        self.editors_choice = Some(editors_choice);
        self
    }

    /// 设置是否启用安全搜索
    pub fn safesearch(mut self, safesearch: bool) -> Self {
        self.safesearch = Some(safesearch);
        self
    }

    /// 设置结果排序方式
    pub fn order(mut self, order: Order) -> Self {
        self.order = Some(order);
        self
    }

    /// 设置搜索语言
    pub fn lang(mut self, lang: Language) -> Self {
        self.lang = Some(lang);
        self
    }
}