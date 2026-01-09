/*!
Pexels CLI - 用于与 Pexels API 交互的命令行工具。
*/
mod api;
mod cli;

use crate::api::{
    get_photo, get_video, search_collections, search_media, search_photos, search_videos,
};
use crate::cli::Cli;
use clap::Parser;
use dotenvy::dotenv;
use pexels_sdk::{MediaSort, MediaType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从 .env 文件加载环境变量
    dotenv().ok();

    // 解析命令行参数
    let args = Cli::parse();

    // 匹配命令并执行对应的函数
    match args.command {
        cli::Command::SearchPhotos {
            query,
            per_page,
            page,
        } => {
            // 根据查询搜索照片
            let photos = search_photos(&query, per_page, page).await?;
            for photo in photos.photos {
                println!("{photo:?}");
            }
        }
        cli::Command::SearchVideos {
            query,
            per_page,
            page,
        } => {
            // 根据查询搜索视频
            let videos = search_videos(&query, per_page, page).await?;
            for video in videos.videos {
                println!("{video:?}");
            }
        }
        cli::Command::GetPhoto { id } => {
            // 根据 ID 获取照片
            let photo = get_photo(id).await?;
            println!("{photo:?}");
        }
        cli::Command::GetVideo { id } => {
            // 根据 ID 获取视频
            let video = get_video(id).await?;
            println!("{video:?}");
        }
        cli::Command::SearchCollections { per_page, page } => {
            // 搜索收藏集
            let collections = search_collections(per_page, page).await?;
            for collection in collections.collections {
                println!("{collection:?}");
            }
        }
        cli::Command::SearchMedia {
            query,
            per_page,
            page,
            r#type,
            sort,
        } => {
            // 根据查询搜索媒体（照片和视频）
            let mtype = r#type.parse::<MediaType>()?;
            let msort = sort.parse::<MediaSort>()?;
            let media_response = search_media(&query, per_page, page, mtype, msort).await?;
            for media in media_response.media {
                println!("{media:?}");
            }
        }
    }

    Ok(())
}
