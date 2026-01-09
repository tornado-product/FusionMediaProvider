# 分页功能说明

Media Downloader 提供完整的分页支持,包括多源聚合时的总页数计算。

## 核心概念

### SearchResult (单个 Provider 的结果)

```rust
pub struct SearchResult {
    pub total: u32,           // 该 provider 的总结果数
    pub total_hits: u32,      // 该 provider 返回的结果数 (可能受限)
    pub page: u32,            // 当前页码
    pub per_page: u32,        // 每页结果数
    pub total_pages: u32,     // 该 provider 的总页数
    pub items: Vec<MediaItem>,// 媒体项
    pub provider: String,     // Provider 名称
}
```

### AggregatedSearchResult (多个 Provider 的聚合结果)

```rust
pub struct AggregatedSearchResult {
    pub total: u32,                       // 所有 provider 的总结果数之和
    pub total_hits: u32,                  // 所有 provider 返回的结果数之和
    pub page: u32,                        // 当前页码
    pub per_page: u32,                    // 每页结果数
    pub total_pages: u32,                 // 所有 provider 的总页数之和
    pub items: Vec<MediaItem>,            // 所有媒体项
    pub provider_results: Vec<SearchResult>, // 各 provider 的详细结果
}
```

## 总页数计算

### 单个 Provider

```rust
total_pages = (total + per_page - 1) / per_page  // 向上取整
```

**示例**:
- `total = 100, per_page = 20` → `total_pages = 5`
- `total = 99, per_page = 20` → `total_pages = 5`
- `total = 101, per_page = 20` → `total_pages = 6`

### 多个 Provider (聚合)

```rust
total_pages = sum(each_provider.total_pages)
```

**示例**:
- Pixabay: 100 results, 20 per page → 5 pages
- Pexels: 80 results, 20 per page → 4 pages
- **总计**: 9 pages (5 + 4)

## 使用示例

### 示例 1: 单个 Provider 的分页

```rust
let downloader = MediaDownloader::new()
    .add_provider(Arc::new(PixabayProvider::new(api_key)));

// 获取第一页
let page1 = downloader.search_from_provider(
    "Pixabay",
    SearchParams::new("nature", MediaType::Image)
        .limit(20)
        .page(1)
).await?;

println!("Total results: {}", page1.total);
println!("Total pages: {}", page1.total_pages);
println!("Current page: {}", page1.page);

// 遍历所有页
for page_num in 1..=page1.total_pages {
    let result = downloader.search_from_provider(
        "Pixabay",
        SearchParams::new("nature", MediaType::Image)
            .limit(20)
            .page(page_num)
    ).await?;
    
    println!("Page {}/{}: {} items", 
        page_num, result.total_pages, result.items.len());
}
```

### 示例 2: 多 Provider 聚合分页

```rust
let downloader = MediaDownloader::new()
    .add_provider(Arc::new(PixabayProvider::new(pixabay_key)))
    .add_provider(Arc::new(PexelsProvider::new(pexels_key)));

// 搜索所有 provider
let results = downloader.search(
    SearchParams::new("sunset", MediaType::Image).limit(20)
).await?;

println!("📊 Aggregated Statistics:");
println!("  Total results: {} (across all providers)", results.total);
println!("  Total pages: {} (sum of all providers)", results.total_pages);
println!("  Items in current page: {}", results.items.len());

println!("\n📦 Per-provider breakdown:");
for provider_result in &results.provider_results {
    println!("  {}:", provider_result.provider);
    println!("    - Total: {}", provider_result.total);
    println!("    - Pages: {}", provider_result.total_pages);
    println!("    - Items: {}", provider_result.items.len());
}
```

### 示例 3: 分页浏览

```rust
async fn browse_all_pages(
    downloader: &MediaDownloader,
    query: &str,
) -> Result<Vec<MediaItem>> {
    let mut all_items = Vec::new();
    
    // 获取第一页以了解总页数
    let first_page = downloader.search(
        SearchParams::new(query, MediaType::Image).limit(20).page(1)
    ).await?;
    
    println!("Total pages to fetch: {}", first_page.total_pages);
    all_items.extend(first_page.items);
    
    // 获取剩余页面
    for page_num in 2..=first_page.total_pages {
        let result = downloader.search(
            SearchParams::new(query, MediaType::Image)
                .limit(20)
                .page(page_num)
        ).await?;
        
        all_items.extend(result.items);
        println!("Fetched page {}/{}", page_num, first_page.total_pages);
    }
    
    Ok(all_items)
}
```

### 示例 4: 查看各 Provider 的贡献

```rust
let results = downloader.search(params).await?;

println!("Total items: {}", results.items.len());
println!("From {} providers:", results.provider_results.len());

for provider_result in &results.provider_results {
    let percentage = (provider_result.items.len() as f64 
                     / results.items.len() as f64) * 100.0;
    
    println!("\n{} Provider:", provider_result.provider);
    println!("  Contribution: {:.1}% ({} items)", 
        percentage, provider_result.items.len());
    println!("  Total available: {}", provider_result.total);
    println!("  Pages available: {}", provider_result.total_pages);
}
```

## 重要注意事项

### 1. API 限制

不同的 provider 有不同的结果限制:

**Pixabay**:
- `total`: 实际匹配的总数
- `totalHits`: 最多返回 500 (API 限制)
- 即使 `total = 10000`,你最多只能访问前 500 个结果

**Pexels**:
- 每页最多 80 个结果
- 具体总数限制参见 Pexels API 文档

### 2. 聚合分页的含义

当使用多个 provider 时:

```rust
// Pixabay: 500 results → 25 pages (20 per page)
// Pexels: 320 results → 16 pages (20 per page)
// Total: 820 results, 41 pages (25 + 16)
```

**注意**: `total_pages = 41` 意味着:
- 你需要对每个 provider 分别进行分页
- 不是说有 41 页可以直接获取
- 它表示**如果遍历所有 provider 的所有页**,总共有 41 页

### 3. 正确的多页获取方式

```rust
// ❌ 错误方式 - 这不会获取所有数据
for page in 1..=results.total_pages {
    let data = downloader.search(
        SearchParams::new("query", MediaType::Image).page(page)
    ).await?;
    // 这只会重复获取每个 provider 的第 N 页
}

// ✅ 正确方式 - 分别处理每个 provider
for provider_result in &results.provider_results {
    for page in 1..=provider_result.total_pages {
        let data = downloader.search_from_provider(
            &provider_result.provider,
            SearchParams::new("query", MediaType::Image).page(page)
        ).await?;
        // 处理数据
    }
}
```

### 4. 性能考虑

```rust
// 获取大量数据时,使用合理的 per_page 值
let params = SearchParams::new("nature", MediaType::Image)
    .limit(100)  // 更大的 per_page 减少请求次数
    .page(1);

// 但要注意 API 限制
// Pixabay: 3-200
// Pexels: 最多 80
```

## 实用函数

### 计算总页数

```rust
use media_downloader::SearchResult;

let total_pages = SearchResult::calculate_total_pages(total, per_page);
```

### 检查是否还有更多页

```rust
fn has_more_pages(result: &SearchResult) -> bool {
    result.page < result.total_pages
}

fn get_next_page(result: &SearchResult) -> Option<u32> {
    if has_more_pages(result) {
        Some(result.page + 1)
    } else {
        None
    }
}
```

### 批量获取助手

```rust
async fn fetch_pages_range(
    downloader: &MediaDownloader,
    provider: &str,
    query: &str,
    from_page: u32,
    to_page: u32,
    per_page: u32,
) -> Result<Vec<MediaItem>> {
    let mut all_items = Vec::new();
    
    for page in from_page..=to_page {
        let result = downloader.search_from_provider(
            provider,
            SearchParams::new(query, MediaType::Image)
                .limit(per_page)
                .page(page)
        ).await?;
        
        all_items.extend(result.items);
        
        // 尊重 API 速率限制
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    Ok(all_items)
}

// 使用
let items = fetch_pages_range(
    &downloader, 
    "Pixabay", 
    "mountains", 
    1, 
    5,  // 获取前 5 页
    20
).await?;
```

## 调试技巧

### 打印分页信息

```rust
fn print_pagination_info(result: &AggregatedSearchResult) {
    println!("=== Pagination Info ===");
    println!("Total results: {}", result.total);
    println!("Total pages: {}", result.total_pages);
    println!("Current page: {}/{}", result.page, result.total_pages);
    println!("Items per page: {}", result.per_page);
    println!("Items in current response: {}", result.items.len());
    
    println!("\n=== Provider Details ===");
    for pr in &result.provider_results {
        println!("{}: {} items, {} pages", 
            pr.provider, pr.total, pr.total_pages);
    }
}
```

### 验证分页一致性

```rust
fn verify_pagination(result: &SearchResult) -> bool {
    let calculated = SearchResult::calculate_total_pages(
        result.total, 
        result.per_page
    );
    
    if calculated != result.total_pages {
        eprintln!("Warning: Pagination mismatch!");
        eprintln!("  Expected: {}", calculated);
        eprintln!("  Got: {}", result.total_pages);
        return false;
    }
    
    true
}
```

## 总结

✅ **单个 Provider**: 直接使用 `search_from_provider()` 和 `page` 参数

✅ **多个 Provider**: `search()` 返回聚合结果,`total_pages` 是所有 provider 页数之和

✅ **获取所有数据**: 需要分别遍历每个 provider 的页面

✅ **性能优化**: 使用合理的 `per_page` 值,注意 API 速率限制

✅ **调试**: 使用 `provider_results` 查看各 provider 的详细信息