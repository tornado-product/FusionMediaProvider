use pexels_sdk::{
    CollectionsResponse, MediaResponse, MediaSort, MediaType, MediaTypeResponse, Pexels, PexelsError,
    Photo, PhotosResponse, SearchBuilder, Video, VideoResponse, VideoSearchBuilder,
};
use std::env;

/// 搜索照片
///
/// # 参数
///
/// * `query` - 搜索关键词
/// * `per_page` - 每页结果数量
/// * `page` - 页码
pub async fn search_photos(
    query: &str,
    per_page: usize,
    page: usize,
) -> Result<PhotosResponse, PexelsError> {
    let _api_key = env::var("PEXELS_API_KEY")?;
    let client = Pexels::new(_api_key);
    let builder = SearchBuilder::new().query(query).per_page(per_page).page(page);
    let photos = client.search_photos(builder).await?;
    Ok(photos)
}

/// 搜索视频
///
/// # 参数
///
/// * `query` - 搜索关键词
/// * `per_page` - 每页结果数量
/// * `page` - 页码
pub async fn search_videos(
    query: &str,
    per_page: usize,
    page: usize,
) -> Result<VideoResponse, PexelsError> {
    let api_key = env::var("PEXELS_API_KEY")?;
    let client = Pexels::new(api_key);
    let builder = VideoSearchBuilder::new().query(query).per_page(per_page).page(page);
    let videos = client.search_videos(builder).await?;
    Ok(videos)
}

/// 根据 ID 获取照片
///
/// # 参数
///
/// * `id` - 照片 ID
pub async fn get_photo(id: usize) -> Result<Photo, PexelsError> {
    let api_key = env::var("PEXELS_API_KEY")?;
    let client = Pexels::new(api_key);
    let photo = client.get_photo(id).await?;
    Ok(photo)
}

/// 根据 ID 获取视频
///
/// # 参数
///
/// * `id` - 视频 ID
pub async fn get_video(id: usize) -> Result<Video, PexelsError> {
    let api_key = env::var("PEXELS_API_KEY")?;
    let client = Pexels::new(api_key);
    let video = client.get_video(id).await?;
    Ok(video)
}

/// 搜索收藏集
///
/// # 参数
///
/// * `per_page` - 每页结果数量
/// * `page` - 页码
pub async fn search_collections(
    per_page: usize,
    page: usize,
) -> Result<CollectionsResponse, PexelsError> {
    let api_key = env::var("PEXELS_API_KEY")?;
    let client = Pexels::new(api_key);
    let collections = client.search_collections(per_page, page).await?;
    Ok(collections)
}

/// 搜索媒体（照片和视频）
///
/// # 参数
///
/// * `query` - 搜索关键词
/// * `per_page` - 每页结果数量
/// * `page` - 页码
/// * `media_type` - 媒体类型
/// * `sort` - 排序方式
pub async fn search_media(
    query: &str,
    per_page: usize,
    page: usize,
    media_type: MediaType,
    _sort: MediaSort,
) -> Result<MediaResponse, PexelsError> {
    let api_key = env::var("PEXELS_API_KEY")?;
    let _client = Pexels::new(api_key);

    match media_type {
        MediaType::Photo => {
            let photos = search_photos(query, per_page, page).await?;
            let media: Vec<_> = photos.photos.into_iter().map(|p| {
                MediaTypeResponse::Photo(pexels_sdk::MediaPhoto {
                    type_: "Photo".to_string(),
                    id: p.id,
                    width: p.width,
                    height: p.height,
                    url: Some(p.url),
                    photographer: Some(p.photographer),
                    photographer_url: Some(p.photographer_url),
                    photographer_id: p.photographer_id,
                    avg_color: p.avg_color,
                    src: p.src,
                    liked: p.liked,
                    alt: p.alt,
                })
            }).collect();
            Ok(MediaResponse {
                id: "search".to_string(),
                media,
                page: photos.page,
                per_page: photos.per_page,
                total_results: photos.total_results,
                next_page: photos.next_page,
                prev_page: photos.prev_page,
            })
        }
        MediaType::Video => {
            let videos = search_videos(query, per_page, page).await?;
            let media: Vec<_> = videos.videos.into_iter().map(|v| {
                MediaTypeResponse::Video(pexels_sdk::MediaVideo {
                    type_: "Video".to_string(),
                    id: v.id,
                    width: v.width,
                    height: v.height,
                    duration: v.duration.unwrap_or(0),
                    full_res: v.full_res,
                    tags: v.tags,
                    url: Some(v.video_url),
                    image: Some(v.image_url),
                    avg_color: v.avg_color,
                    user: v.user,
                    video_files: v.video_files,
                    video_pictures: v.video_pictures,
                })
            }).collect();
            Ok(MediaResponse {
                id: "search".to_string(),
                media,
                page: videos.page,
                per_page: videos.per_page,
                total_results: videos.total_results,
                next_page: videos.next_page,
                prev_page: videos.prev_page,
            })
        }
        MediaType::Empty => {
            Err(PexelsError::ParseMediaTypeError)
        }
    }
}
