# Media Downloader

统一的媒体下载库,支持多个免费媒体源 (Pixabay, Pexels 等)。

## 特性

- ✅ **多源支持**: 同时使用 Pixabay 和 Pexels
- ✅ **智能聚合**: 自动从所有源聚合结果
- ✅ **容错机制**: 某个源失败时自动使用其他源
- ✅ **统一接口**: 所有源使用相同的数据模型
- ✅ **并发下载**: 高效的并发下载管理
- ✅ **质量选择**: 自动选择最佳媒体质量
- ✅ **类型安全**: 完整的 Rust 类型系统
- ✅ **可扩展**: 轻松添加新的媒体源

## 快速开始

### 添加依赖

```toml
[dependencies]
fusion-media-provider = "xxxx"
dotenvy = "0.15"
tokio = { version = "1", features = ["full"] }
```

### 基础使用

```rust
use media_downloader::{
    MediaDownloader, DownloadConfig, SearchParams, MediaType,
    PixabayProvider, ImageQuality,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建下载器
    let downloader = MediaDownloader::new()
        .add_provider(Arc::new(PixabayProvider::new(
            std::env::var("PIXABAY_API_KEY")?
        )));

    // 搜索图片
    let results = downloader.search(
        SearchParams::new("nature", MediaType::Image).limit(10)
    ).await?;

    println!("Found {} images", results.len());

    // 下载前 5 张
    let paths = downloader.download_items(&results[..5]).await;
    
    for (i, result) in paths.iter().enumerate() {
        match result {
            Ok(path) => println!("Downloaded {}: {}", i + 1, path),
            Err(e) => eprintln!("Failed {}: {}", i + 1, e),
        }
    }

    Ok(())
}
```

## 核心概念

### MediaProvider Trait

所有媒体源都实现这个 trait:

```rust
#[async_trait]
pub trait MediaProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn search_images(&self, query: &str, limit: u32, page: u32) -> Result<Vec<MediaItem>>;
    async fn search_videos(&self, query: &str, limit: u32, page: u32) -> Result<Vec<MediaItem>>;
    async fn get_media(&self, id: &str, media_type: MediaType) -> Result<MediaItem>;
}
```

### MediaItem - 统一数据模型

```rust
pub struct MediaItem {
    pub id: String,
    pub media_type: MediaType,      // Image or Video
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub author: String,
    pub provider: String,            // "Pixabay", "Pexels", etc.
    pub urls: MediaUrls,             // 各种质量的 URL
    pub metadata: MediaMetadata,     // 尺寸、统计信息等
}
```

### MediaUrls

```rust
pub struct MediaUrls {
    pub thumbnail: String,           // 缩略图 (总是可用)
    pub medium: Option<String>,      // 中等质量
    pub large: Option<String>,       // 大尺寸
    pub original: Option<String>,    // 原始质量
    pub video_files: Option<Vec<VideoFile>>, // 视频的多个分辨率
}
```

## 配置选项

### DownloadConfig

```rust
let config = DownloadConfig {
    // 图片质量: Thumbnail, Medium, Large, Original
    image_quality: ImageQuality::Large,
    
    // 视频质量: Tiny(360p), Small(540p), Medium(720p), Large(1080p)
    video_quality: VideoQuality::Large,
    
    // 下载目录
    output_dir: "./downloads".to_string(),
    
    // 文件命名方式
    use_original_names: false,
    
    // 最大并发下载数
    max_concurrent: 5,
};

let downloader = MediaDownloader::new().with_config(config);
```

### SearchParams

```rust
let params = SearchParams::new("sunset", MediaType::Image)
    .limit(50)      // 每页结果数
    .page(1);       // 页码
```

## 使用示例

### 示例 1: 多源搜索

```rust
let downloader = MediaDownloader::new()
    .add_provider(Arc::new(PixabayProvider::new(pixabay_key)))
    .add_provider(Arc::new(PexelsProvider::new(pexels_key)));

// 从所有源搜索
let results = downloader.search(
    SearchParams::new("mountains", MediaType::Image).limit(20)
).await?;

// 结果会包含来自两个源的图片
for item in results {
    println!("{}: {} (from {})", item.id, item.title, item.provider);
}
```

### 示例 2: 指定源搜索

```rust
// 只从 Pixabay 搜索
let pixabay_results = downloader
    .search_from_provider("Pixabay", params)
    .await?;

// 只从 Pexels 搜索
let pexels_results = downloader
    .search_from_provider("Pexels", params)
    .await?;
```

### 示例 3: 视频下载

```rust
let params = SearchParams::new("ocean waves", MediaType::Video)
    .limit(5);

let videos = downloader.search(params).await?;

for video in &videos {
    println!("Video: {}", video.title);
    println!("  Duration: {}s", video.metadata.duration.unwrap_or(0));
    
    // 查看可用的质量选项
    if let Some(files) = &video.urls.video_files {
        for file in files {
            println!("  - {}: {}x{} ({} MB)",
                file.quality, file.width, file.height,
                file.size / 1_000_000);
        }
    }
}

// 下载视频
let paths = downloader.download_items(&videos).await;
```

### 示例 4: 自定义质量和目录

```rust
let config = DownloadConfig {
    image_quality: ImageQuality::Original,
    output_dir: "./high_res_images".to_string(),
    use_original_names: true,
    max_concurrent: 10,
};

let downloader = MediaDownloader::new()
    .with_config(config)
    .add_provider(Arc::new(PixabayProvider::new(api_key)));

let results = downloader.search(
    SearchParams::new("wallpaper 4k", MediaType::Image).limit(100)
).await?;

// 批量下载,最多 10 个并发
let paths = downloader.download_items(&results).await;
```

### 示例 5: 单个文件下载

```rust
let item = &results[0];

match downloader.download_item(item).await {
    Ok(path) => println!("Downloaded to: {}", path),
    Err(e) => eprintln!("Download failed: {}", e),
}
```

## 错误处理

```rust
use media_downloader::{MediaError, Result};

match downloader.search(params).await {
    Ok(results) => {
        println!("Success: {} items", results.len());
    }
    Err(MediaError::NoProviders) => {
        eprintln!("No providers configured!");
    }
    Err(MediaError::AllProvidersFailed) => {
        eprintln!("All providers failed to return results");
    }
    Err(MediaError::PixabayError(e)) => {
        eprintln!("Pixabay error: {}", e);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## 扩展新的提供商

### 步骤 1: 实现 MediaProvider

```rust
use async_trait::async_trait;
use media_downloader::{MediaProvider, MediaItem, MediaType, Result};

pub struct MyCustomProvider {
    api_key: String,
}

impl MyCustomProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl MediaProvider for MyCustomProvider {
    fn name(&self) -> &str {
        "MyCustom"
    }

    async fn search_images(&self, query: &str, limit: u32, page: u32) 
        -> Result<Vec<MediaItem>> 
    {
        // 调用你的 API
        // 将响应转换为 MediaItem
        todo!()
    }

    async fn search_videos(&self, query: &str, limit: u32, page: u32) 
        -> Result<Vec<MediaItem>> 
    {
        todo!()
    }

    async fn get_media(&self, id: &str, media_type: MediaType) 
        -> Result<MediaItem> 
    {
        todo!()
    }
}
```

### 步骤 2: 使用新的提供商

```rust
let downloader = MediaDownloader::new()
    .add_provider(Arc::new(MyCustomProvider::new(api_key)));
```

## 最佳实践

### 1. 使用环境变量存储 API Keys

```rust
use dotenvy::dotenv;

dotenv().ok();
let pixabay_key = std::env::var("PIXABAY_API_KEY")?;
let pexels_key = std::env::var("PEXELS_API_KEY").ok();
```

### 2. 合理设置并发数

```rust
// 根据网络状况调整
let config = DownloadConfig {
    max_concurrent: 5,  // 快速网络可以更高
    ..Default::default()
};
```

### 3. 处理下载失败

```rust
let results = downloader.download_items(&items).await;

let successful: Vec<_> = results.iter()
    .filter_map(|r| r.as_ref().ok())
    .collect();

let failed: Vec<_> = results.iter()
    .filter_map(|r| r.as_ref().err())
    .collect();

println!("Downloaded: {}, Failed: {}", successful.len(), failed.len());
```

### 4. 显示进度

```rust
for (i, result) in paths.iter().enumerate() {
    match result {
        Ok(path) => {
            println!("[{}/{}] ✓ {}", i + 1, paths.len(), path);
        }
        Err(e) => {
            eprintln!("[{}/{}] ✗ {}", i + 1, paths.len(), e);
        }
    }
}
```

## 性能优化

### 1. 批量下载

```rust
// ✅ 好 - 批量下载利用并发
let paths = downloader.download_items(&items).await;

// ❌ 不好 - 串行下载
for item in items {
    downloader.download_item(item).await?;
}
```

### 2. 限制搜索结果

```rust
// 只获取需要的数量
let params = SearchParams::new("nature", MediaType::Image)
    .limit(10);  // 而不是 limit(1000)
```

### 3. 缓存搜索结果

```rust
// 如果需要多次使用相同查询,缓存结果
let results = downloader.search(params).await?;
// 存储到变量中重复使用,而不是重复搜索
```

## 常见问题

### Q: 如何只使用 Pixabay?

A: 只添加 Pixabay provider:

```rust
let downloader = MediaDownloader::new()
    .add_provider(Arc::new(PixabayProvider::new(key)));
```

### Q: 如何获取原始质量的图片?

A: 设置 `image_quality`:

```rust
let config = DownloadConfig {
    image_quality: ImageQuality::Original,
    ..Default::default()
};
```

注意: 原始质量可能需要完整 API 访问权限。

### Q: 下载的文件保存在哪里?

A: 默认在 `./downloads`,可以通过配置修改:

```rust
let config = DownloadConfig {
    output_dir: "/path/to/your/dir".to_string(),
    ..Default::default()
};
```

### Q: 如何处理速率限制?

A: 库会返回相应错误,你需要处理:

```rust
match downloader.search(params).await {
    Err(MediaError::PixabayError(e)) if e.to_string().contains("rate limit") => {
        eprintln!("Rate limited, waiting...");
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
    result => result?,
}
```

## 运行示例

```bash
# 设置 API keys
export PIXABAY_API_KEY=your_key
export PEXELS_API_KEY=your_key

# 运行示例
cargo run --example download
```

## 测试

```bash
cargo test
```

## 许可证

MIT OR Apache-2.0

## 相关链接

- Pixabay API: https://pixabay.com/api/docs/
- Pexels API: https://www.pexels.com/api/