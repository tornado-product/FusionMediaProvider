/*!
Poly Media Downloader CLI - 多媒体下载命令行工具。
支持从 Pexels 和 Pixabay 搜索和下载图片及视频。
*/
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use poly_media_provider::{
    DownloadConfig, DownloadProgress, MediaDownloader, MediaItem, MediaType, ProgressCallback,
    SearchParams,
};
use std::env;
use std::sync::Arc;

/// CLI 配置结构体
#[derive(Parser)]
#[command(name = "poly-media-provider")]
#[command(about = "多媒体下载命令行工具，支持从 Pexels 和 Pixabay 搜索和下载媒体", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// 可用的命令枚举
#[derive(Subcommand)]
enum Commands {
    /// 从所有提供商搜索媒体
    Search {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,

        /// 媒体类型 (image 或 video)
        #[arg(short, long, default_value = "image")]
        media_type: String,

        /// 每页结果数量
        #[arg(short, long, default_value = "20")]
        per_page: u32,

        /// 页码
        #[arg(long, default_value = "1")]
        page: u32,
    },

    /// 从指定提供商搜索媒体
    SearchProvider {
        /// 提供商名称 (pexels 或 pixabay)
        #[arg(short, long)]
        provider: String,

        /// 搜索关键词
        #[arg(short, long)]
        query: String,

        /// 媒体类型 (image 或 video)
        #[arg(short, long, default_value = "image")]
        media_type: String,

        /// 每页结果数量
        #[arg(short, long, default_value = "20")]
        per_page: u32,

        /// 页码
        #[arg(long, default_value = "1")]
        page: u32,
    },

    /// 下载指定媒体
    Download {
        /// 媒体 ID
        #[arg(short, long)]
        id: String,

        /// 媒体类型 (image 或 video)
        #[arg(short, long, default_value = "image")]
        media_type: String,

        /// 提供商名称 (可选，如果指定则只从该提供商下载)
        #[arg(short, long)]
        provider: Option<String>,
    },

    /// 批量下载搜索结果中的媒体
    DownloadSearch {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,

        /// 媒体类型 (image 或 video)
        #[arg(short, long, default_value = "image")]
        media_type: String,

        /// 每页结果数量
        #[arg(short, long, default_value = "10")]
        per_page: u32,

        /// 下载数量限制
        #[arg(long, default_value = "5")]
        limit: u32,

        /// 输出目录
        #[arg(short, long, default_value = "./downloads")]
        output_dir: String,
    },

    /// 列出所有已配置的提供商
    #[command(name = "list-providers")]
    ListProviders,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从 .env 文件加载环境变量
    dotenv().ok();

    // 解析命令行参数
    let cli = Cli::parse();

    // 创建下载器实例
    let downloader = MediaDownloader::new();

    match cli.command {
        Commands::Search {
            query,
            media_type,
            per_page,
            page,
        } => {
            // 解析媒体类型
            let media_type: MediaType = media_type.parse().unwrap_or(MediaType::Image);

            // 创建搜索参数
            let params = SearchParams::new(query, media_type)
                .per_page(per_page)
                .page(page);

            // 执行搜索
            let result = downloader.search(params).await?;

            // 打印结果
            println!("总共找到 {} 个结果", result.total);
            println!("当前页: {} / {}", result.page, result.total_pages);
            println!("提供商: {}", result.provider);
            println!("\n结果列表:");
            for (i, item) in result.items.iter().enumerate() {
                println!(
                    "{}. [{}] {} - {}",
                    i + 1,
                    item.media_type,
                    item.title,
                    item.source_url
                );
            }
        }

        Commands::SearchProvider {
            provider,
            query,
            media_type,
            per_page,
            page,
        } => {
            // 解析媒体类型
            let media_type: MediaType = media_type.parse().unwrap_or(MediaType::Image);

            // 创建搜索参数
            let params = SearchParams::new(query, media_type)
                .per_page(per_page)
                .page(page);

            // 从指定提供商搜索
            let result = downloader.search_from_provider(&provider, params).await?;

            // 打印结果
            println!("提供商: {}", result.provider);
            println!("总共找到 {} 个结果", result.total);
            println!("\n结果列表:");
            for (i, item) in result.items.iter().enumerate() {
                println!(
                    "{}. [{}] {} - {}",
                    i + 1,
                    item.media_type,
                    item.title,
                    item.source_url
                );
            }
        }

        Commands::Download {
            id,
            media_type,
            provider,
        } => {
            // 解析媒体类型
            let media_type: MediaType = media_type.parse().unwrap_or(MediaType::Image);

            // 如果指定了提供商，添加该提供商
            let downloader = match provider {
                Some(p) => {
                    let api_key = env::var(&format!("{}_API_KEY", p.to_uppercase()))
                        .unwrap_or_else(|_| panic!("请设置 {}_API_KEY 环境变量", p.to_uppercase()));
                    downloader.add_provider_by_name_and_apikey(&p, &api_key)
                }
                None => downloader,
            };

            // 下载媒体
            let file_path = downloader.download_by_id(&id, media_type).await?;
            println!("下载完成: {}", file_path);
        }

        Commands::DownloadSearch {
            query,
            media_type,
            per_page,
            limit,
            output_dir,
        } => {
            // 解析媒体类型
            let media_type: MediaType = media_type.parse().unwrap_or(MediaType::Image);

            // 配置下载器
            let config = DownloadConfig {
                output_dir,
                max_concurrent: 3,
                ..Default::default()
            };
            let downloader = downloader.with_config(config);

            // 执行搜索
            let params = SearchParams::new(query, media_type)
                .per_page(per_page)
                .page(1);

            let result = downloader.search(params).await?;
            println!("总共找到 {} 个结果", result.total);

            // 限制下载数量
            let items_to_download: Vec<&MediaItem> =
                result.items.iter().take(limit as usize).collect();
            println!("将下载 {} 个项目", items_to_download.len());

            // 创建进度回调
            let progress_callback: Option<ProgressCallback> =
                Some(Arc::new(|progress: DownloadProgress| {
                    println!(
                        "下载进度: {} - {:.1}%",
                        progress.item_title, progress.percentage
                    );
                }));

            // 下载媒体
            let downloaded_files = downloader
                .download_batch(&items_to_download, progress_callback)
                .await?;

            println!("\n下载完成！共成功下载 {} 个文件", downloaded_files.len());
            for file in &downloaded_files {
                println!("  - {}", file);
            }
        }

        Commands::ListProviders => {
            let providers = downloader.providers();
            if providers.is_empty() {
                println!("未配置任何提供商。请设置环境变量（如 PIXABAY_API_KEY, PEXELS_API_KEY）来添加提供商。");
            } else {
                println!("已配置的提供商:");
                for provider in providers {
                    println!("  - {}", provider.name());
                }
            }
        }
    }

    Ok(())
}
