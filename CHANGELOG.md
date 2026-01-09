# Changelog

## [1.0.2] - 2025-12-15

### 新增
- 添加了 `fusion-media-provider` 包，统一 Pexels 和 Pixabay 接口
- 支持媒体文件下载功能
- 添加了分页辅助函数

### 变更
- 更新了 `pexels-sdk` 依赖到最新版本
- 改进了错误处理，提供更详细的错误信息
- 优化了 API 请求性能

### 修复
- 修复了分页计算中的整数溢出问题
- 修复了 API 密钥验证逻辑
- 修复了某些情况下空结果的处理

### 文档
- 更新了 README.md，添加了新的使用示例
- 完善了 API 文档注释
- 添加了故障排查指南

## v0.1.0 - 基于官方 Pixabay API 文档实现

### 参考文档
- 官方 API 文档: https://pixabay.com/api/docs/
- 实现日期: 2024-11

### 已实现功能

#### ✅ 图片搜索 API
完全按照官方文档实现,支持所有参数:

**基础参数:**
- ✅ `q` - 搜索关键词 (最多 100 字符)
- ✅ `lang` - 26 种语言支持
- ✅ `id` - 按 ID 检索图片
- ✅ `page` / `per_page` - 分页 (3-200 结果/页)

**过滤参数:**
- ✅ `image_type` - all, photo, illustration, vector
- ✅ `orientation` - all, horizontal, vertical
- ✅ `category` - 20+ 个分类
- ✅ `min_width` / `min_height` - 最小尺寸
- ✅ `colors` - 颜色过滤
- ✅ `editors_choice` - 编辑精选
- ✅ `safesearch` - 安全搜索
- ✅ `order` - popular, latest

#### ✅ 视频搜索 API
完全按照官方文档实现:
- ✅ 基础搜索参数 (q, lang, id, 分页)
- ✅ `video_type` - all, film, animation
- ✅ 其他过滤参数 (category, 尺寸, 编辑精选等)
- ✅ 多分辨率视频文件 (large, medium, small, tiny)

#### ✅ 响应结构
完全匹配官方 API 响应格式:

**图片响应字段:**
```rust
- id, pageURL, type, tags
- previewURL, previewWidth, previewHeight
- webformatURL, webformatWidth, webformatHeight
- largeImageURL
- fullHDURL, imageURL, vectorURL (需完整 API 访问)
- imageWidth, imageHeight, imageSize
- views, downloads, likes, comments, collections
- user_id, user, userImageURL
```

**视频响应字段:**
```rust
- id, pageURL, type, tags, duration
- picture_id
- videos (large, medium, small, tiny)
  - url, width, height, size
- views, downloads, likes, comments
- user_id, user, userImageURL
```

### 与官方文档的对应关系

| 官方文档 | 本库实现 | 状态 |
|---------|---------|------|
| 图片搜索端点 | `client.search_images()` | ✅ 完整实现 |
| 图片高级搜索 | `client.search_images_advanced()` | ✅ 支持所有参数 |
| 通过 ID 获取图片 | `client.get_image()` | ✅ 完整实现 |
| 视频搜索端点 | `client.search_videos()` | ✅ 完整实现 |
| 视频高级搜索 | `client.search_videos_advanced()` | ✅ 支持所有参数 |
| 通过 ID 获取视频 | `client.get_video()` | ✅ 完整实现 |
| 完整 API 访问字段 | Optional 字段支持 | ✅ 支持但需权限 |

### 错误处理
按照官方文档实现:
- ✅ HTTP 429 - 速率限制 (RateLimitExceeded)
- ✅ HTTP 400 - 参数错误 (ApiError)
- ✅ HTTP 401/403 - API key 无效 (InvalidApiKey)
- ✅ 网络错误 (RequestError)
- ✅ JSON 解析错误 (JsonError)

### API 限制遵守
严格遵守官方要求:
- ✅ 速率限制: 100 请求/60秒
- ⚠️ 缓存要求: 需要用户自行实现 24 小时缓存
- ⚠️ 来源标注: 需要用户在显示结果时标注 Pixabay
- ⚠️ 禁止热链接: 文档中已说明需下载图片到自己服务器
- ✅ 真实请求: 库设计用于正常 API 调用

### 数据类型
完全类型安全的枚举:
```rust
- ImageType: All, Photo, Illustration, Vector
- VideoType: All, Film, Animation
- Orientation: All, Horizontal, Vertical
- Order: Popular, Latest
- Category: 20 个分类枚举
- Language: 26 种语言枚举
```

### Builder 模式
提供友好的参数构建器:
```rust
SearchImageParams::new()
    .query("nature")
    .image_type(ImageType::Photo)
    .orientation(Orientation::Horizontal)
    .category(Category::Nature)
    .min_width(1920)
    .editors_choice(true)
    .safesearch(true)
```

### CLI 工具
命令行工具支持所有 API 功能:
```bash
pixabay search-images --query "nature" --image-type photo
pixabay search-videos --query "ocean" --video-type film
pixabay get-image --id 195893
pixabay get-video --id 3998
```

### 依赖版本
基于 Rust 2021 Edition:
- `reqwest` 0.12 (HTTP 客户端)
- `serde` 1.0 (JSON 序列化)
- `tokio` 1.0 (异步运行时)
- `thiserror` 2.0 (错误处理)
- `clap` 4.5 (CLI 解析)

### 文档参考
所有实现细节参考自:
- Pixabay API 官方文档: https://pixabay.com/api/docs/
- 其他语言的社区实现 (Ruby, PHP, JavaScript)
- Pexels API 库的设计模式: https://github.com/houseme/pexels

### 已知限制
- 完整 API 访问字段: `fullHDURL`, `imageURL`, `vectorURL` 需要申请完整 API 访问权限
- 缓存: 库本身不实现缓存,需要用户层面实现 24 小时缓存要求
- 热链接: 库返回 Pixabay CDN URL,但根据官方要求不应永久使用这些 URL
- 分页限制: `totalHits` 最大为 500,即使实际结果更多

### 测试状态
- ✅ 单元测试: 客户端创建
- ⚠️ 集成测试: 需要有效 API key
- ⚠️ 端到端测试: 需要手动测试

### 下一步计划
- 添加更多单元测试
- 添加缓存层实现示例
- 添加图片下载辅助函数
- 添加批量操作支持
- 改进错误消息
- 添加重试机制
- 添加超时配置

### 贡献
欢迎贡献! 请确保:
- 代码符合官方 API 文档
- 添加适当的测试
- 更新文档
- 遵循 Rust 最佳实践

### 许可证
MIT OR Apache-2.0

### 致谢
- Pixabay 提供的免费 API
- Pexels Rust 库的设计灵感
- Rust 社区的优秀工具链
