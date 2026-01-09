# Fusion Media Provider - 统一媒体下载库

[![Version](https://img.shields.io/badge/version-1.0.1-blue.svg)](https://github.com/tornado-product/FusionMediaProvider)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80.0+-orange.svg)](https://www.rust-lang.org/)

一个支持多个免费图片/视频源的 Rust 媒体下载库，目前支持 **Pixabay** 和 **Pexels** API。

## 📦 项目结构

这个 workspace 包含五个主要组件:

```
.
├── pixabay-sdk/          # Pixabay API 客户端库
├── pixabay-sdk-cli/          # Pixabay 命令行工具
├── pexels-sdk/           # Pexels API 客户端库
├── pexels-sdk-cli/           # Pexels 命令行工具
└── fusion-media-provider/  # 统一媒体下载器 (抽象层) ⭐
```

### 组件说明

#### 1. **pixabay-sdk** - Pixabay API 客户端

完整的 Pixabay API Rust 封装，支持:
- ✅ 图片搜索 (简单/高级)
- ✅ 视频搜索 (简单/高级)
- ✅ 按 ID 获取媒体
- ✅ 所有 API 参数 (类型、方向、分类等)
- ✅ 类型安全的枚举
- ✅ 完整的错误处理
- ✅ 异步操作

**文档**: [pixabay-sdk/README.md](pixabay-sdk/README.md)

#### 2. **pixabay-sdk-cli** - Pixabay 命令行工具

方便的 CLI 工具用于 Pixabay API:
- 搜索图片和视频
- 获取特定媒体
- JSON 输出格式
- 支持所有搜索参数

**使用**: `cargo run --bin pixabay-sdk-cli -- search-images --query "nature"`

#### 3. **pexels-sdk** - Pexels API 客户端

完整的 Pexels API Rust 封装，支持:
- ✅ 图片搜索
- ✅ 视频搜索
- ✅ 集合搜索
- ✅ 精选集合
- ✅ 按 ID 获取媒体
- ✅ 异步操作

**文档**: [pexels-sdk/README.md](pexels-sdk/README.md)

#### 4. **pexels-sdk-cli** - Pexels 命令行工具

CLI 工具用于 Pexels API:
- 搜索图片和视频
- 搜索集合
- 获取特定媒体

**使用**: `cargo run --bin pexels-sdk-cli -- search-photos --query "nature"`

#### 5. **fusion-media-provider** - 统一抽象层 ⭐

**这是本项目的核心功能** - 提供统一接口来:
- ✅ 同时使用多个媒体源 (Pixabay, Pexels)
- ✅ 智能降级和容错
- ✅ 统一的数据模型
- ✅ 并发下载管理
- ✅ 自动质量选择
- ✅ 下载进度回调
- ✅ 批量下载进度追踪
- ✅ 完整的分页支持
- ✅ 可扩展架构

## 🚀 快速开始

### 安装

将以下内容添加到 `Cargo.toml`:

```toml
[dependencies]
# 使用统一下载器 (推荐)
fusion-media-provider = { path = "path/to/fusion-media-provider", features = ["pexels"] }

# 或单独使用各个 SDK
pixabay-sdk = { path = "path/to/pixabay-sdk" }
pexels-sdk = { path = "path/to/pexels-sdk" }

dotenvy = "0.15"
tokio = { version = "1", features = ["full"] }
```

**功能特性**:
- `default`: 包含 `pixabay` 和 `pexels` 特性
- `pixabay`: 启用 Pixabay 支持
- `pexels`: 启用 Pexels 支持（需要 `pexels-sdk`）

### 环境配置

创建 `.env` 文件:

```env
# Pixabay API Key (必需)
PIXABAY_API_KEY=your_pixabay_key

# Pexels API Key (可选,用于多源支持)
PEXELS_API_KEY=your_pexels_key
```

**获取 API Keys:**
- Pixabay: https://pixabay.com/api/docs/
- Pexels: https://www.pexels.com/api/

### 基础使用

#### 选项 1: 使用统一下载器 (推荐)

```rust
use dotenvy::dotenv;
use fusion_media_provider::{
    MediaDownloader, DownloadConfig, SearchParams, MediaType,
    PixabayProvider, ImageQuality,
};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // 创建配置
    let config = DownloadConfig {
        image_quality: ImageQuality::Large,
        output_dir: "./downloads".to_string(),
        max_concurrent: 5,
        ..Default::default()
    };

    // 添加多个提供商
    let mut downloader = MediaDownloader::new()
        .with_config(config)
        .add_provider(Arc::new(
            PixabayProvider::new(env::var("PIXABAY_API_KEY")?)
        ));
    
    // 可以添加更多提供商
    #[cfg(feature = "pexels")]
    if let Ok(pexels_key) = env::var("PEXELS_API_KEY") {
        downloader = downloader.add_provider(Arc::new(
            fusion_media_provider::PexelsProvider::new(pexels_key)
        ));
    }

    // 或者使用便捷方法根据名称添加
    // downloader = downloader.add_provider_by_name_and_apikey("Pexels", &pexels_key);

    // 从所有源搜索
    let results = downloader.search(
        SearchParams::new("nature", MediaType::Image).limit(10)
    ).await?;

    println!("Found {} images from all providers", results.items.len());
    println!("Total available: {}", results.total);
    println!("Total pages: {}", results.total_pages);

    // 批量下载
    let paths = downloader.download_items(&results.items[..5.min(results.items.len())]).await;

    for (item, result) in results.items.iter().zip(paths.iter()) {
        match result {
            Ok(path) => println!("✓ Downloaded: {} -> {}", item.title, path),
            Err(e) => eprintln!("✗ Failed: {} - {}", item.title, e),
        }
    }

    Ok(())
}
```

#### 选项 2: 直接使用 Pixabay SDK

```rust
use pixabay_sdk::{Pixabay, SearchImageParams, ImageType, Category};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Pixabay::new(std::env::var("PIXABAY_API_KEY")?);

    // 简单搜索
    let images = client.search_images("cats", Some(10), Some(1)).await?;
    println!("Found {} images", images.total_hits);

    // 高级搜索
    let params = SearchImageParams::new()
        .query("mountains")
        .image_type(ImageType::Photo)
        .category(Category::Nature)
        .min_width(1920);

    let results = client.search_images_advanced(params).await?;
    println!("Found {} images", results.total_hits);

    Ok(())
}
```

#### 选项 3: 直接使用 Pexels SDK

```rust
use pexels_sdk::{Pexels, SearchBuilder, VideoSearchBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Pexels::new(std::env::var("PEXELS_API_KEY")?);

    // 搜索图片（使用构建器模式）
    let photos = client.search_photos(
        SearchBuilder::new()
            .query("nature")
            .per_page(10)
            .page(1)
    ).await?;
    println!("Found {} photos", photos.total_results);

    // 搜索视频（使用构建器模式）
    let videos = client.search_videos(
        VideoSearchBuilder::new()
            .query("ocean")
            .per_page(10)
            .page(1)
    ).await?;
    println!("Found {} videos", videos.total_results);

    // 获取特定图片
    let photo = client.get_photo(10967).await?;
    println!("Photo URL: {}", photo.src.large);

    Ok(())
}
```

## 🎯 核心功能

### 1. 多源支持

同时从多个源获取媒体,自动聚合结果:

```rust
let downloader = MediaDownloader::new()
.add_provider(Arc::new(PixabayProvider::new(pixabay_key)))
.add_provider(Arc::new(PexelsProvider::new(pexels_key)));

// 从所有源搜索
let results = downloader.search(params).await?;

// 查看聚合的分页信息
println!("Total: {}", results.total);
println!("Total pages: {}", results.total_pages);  // 所有 provider 的总页数之和
println!("Items: {}", results.items.len());

// 或指定特定源
let pixabay_only = downloader
.search_from_provider("Pixabay", params)
.await?;
```

### 2. 智能质量选择

根据偏好自动选择最佳质量:

```rust
let config = DownloadConfig {
    image_quality: ImageQuality::Large,    // Thumbnail, Medium, Large, Original
    video_quality: VideoQuality::Large,     // Tiny, Small, Medium, Large
    ..Default::default()
};
```

### 3. 并发下载

高效的并发下载管理:

```rust
let config = DownloadConfig {
    max_concurrent: 10,  // 最多同时下载 10 个文件
    ..Default::default()
};

// 自动管理并发
let results = downloader.download_items(&media_items).await;

// 或使用批量下载方法
let results = downloader.download_batch(
    &media_items,
    |progress| {
        // 批量进度回调
    }
).await;
```

**下载方法**:
- `download_item(item)` - 下载单个媒体项
- `download_items(items)` - 批量下载（返回 Vec<Result<String>>）
- `download_items_with_batch_progress(items, callback)` - 带批量进度追踪
- `download_by_id(id, media_type)` - 通过 ID 下载
- `download_batch(items, callback)` - 批量下载（带回调）

### 4. 统一数据模型

所有提供商返回统一的结构，包含完整的分页信息:

```rust
// 聚合搜索结果
pub struct AggregatedSearchResult {
    pub total: u32,              // 所有 provider 的总结果数
    pub total_hits: u32,         // 实际返回的结果数
    pub total_pages: u32,        // 所有 provider 的总页数之和
    pub page: u32,               // 当前页
    pub per_page: u32,           // 每页数量
    pub items: Vec<MediaItem>,   // 所有媒体项
    pub provider_results: Vec<SearchResult>, // 各 provider 详情
}

// 单个媒体项
pub struct MediaItem {
    pub id: String,
    pub media_type: MediaType,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub author: String,
    pub author_url: String,
    pub source_url: String,
    pub provider: String,        // "Pixabay" 或 "Pexels"
    pub urls: MediaUrls,         // 各种质量的 URL
    pub metadata: MediaMetadata, // 尺寸、时长、统计信息
}
```

### 5. 下载进度追踪

支持单个和批量下载的进度回调:

```rust
use fusion_media_provider::{DownloadProgress, DownloadState, BatchDownloadProgress};

// 单个下载进度
let config = DownloadConfig {
    progress_callback: Some(Arc::new(|progress: DownloadProgress| {
        match progress.state {
            DownloadState::Downloading => {
                println!("下载中: {:.1}% - {}",
                    progress.percentage,
                    progress.format_speed()
                );
            }
            DownloadState::Completed => {
                println!("完成: {} ({:.2}s)",
                    progress.item_title,
                    progress.elapsed_secs
                );
            }
            _ => {}
        }
    })),
    ..Default::default()
};

// 批量下载进度
let results = downloader.download_items_with_batch_progress(
    &items,
    |batch: BatchDownloadProgress| {
        println!("批量进度: {:.1}% ({}/{})",
            batch.overall_percentage,
            batch.completed_items,
            batch.total_items
        );
    }
).await;
```

### 6. 完整分页支持

支持多源聚合分页，详见 [分页功能说明](fusion-media-provider/PAGINATION.md):

```rust
let results = downloader.search(
    SearchParams::new("nature", MediaType::Image)
        .limit(20)
        .page(1)
).await?;

println!("总结果数: {}", results.total);
println!("总页数: {}", results.total_pages);
println!("当前页: {}/{}", results.page, results.total_pages);

// 查看各 provider 的分页信息
for provider_result in &results.provider_results {
    println!("{}: {} 结果, {} 页",
        provider_result.provider,
        provider_result.total,
        provider_result.total_pages
    );
}
```

## 📚 详细文档

### API 文档

- **Pixabay SDK**: [pixabay-sdk/README.md](pixabay-sdk/README.md)
  - 完整的 API 参考
  - 搜索参数说明
  - 错误处理指南

- **Pexels SDK**: [pexels-sdk/README.md](pexels-sdk/README.md)
  - API 方法说明
  - 使用示例

- **Fusion Media Provider**: [fusion-media-provider/README.md](fusion-media-provider/README.md)
  - 统一接口说明
  - 配置选项
  - 扩展指南
  - [分页功能说明](fusion-media-provider/PAGINATION.md)

### 示例代码

- **统一下载器示例**: [fusion-media-provider/examples/download_example.rs](fusion-media-provider/examples/download_example.rs)
- **Pixabay 基础示例**: [pixabay-sdk/examples/base.rs](pixabay-sdk/examples/base.rs)

### 使用限制

#### Pixabay
- ✅ 速率限制: 100 请求/60秒
- ✅ 必须缓存 24 小时
- ✅ 显示来源标注
- ✅ 下载到自己服务器 (不要热链接)

#### Pexels
- ✅ 速率限制: 200 请求/小时
- ✅ 必须显示摄影师署名
- ✅ 查看完整 API 文档: https://www.pexels.com/api/documentation/

## 🛠️ 构建和运行

### 构建整个 workspace

```bash
# 克隆仓库
git clone https://github.com/tornado-product/FusionMediaProvider.git
cd FusionMediaProvider

# 构建所有组件
cargo build --release

# 运行测试
cargo test
```

### 运行示例

```bash
# 配置 API keys
# 创建 .env 文件并添加:
# PIXABAY_API_KEY=your_key
# PEXELS_API_KEY=your_key (可选)

# 运行统一下载器示例
cd fusion-media-provider
cargo run --example download_example

# 运行 Pixabay 基础示例
cd pixabay-sdk
cargo run --example base
```

### 使用 CLI 工具

#### Pixabay CLI

```bash
cd pixabay-sdk-cli

# 搜索图片
cargo run --bin pixabay-sdk-cli -- search-images --query "nature" --per-page 10

# 搜索视频
cargo run --bin pixabay-sdk-cli -- search-videos --query "ocean" --per-page 5

# 获取特定图片
cargo run --bin pixabay-sdk-cli -- get-image --id 736885

# 安装到系统
cargo install --path . --bin pixabay-sdk-cli
pixabay-sdk-cli search-images --query "sunset"
```

#### Pexels CLI

```bash
cd pexels-sdk-cli

# 搜索图片
cargo run --bin pexels-sdk-cli -- search-photos --query "nature" --per-page 10

# 搜索视频
cargo run --bin pexels-sdk-cli -- search-videos --query "ocean" --per-page 5

# 安装到系统
cargo install --path . --bin pexels-sdk-cli
pexels-sdk-cli search-photos --query "mountains"
```

## 🏗️ 架构设计

### 抽象层架构

```
┌─────────────────────────────────────────┐
│        MediaDownloader (统一接口)        │
├─────────────────────────────────────────┤
│  - 多源聚合                              │
│  - 并发下载                              │
│  - 质量选择                              │
│  - 错误处理                              │
│  - 进度追踪                              │
│  - 分页管理                              │
└─────────────┬───────────────────────────┘
              │
      ┌───────┴───────┐
      ▼               ▼
┌──────────┐    ┌──────────┐
│ Pixabay  │    │  Pexels  │
│ Provider │    │ Provider │
└──────────┘    └──────────┘
      │               │
      ▼               ▼
┌──────────┐    ┌──────────┐
│pixabay-  │    │ pexels-   │
│  sdk     │    │   sdk     │
└──────────┘    └──────────┘
      │               │
      ▼               ▼
 Pixabay API    Pexels API
```

### 核心方法

**MediaDownloader** 提供的主要方法:

```rust
// 搜索相关
pub async fn search(&self, params: SearchParams) -> Result<AggregatedSearchResult>
pub async fn search_from_provider(&self, provider_name: &str, params: SearchParams) -> Result<SearchResult>

// 下载相关
pub async fn download_item(&self, item: &MediaItem) -> Result<String>
pub async fn download_items(&self, items: &[MediaItem]) -> Vec<Result<String>>
pub async fn download_items_with_batch_progress<F>(&self, items: &[MediaItem], callback: F) -> Vec<Result<String>>
pub async fn download_by_id(&self, id: &str, media_type: MediaType) -> Result<String>
pub async fn download_batch<F>(&self, items: &[MediaItem], callback: F) -> Vec<Result<String>>

// 配置相关
pub fn with_config(self, config: DownloadConfig) -> Self
pub fn add_provider(self, provider: Arc<dyn MediaProvider>) -> Self
pub fn add_provider_by_name_and_apikey(self, provider_name: &str, api_key: &str) -> Self
pub fn providers(&self) -> &[Arc<dyn MediaProvider>]
```

### 扩展新的提供商

实现 `MediaProvider` trait:

```rust
use async_trait::async_trait;
use fusion_media_provider::{MediaProvider, MediaItem, MediaType, SearchResult, Result};
use std::sync::Arc;

pub struct MyProvider {
    api_key: String,
    // ... 你的客户端
}

impl MyProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl MediaProvider for MyProvider {
    fn name(&self) -> &str {
        "MyProvider"
    }

    async fn search_images(&self, query: &str, limit: u32, page: u32) 
        -> Result<SearchResult> {
        // 实现搜索逻辑
        // 调用你的 API，转换为 SearchResult
        // SearchResult 包含: total, total_hits, page, per_page, total_pages, items, provider
        todo!()
    }

    async fn search_videos(&self, query: &str, limit: u32, page: u32) 
        -> Result<SearchResult> {
        // 类似实现
        todo!()
    }

    async fn get_media(&self, id: &str, media_type: MediaType) 
        -> Result<MediaItem> {
        // 通过 ID 获取单个媒体项
        todo!()
    }
}

// 使用
let downloader = MediaDownloader::new()
    .add_provider(Arc::new(MyProvider::new(api_key)));
```

**重要提示**:
- `search_images` 和 `search_videos` 需要返回 `SearchResult`，而不是 `Vec<MediaItem>`
- `SearchResult` 必须包含完整的分页信息（total, total_pages 等）
- 确保正确设置 `provider` 字段为你的提供商名称

详细扩展指南请参考 [fusion-media-provider/README.md](fusion-media-provider/README.md)

## 📊 功能对比

| 功能 | pixabay-sdk | pexels-sdk | fusion-media-provider |
|------|------------|------------|------------------|
| Pixabay 支持 | ✅ 完整 | ❌ | ✅ 通过 Provider |
| Pexels 支持 | ❌ | ✅ 完整 | ✅ 通过 Provider |
| 多源聚合 | ❌ | ❌ | ✅ |
| 并发下载 | ❌ | ❌ | ✅ |
| 质量选择 | 手动 | 手动 | ✅ 自动 |
| 统一接口 | ❌ | ❌ | ✅ |
| 错误容错 | 基础 | 基础 | ✅ 高级 |
| 进度追踪 | ❌ | ❌ | ✅ |
| 分页支持 | 基础 | 基础 | ✅ 完整（含聚合） |
| 直接 API 访问 | ✅ | ✅ | ❌ (通过 Provider) |

## 🔧 配置选项

### DownloadConfig

```rust
pub struct DownloadConfig {
    /// 图片质量偏好: Thumbnail, Medium, Large, Original
    pub image_quality: ImageQuality,
    
    /// 视频质量偏好: Tiny(360p), Small(540p), Medium(720p), Large(1080p)
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
```

### SearchParams

```rust
pub struct SearchParams {
    pub query: String,
    pub limit: u32,      // 每页结果数
    pub page: u32,       // 页码
    pub media_type: MediaType,
}

// 使用 builder 模式
let params = SearchParams::new("nature", MediaType::Image)
    .limit(50)      // 或使用 .per_page(50)
    .page(2);
```

**注意**: `limit()` 和 `per_page()` 是等价的，都用于设置每页结果数。

## 🐛 故障排查

### 常见错误

#### "PIXABAY_API_KEY not set" / "PEXELS_API_KEY not set"
确保 `.env` 文件存在且包含正确的 API key：
```bash
# 检查环境变量
echo $PIXABAY_API_KEY
echo $PEXELS_API_KEY
```

#### "All providers failed"
检查:
1. API keys 是否有效
2. 网络连接是否正常
3. 是否超过速率限制
4. API 服务是否可用

#### "No suitable video quality found"
某些视频可能没有所有质量选项，库会自动降级到可用的最高质量。这是正常行为。

#### "Provider not enabled"
如果使用 Pexels，确保在 `Cargo.toml` 中启用了 `pexels` 特性：
```toml
fusion-media-provider = { path = "...", features = ["pexels"] }
```

#### "Unknown provider"
使用 `add_provider_by_name_and_apikey` 时，确保 provider 名称正确：
- `"Pixabay"` 或 `"pixabay"`
- `"Pexels"` 或 `"pexels"`（需要启用 `pexels` 特性）

#### "No providers configured"
在调用 `search()` 之前，确保至少添加了一个 provider：
```rust
let downloader = MediaDownloader::new()
    .add_provider(Arc::new(PixabayProvider::new(api_key)));
```

#### "Rate limit exceeded"
已超过 API 速率限制：
- **Pixabay**: 100 请求/60秒
- **Pexels**: 200 请求/小时

解决方案：
```rust
// 添加重试逻辑
use tokio::time::{sleep, Duration};

loop {
    match downloader.search(params.clone()).await {
        Ok(results) => break,
        Err(MediaError::PixabayError(e)) if e.to_string().contains("rate limit") => {
            println!("速率限制，等待 60 秒...");
            sleep(Duration::from_secs(60)).await;
        }
        Err(e) => return Err(e.into()),
    }
}
```

### 错误类型

`fusion-media-provider` 使用统一的错误类型 `MediaError`：

```rust
use fusion_media_provider::MediaError;

match downloader.search(params).await {
    Ok(results) => println!("成功"),
    Err(MediaError::NoProviders) => eprintln!("未配置 provider"),
    Err(MediaError::AllProvidersFailed) => eprintln!("所有 provider 都失败"),
    Err(MediaError::PixabayError(e)) => eprintln!("Pixabay 错误: {}", e),
    Err(MediaError::PexelsError(e)) => eprintln!("Pexels 错误: {}", e),
    Err(MediaError::DownloadError(msg)) => eprintln!("下载错误: {}", msg),
    Err(e) => eprintln!("其他错误: {}", e),
}
```

### 编译错误
```bash
# 更新依赖
cargo update

# 清理并重新构建
cargo clean
cargo build
```

## 📝 示例场景

### 场景 1: 批量下载高质量壁纸

```rust
use fusion_media_provider::{
    MediaDownloader, DownloadConfig, SearchParams, MediaType,
    PixabayProvider, ImageQuality,
};
use std::sync::Arc;

let config = DownloadConfig {
    image_quality: ImageQuality::Original,
    output_dir: "./wallpapers".to_string(),
    max_concurrent: 10,
    ..Default::default()
};

let downloader = MediaDownloader::new()
    .with_config(config)
    .add_provider(Arc::new(PixabayProvider::new(api_key)));

let params = SearchParams::new("4k wallpaper", MediaType::Image)
    .limit(100);

let results = downloader.search(params).await?;
let paths = downloader.download_items(&results.items).await;

// 统计结果
let successful = paths.iter().filter(|r| r.is_ok()).count();
println!("成功下载: {}/{}", successful, paths.len());
```

### 场景 2: 视频素材收集

```rust
use fusion_media_provider::{VideoQuality, DownloadConfig};

let config = DownloadConfig {
    video_quality: VideoQuality::Large,  // 1080p
    output_dir: "./videos".to_string(),
    max_concurrent: 3,  // 视频文件较大，降低并发
    ..Default::default()
};

let downloader = MediaDownloader::new()
    .with_config(config)
    .add_provider(Arc::new(PixabayProvider::new(api_key)));

let params = SearchParams::new("nature timelapse", MediaType::Video)
    .limit(20);

let videos = downloader.search(params).await?;

// 查看视频信息
for video in &videos.items {
    println!("{} - {}s", video.title, video.metadata.duration.unwrap_or(0));
    if let Some(files) = &video.urls.video_files {
        for file in files {
            println!("  {}: {}x{}", file.quality, file.width, file.height);
        }
    }
}

// 下载视频
let paths = downloader.download_items(&videos.items).await;
```

### 场景 3: 多源对比

```rust
use fusion_media_provider::PexelsProvider;

let downloader = MediaDownloader::new()
    .add_provider(Arc::new(PixabayProvider::new(pixabay_key)))
    .add_provider(Arc::new(PexelsProvider::new(pexels_key)));

let params = SearchParams::new("mountains", MediaType::Image).limit(20);

// 从 Pixabay 搜索
let pixabay_result = downloader
    .search_from_provider("Pixabay", params.clone())
    .await?;

// 从 Pexels 搜索
let pexels_result = downloader
    .search_from_provider("Pexels", params.clone())
    .await?;

// 比较结果
println!("Pixabay: {} results, {} pages",
         pixabay_result.total, pixabay_result.total_pages);
println!("Pexels: {} results, {} pages",
         pexels_result.total, pexels_result.total_pages);

// 或者一次性获取所有源的聚合结果
let all_results = downloader.search(params).await?;
println!("Combined: {} total results, {} total pages",
         all_results.total, all_results.total_pages);

// 查看各 provider 的贡献
for provider_result in &all_results.provider_results {
    println!("{}: {} items ({} pages)",
        provider_result.provider,
        provider_result.items.len(),
        provider_result.total_pages
    );
}
```

### 场景 4: 带进度追踪的批量下载

```rust
use fusion_media_provider::{DownloadProgress, DownloadState, BatchDownloadProgress};
use std::sync::Arc;

let config = DownloadConfig {
    progress_callback: Some(Arc::new(|progress: DownloadProgress| {
        match progress.state {
            DownloadState::Downloading => {
                print!("\r⬇️  {} | {:.1}% | {} | {}     ",
                    progress.item_title,
                    progress.percentage,
                    progress.format_speed(),
                    progress.format_eta()
                );
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
            DownloadState::Completed => {
                println!("\r✅ {} ({:.2}s)                    ",
                    progress.item_title,
                    progress.elapsed_secs
                );
            }
            _ => {}
        }
    })),
    ..Default::default()
};

let downloader = MediaDownloader::new()
    .with_config(config)
    .add_provider(Arc::new(PixabayProvider::new(api_key)));

let results = downloader.search(
    SearchParams::new("nature", MediaType::Image).limit(10)
).await?;

// 使用批量进度追踪
let paths = downloader.download_items_with_batch_progress(
    &results.items,
    |batch: BatchDownloadProgress| {
        println!("\n📊 批量进度: {:.1}% ({}/{})",
            batch.overall_percentage,
            batch.completed_items,
            batch.total_items
        );
    }
).await;
```

### 场景 5: 通过 ID 下载媒体

```rust
// 通过 ID 直接下载（无需先搜索）
match downloader.download_by_id("12345", MediaType::Image).await {
    Ok(path) => println!("下载成功: {}", path),
    Err(e) => eprintln!("下载失败: {}", e),
}

// 或先获取媒体信息再下载
let item = downloader
    .search_from_provider("Pixabay", 
        SearchParams::new("nature", MediaType::Image).limit(1)
    )
    .await?;
    
if let Some(first_item) = item.items.first() {
    let path = downloader.download_item(first_item).await?;
    println!("下载到: {}", path);
}
```

### 场景 6: 使用便捷方法添加 Provider

```rust
// 方式 1: 直接创建 Provider（推荐，类型安全）
let downloader = MediaDownloader::new()
    .add_provider(Arc::new(PixabayProvider::new(pixabay_key)))
    .add_provider(Arc::new(PexelsProvider::new(pexels_key)));

// 方式 2: 使用便捷方法（适合动态配置）
let downloader = MediaDownloader::new()
    .add_provider_by_name_and_apikey("Pixabay", &pixabay_key)
    .add_provider_by_name_and_apikey("Pexels", &pexels_key);

// 查看已添加的 Provider
for provider in downloader.providers() {
    println!("已添加: {}", provider.name());
}
```

## 🤝 贡献

欢迎贡献! 你可以:
- 添加新的媒体源 (Unsplash, Flickr 等)
- 改进错误处理
- 添加更多测试
- 改进文档

## 📄 许可证

本项目采用双许可证:
- MIT License
- Apache License 2.0

任选其一使用。

## 🙏 致谢

- **Pixabay** - 提供免费 API
- **Pexels** - 提供高质量媒体
- **Rust 社区** - 优秀的工具和库

## 🔗 相关链接

### API 文档
- Pixabay API: https://pixabay.com/api/docs/
- Pexels API: https://www.pexels.com/api/documentation/

### 项目链接
- GitHub: https://github.com/tornado-product/FusionMediaProvider
- 文档: https://docs.rs/fusion-media-provider
- 问题反馈: https://github.com/tornado-product/FusionMediaProvider/issues

### 学习资源
- Rust async book: https://rust-lang.github.io/async-book/
- Tokio 文档: https://tokio.rs/

## 📋 版本信息

- **当前版本**: 1.0.1
- **Rust 版本要求**: 1.80.0+
- **许可证**: MIT OR Apache-2.0

## 🚀 CI/CD 和发布

本项目配置了完整的 CI/CD 流程：

### 自动化流程

- ✅ **持续集成**: 每次推送代码时自动运行测试和 lint 检查
- ✅ **自动发布**: 推送版本 tag 时自动创建 GitHub Release
- ✅ **自动发布到 crates.io**: 自动发布所有包到 crates.io
- ✅ **文档构建**: 自动构建并部署文档到 GitHub Pages

### 发布新版本

1. **更新版本号**（在 `Cargo.toml` 中）
2. **更新 CHANGELOG.md**
3. **创建并推送 tag**:
   ```bash
   git tag -a v1.0.2 -m "Release v1.0.2"
   git push origin v1.0.2
   ```

推送 tag 后，GitHub Actions 会自动：
- 创建 GitHub Release
- 发布所有包到 crates.io

详细说明请参考 [发布指南](.github/PUBLISHING.md)

### 状态徽章

可以在 README 中添加状态徽章：

```markdown
![CI](https://github.com/tornado-product/FusionMediaProvider/workflows/CI/badge.svg)
![Release](https://github.com/tornado-product/FusionMediaProvider/workflows/Release/badge.svg)
```

## ⚠️ 重要提示

1. **API 密钥安全**: 永远不要将 API 密钥提交到版本控制系统。使用 `.env` 文件并确保它已添加到 `.gitignore`。

2. **速率限制**: 
   - Pixabay: 100 请求/60秒
   - Pexels: 200 请求/小时
   - 请合理控制请求频率，避免触发限制

3. **许可证合规**: 使用本库下载的媒体内容仍受原提供商的许可证约束。请确保：
   - 遵守各个平台的使用条款
   - 正确标注来源和作者
   - 不要热链接，应下载到自己的服务器

4. **缓存要求**: Pixabay 要求搜索结果必须缓存 24 小时，请确保实现适当的缓存机制。

---

**注意**: 使用本库下载的媒体内容仍受原提供商的许可证约束。请确保遵守各个平台的使用条款。