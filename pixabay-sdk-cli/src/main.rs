/*!
Pixabay CLI - 用于与 Pixabay API 交互的命令行工具。
*/
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use pixabay_sdk::{
    Category, ImageType, Order, Orientation, Pixabay, SearchImageParams, SearchVideoParams,
    VideoType,
};
use std::env;

/// Pixabay CLI 命令行参数解析结构体
#[derive(Parser)]
#[command(name = "pixabay")]
#[command(about = "用于与 Pixabay API 交互的命令行工具", long_about = None)]
struct Cli {
    /// 子命令
    #[command(subcommand)]
    command: Commands,
}

/// Pixabay CLI 可用的命令枚举
#[derive(Subcommand)]
enum Commands {
    /// 搜索图片
    SearchImages {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,

        /// 每页结果数量（最大 200）
        #[arg(short, long, default_value = "20")]
        per_page: u32,

        /// 页码
        #[arg(long, default_value = "1")]
        page: u32,

        /// 图片类型（all, photo, illustration, vector）
        #[arg(short = 't', long)]
        image_type: Option<String>,

        /// 方向（all, horizontal, vertical）
        #[arg(short, long)]
        orientation: Option<String>,

        /// 分类
        #[arg(short, long)]
        category: Option<String>,

        /// 最小宽度
        #[arg(long)]
        min_width: Option<u32>,

        /// 最小高度
        #[arg(long)]
        min_height: Option<u32>,

        /// 排序（popular, latest）
        #[arg(long)]
        order: Option<String>,

        /// 仅编辑精选
        #[arg(long)]
        editors_choice: bool,

        /// 启用安全搜索
        #[arg(long)]
        safesearch: bool,
    },

    /// 根据 ID 获取指定图片
    GetImage {
        /// 图片 ID
        #[arg(short, long)]
        id: u64,
    },

    /// 搜索视频
    SearchVideos {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,

        /// 每页结果数量（最大 200）
        #[arg(short, long, default_value = "20")]
        per_page: u32,

        /// 页码
        #[arg(long, default_value = "1")]
        page: u32,

        /// 视频类型（all, film, animation）
        #[arg(short = 't', long)]
        video_type: Option<String>,

        /// 分类
        #[arg(short, long)]
        category: Option<String>,

        /// 最小宽度
        #[arg(long)]
        min_width: Option<u32>,

        /// 最小高度
        #[arg(long)]
        min_height: Option<u32>,

        /// 排序（popular, latest）
        #[arg(long)]
        order: Option<String>,

        /// 仅编辑精选
        #[arg(long)]
        editors_choice: bool,

        /// 启用安全搜索
        #[arg(long)]
        safesearch: bool,
    },

    /// 根据 ID 获取指定视频
    GetVideo {
        /// 视频 ID
        #[arg(short, long)]
        id: u64,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从 .env 文件加载环境变量
    dotenv().ok();

    // 从环境变量获取 API 密钥
    let api_key =
        env::var("PIXABAY_API_KEY").expect("必须在环境变量或 .env 文件中设置 PIXABAY_API_KEY");

    // 创建 Pixabay 客户端
    let client = Pixabay::new(api_key);

    // 解析命令行参数
    let cli = Cli::parse();

    match cli.command {
        Commands::SearchImages {
            query,
            per_page,
            page,
            image_type,
            orientation,
            category,
            min_width,
            min_height,
            order,
            editors_choice,
            safesearch,
        } => {
            // 构建搜索图片参数
            let mut params = SearchImageParams::new()
                .query(query)
                .per_page(per_page)
                .page(page);

            // 设置图片类型
            if let Some(it) = image_type {
                params = params.image_type(match it.as_str() {
                    "photo" => ImageType::Photo,
                    "illustration" => ImageType::Illustration,
                    "vector" => ImageType::Vector,
                    _ => ImageType::All,
                });
            }

            // 设置方向
            if let Some(o) = orientation {
                params = params.orientation(match o.as_str() {
                    "horizontal" => Orientation::Horizontal,
                    "vertical" => Orientation::Vertical,
                    _ => Orientation::All,
                });
            }

            // 设置分类
            if let Some(c) = category {
                if let Some(cat) = parse_category(&c) {
                    params = params.category(cat);
                }
            }

            // 设置最小宽度
            if let Some(mw) = min_width {
                params = params.min_width(mw);
            }

            // 设置最小高度
            if let Some(mh) = min_height {
                params = params.min_height(mh);
            }

            // 设置排序方式
            if let Some(ord) = order {
                params = params.order(match ord.as_str() {
                    "latest" => Order::Latest,
                    _ => Order::Popular,
                });
            }

            // 设置编辑精选
            if editors_choice {
                params = params.editors_choice(true);
            }

            // 设置安全搜索
            if safesearch {
                params = params.safesearch(true);
            }

            // 执行高级图片搜索
            let response = client.search_images_advanced(params).await?;
            println!("{}", serde_json::to_string_pretty(&response)?);
        }

        Commands::GetImage { id } => {
            // 根据 ID 获取图片
            let image = client.get_image(id).await?;
            println!("{}", serde_json::to_string_pretty(&image)?);
        }

        Commands::SearchVideos {
            query,
            per_page,
            page,
            video_type,
            category,
            min_width,
            min_height,
            order,
            editors_choice,
            safesearch,
        } => {
            // 构建搜索视频参数
            let mut params = SearchVideoParams::new()
                .query(query)
                .per_page(per_page)
                .page(page);

            // 设置视频类型
            if let Some(vt) = video_type {
                params = params.video_type(match vt.as_str() {
                    "film" => VideoType::Film,
                    "animation" => VideoType::Animation,
                    _ => VideoType::All,
                });
            }

            // 设置分类
            if let Some(c) = category {
                if let Some(cat) = parse_category(&c) {
                    params = params.category(cat);
                }
            }

            // 设置最小宽度
            if let Some(mw) = min_width {
                params = params.min_width(mw);
            }

            // 设置最小高度
            if let Some(mh) = min_height {
                params = params.min_height(mh);
            }

            // 设置排序方式
            if let Some(ord) = order {
                params = params.order(match ord.as_str() {
                    "latest" => Order::Latest,
                    _ => Order::Popular,
                });
            }

            // 设置编辑精选
            if editors_choice {
                params = params.editors_choice(true);
            }

            // 设置安全搜索
            if safesearch {
                params = params.safesearch(true);
            }

            // 执行高级视频搜索
            let response = client.search_videos_advanced(params).await?;
            println!("hits lent:{}", response.hits.len());
            let first_video = response.hits.first();
            if let Some(video) = first_video {
                println!("video1 url:{}", video.page_url);
                let video_result = client.get_video(video.id).await;
                if let Ok(video) = video_result {
                    println!("videoUrl:{}", video.page_url);
                }
            }

            //println!("{}", serde_json::to_string_pretty(&response)?);
        }

        Commands::GetVideo { id } => {
            // 根据 ID 获取视频
            let video = client.get_video(id).await?;
            println!("{}", serde_json::to_string_pretty(&video)?);
        }
    }

    Ok(())
}

/// 解析分类字符串为 Category 枚举
fn parse_category(s: &str) -> Option<Category> {
    match s.to_lowercase().as_str() {
        "backgrounds" => Some(Category::Backgrounds),
        "fashion" => Some(Category::Fashion),
        "nature" => Some(Category::Nature),
        "science" => Some(Category::Science),
        "education" => Some(Category::Education),
        "feelings" => Some(Category::Feelings),
        "health" => Some(Category::Health),
        "people" => Some(Category::People),
        "religion" => Some(Category::Religion),
        "places" => Some(Category::Places),
        "animals" => Some(Category::Animals),
        "industry" => Some(Category::Industry),
        "computer" => Some(Category::Computer),
        "food" => Some(Category::Food),
        "sports" => Some(Category::Sports),
        "transportation" => Some(Category::Transportation),
        "travel" => Some(Category::Travel),
        "buildings" => Some(Category::Buildings),
        "business" => Some(Category::Business),
        "music" => Some(Category::Music),
        _ => None,
    }
}
