use clap::{Parser, Subcommand};

/// Pexels CLI 命令行参数解析结构体
#[derive(Parser, Debug)]
#[clap(
    name = "pexels-sdk-cli",
    version = "0.0.1",
    about = "用于与 Pexels API 交互的命令行工具"
)]
pub struct Cli {
    /// 子命令
    #[clap(subcommand)]
    pub command: Command,
}

/// Pexels CLI 可用的命令枚举
#[derive(Subcommand, Debug)]
pub enum Command {
    /// 搜索照片
    SearchPhotos {
        /// 搜索关键词
        #[clap(short, long)]
        query: String,
        /// 每页结果数量
        #[clap(short, long, default_value = "15")]
        per_page: usize,
        /// 页码
        #[clap(short, long, default_value = "1")]
        page: usize,
    },
    /// 搜索视频
    SearchVideos {
        /// 搜索关键词
        #[clap(short, long)]
        query: String,
        /// 每页结果数量
        #[clap(short = 'n', long, default_value = "15")]
        per_page: usize,
        /// 页码
        #[clap(short, long, default_value = "1")]
        page: usize,
    },
    /// 根据 ID 获取特定照片
    GetPhoto {
        /// 照片 ID
        #[clap(short, long)]
        id: usize,
    },
    /// 根据 ID 获取特定视频
    GetVideo {
        /// 视频 ID
        #[clap(short, long)]
        id: usize,
    },
    /// 搜索收藏集
    SearchCollections {
        /// 每页结果数量
        #[clap(short, long, default_value = "15")]
        per_page: usize,
        /// 页码
        #[clap(short, long, default_value = "1")]
        page: usize,
    },
    /// 搜索媒体（照片和视频）
    SearchMedia {
        /// 搜索关键词
        #[clap(short, long)]
        query: String,
        /// 每页结果数量
        #[clap(short, long, default_value = "15")]
        per_page: usize,
        /// 页码
        #[clap(short, long, default_value = "1")]
        page: usize,
        /// 媒体类型（photo, video, all）
        #[clap(short, long, default_value = "")]
        r#type: String,
        /// 排序方式（asc, desc）
        #[clap(short, long, default_value = "asc")]
        sort: String,
    },
}
