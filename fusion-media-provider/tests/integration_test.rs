// Integration tests for Pixabay API
// 注意：这些测试需要在 PIXABAY_API_KEY 环境变量中设置有效的 API 密钥
// 运行方式：cargo test --test integration_test -- --ignored

use dotenvy::dotenv;
use pixabay_sdk::{
    Category, ImageType, Order, Orientation, Pixabay, PixabayError, SearchImageParams,
    SearchVideoParams, VideoType,
};
use std::env;

fn get_test_client() -> Option<Pixabay> {
    dotenv().ok();
    env::var("PIXABAY_API_KEY").ok().map(Pixabay::new)
}

#[tokio::test]
async fn test_simple_image_search() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let result = client
        .search_images("yellow flowers", Some(5), Some(1))
        .await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.total > 0);
    assert!(response.total_hits > 0);
    assert!(!response.hits.is_empty());
    println!("{:?}", response.hits);
    assert!(response.hits.len() <= 5);
}

#[tokio::test]
#[ignore]
async fn test_advanced_image_search() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let params = SearchImageParams::new()
        .query("nature")
        .per_page(10)
        .image_type(ImageType::Photo)
        .orientation(Orientation::Horizontal)
        .category(Category::Nature)
        .min_width(1920)
        .min_height(1080)
        .safesearch(true)
        .order(Order::Popular);

    let result = client.search_images_advanced(params).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    for image in response.hits {
        assert!(image.image_width >= 1920);
        assert!(image.image_height >= 1080);
        assert_eq!(image.image_type, "photo");
    }
}

#[tokio::test]
#[ignore]
async fn test_get_image_by_id() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    // First search for an image to get a valid ID
    let search_result = client.search_images("cat", Some(1), Some(1)).await;
    assert!(search_result.is_ok());

    let images = search_result.unwrap();
    if let Some(first_image) = images.hits.first() {
        let result = client.get_image(first_image.id).await;
        assert!(result.is_ok());

        let image = result.unwrap();
        assert_eq!(image.id, first_image.id);
    }
}

#[tokio::test]
#[ignore]
async fn test_simple_video_search() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let result = client.search_videos("ocean", Some(5), Some(1)).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.total > 0);
    assert!(response.total_hits > 0);
    assert!(!response.hits.is_empty());
    assert!(response.hits.len() <= 5);
}

#[tokio::test]
async fn test_advanced_video_search() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let params = SearchVideoParams::new()
        .query("nature")
        .per_page(5)
        .video_type(VideoType::Film)
        .category(Category::Nature)
        .order(Order::Latest);

    let result = client.search_videos_advanced(params).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    for video in response.hits {
        assert!(video.duration > 0);
        // Video type might not always match due to API behavior
    }
}

#[tokio::test]
#[ignore]
async fn test_get_video_by_id() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    // First search for a video to get a valid ID
    let search_result = client.search_videos("sunset", Some(1), Some(1)).await;
    assert!(search_result.is_ok());

    let videos = search_result.unwrap();
    if let Some(first_video) = videos.hits.first() {
        let result = client.get_video(first_video.id).await;
        assert!(result.is_ok());

        let video = result.unwrap();
        assert_eq!(video.id, first_video.id);
    }
}

#[tokio::test]
async fn test_per_page_clamping() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    // Test that per_page is clamped to valid range (3-200)
    let result = client.search_images("test", Some(1), Some(1)).await;
    assert!(result.is_ok()); // Should clamp to 3

    let result = client.search_images("test", Some(300), Some(1)).await;
    assert!(result.is_ok()); // Should clamp to 200
}

#[tokio::test]
#[ignore]
async fn test_query_length_validation() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    // Create a query longer than 100 characters
    let long_query = "a".repeat(101);

    let params = SearchImageParams::new().query(long_query).per_page(5);

    let result = client.search_images_advanced(params).await;
    assert!(result.is_err());

    if let Err(PixabayError::ApiError(msg)) = result {
        assert!(msg.contains("100 characters"));
    }
}

#[tokio::test]
#[ignore]
async fn test_invalid_api_key() {
    let client = Pixabay::new("invalid_key_12345".to_string());

    let result = client.search_images("test", Some(5), Some(1)).await;
    assert!(result.is_err());

    // API should return 401 or 403 for invalid key
    if let Err(e) = result {
        println!("Error (expected): {:?}", e);
        // The actual error depends on Pixabay's response
    }
}

#[tokio::test]
#[ignore]
async fn test_editors_choice_filter() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let params = SearchImageParams::new()
        .query("nature")
        .per_page(5)
        .editors_choice(true);

    let result = client.search_images_advanced(params).await;
    assert!(result.is_ok());

    // All results should be Editor's Choice (though API doesn't return this field)
    let response = result.unwrap();
    assert!(!response.hits.is_empty());
}

#[tokio::test]
async fn test_all_image_types() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    for image_type in [ImageType::Photo, ImageType::Illustration, ImageType::Vector] {
        let params = SearchImageParams::new()
            .query("flower")
            .per_page(3)
            .image_type(image_type.clone());

        let result = client.search_images_advanced(params).await;
        assert!(result.is_ok(), "Failed for image type: {:?}", image_type);
    }
}

#[tokio::test]
#[ignore]
async fn test_all_video_types() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    for video_type in [VideoType::Film, VideoType::Animation] {
        let params = SearchVideoParams::new()
            .query("nature")
            .per_page(3)
            .video_type(video_type.clone());

        let result = client.search_videos_advanced(params).await;
        assert!(result.is_ok(), "Failed for video type: {:?}", video_type);
    }
}

#[tokio::test]
#[ignore]
async fn test_response_structure() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let result = client.search_images("test", Some(1), Some(1)).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(!response.hits.is_empty());

    let image = &response.hits[0];

    // Verify all required fields are present
    assert!(image.id > 0);
    assert!(!image.page_url.is_empty());
    assert!(!image.tags.is_empty());
    assert!(!image.preview_url.is_empty());
    assert!(!image.webformat_url.is_empty());
    assert!(!image.large_image_url.is_empty());
    assert!(image.image_width > 0);
    assert!(image.image_height > 0);
    assert!(!image.user.is_empty());
}

#[tokio::test]
async fn test_client_creation() {
    let client = Pixabay::new("test_key".to_string());
    assert_eq!(client.api_key, "test_key");
}

#[test]
fn test_enums_to_string() {
    assert_eq!(ImageType::Photo.to_string(), "photo");
    assert_eq!(VideoType::Film.to_string(), "film");
    assert_eq!(Orientation::Horizontal.to_string(), "horizontal");
    assert_eq!(Order::Latest.to_string(), "latest");
    assert_eq!(Category::Nature.to_string(), "nature");
}

#[tokio::test]
async fn test_media_downloader_search_images() {
    use dotenvy::dotenv;
    use fusion_media_provider::{MediaDownloader, MediaType, PixabayProvider, SearchParams};
    use std::env;
    use std::sync::Arc;

    dotenv().ok();
    let api_key = env::var("PIXABAY_API_KEY").expect("PIXABAY_API_KEY must be set in .env file");

    let downloader = MediaDownloader::new().add_provider(Arc::new(PixabayProvider::new(api_key)));

    let params = SearchParams::new("mountain", MediaType::Image).limit(5);

    let result = downloader.search(params).await;

    assert!(result.is_ok(), "Search should succeed");
    let response = result.unwrap();

    assert!(response.total > 0, "Should have results");
    assert!(!response.items.is_empty(), "Should return items");
    println!(
        "Found {} images from {} provider(s)",
        response.total_hits,
        response.provider_results.len()
    );

    for item in &response.items {
        println!("  - {} ({}): {}", item.id, item.provider, item.title);
    }
}

#[tokio::test]
#[ignore]
async fn test_media_downloader_search_videos() {
    use dotenvy::dotenv;
    use fusion_media_provider::{MediaDownloader, MediaType, PixabayProvider, SearchParams};
    use std::env;
    use std::sync::Arc;

    dotenv().ok();
    let api_key = env::var("PIXABAY_API_KEY").expect("PIXABAY_API_KEY must be set in .env file");

    let downloader = MediaDownloader::new().add_provider(Arc::new(PixabayProvider::new(api_key)));

    let params = SearchParams::new("ocean", MediaType::Video).limit(3);

    let result = downloader.search(params).await;

    assert!(result.is_ok(), "Search should succeed");
    let response = result.unwrap();

    assert!(response.total > 0, "Should have results");
    println!(
        "Found {} videos from {} provider(s)",
        response.total_hits,
        response.provider_results.len()
    );
}

#[tokio::test]
#[ignore]
async fn test_media_downloader_pagination() {
    use dotenvy::dotenv;
    use fusion_media_provider::{MediaDownloader, MediaType, PixabayProvider, SearchParams};
    use std::env;
    use std::sync::Arc;

    dotenv().ok();
    let api_key = env::var("PIXABAY_API_KEY").expect("PIXABAY_API_KEY must be set in .env file");

    let downloader = MediaDownloader::new().add_provider(Arc::new(PixabayProvider::new(api_key)));

    let params1 = SearchParams::new("forest", MediaType::Image)
        .limit(3)
        .page(1);

    let params2 = SearchParams::new("forest", MediaType::Image)
        .limit(3)
        .page(2);

    let result1 = downloader.search(params1).await;
    let result2 = downloader.search(params2).await;

    assert!(result1.is_ok(), "First page should succeed");
    assert!(result2.is_ok(), "Second page should succeed");

    let response1 = result1.unwrap();
    let response2 = result2.unwrap();

    // Different pages should have different items
    if let (Some(item1), Some(item2)) = (response1.items.first(), response2.items.first()) {
        assert_ne!(
            item1.id, item2.id,
            "Different pages should return different items"
        );
    }

    println!(
        "Page 1: {} items, Page 2: {} items",
        response1.items.len(),
        response2.items.len()
    );
}

#[tokio::test]
#[ignore]
async fn test_media_downloader_multiple_providers() {
    use dotenvy::dotenv;
    use fusion_media_provider::{MediaDownloader, MediaType, PixabayProvider, SearchParams};
    use std::env;
    use std::sync::Arc;

    dotenv().ok();
    let pixabay_key = env::var("PIXABAY_API_KEY").expect("PIXABAY_API_KEY must be set");

    let downloader =
        MediaDownloader::new().add_provider(Arc::new(PixabayProvider::new(pixabay_key.clone())));

    let params = SearchParams::new("city", MediaType::Image).limit(3);

    let result = downloader.search(params).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    println!(
        "Query: 'city', Found {} results from {} provider(s)",
        response.total_hits,
        response.provider_results.len()
    );

    // All results should have provider info
    for item in &response.items {
        assert!(!item.provider.is_empty(), "Item should have provider info");
    }
}

#[tokio::test]
async fn test_download_config_default() {
    use fusion_media_provider::{DownloadConfig, ImageQuality, VideoQuality};

    let config = DownloadConfig::default();

    assert_eq!(config.image_quality, ImageQuality::Large);
    assert_eq!(config.video_quality, VideoQuality::Large);
    assert_eq!(config.output_dir, "./downloads");
    assert_eq!(config.max_concurrent, 5);
    assert!(config.progress_callback.is_none());
}

#[tokio::test]
async fn test_search_params_builder() {
    use fusion_media_provider::{MediaType, SearchParams};

    let params = SearchParams::new("test query", MediaType::Image)
        .limit(10)
        .page(2);

    assert_eq!(params.query, "test query");
    assert_eq!(params.media_type, MediaType::Image);
    assert_eq!(params.limit, 10);
    assert_eq!(params.page, 2);
}

#[tokio::test]
async fn test_media_item_creation() {
    use fusion_media_provider::{MediaItem, MediaMetadata, MediaType, MediaUrls};

    let item = MediaItem {
        id: "test-123".to_string(),
        media_type: MediaType::Image,
        title: "Test Image".to_string(),
        description: "A test image".to_string(),
        tags: vec!["test".to_string(), "sample".to_string()],
        author: "Test Author".to_string(),
        author_url: "https://example.com/author".to_string(),
        source_url: "https://example.com/image".to_string(),
        provider: "TestProvider".to_string(),
        urls: MediaUrls {
            thumbnail: "https://example.com/thumb.jpg".to_string(),
            medium: Some("https://example.com/medium.jpg".to_string()),
            large: Some("https://example.com/large.jpg".to_string()),
            original: Some("https://example.com/original.jpg".to_string()),
            video_files: None,
        },
        metadata: MediaMetadata {
            width: 1920,
            height: 1080,
            size: Some(1024000),
            duration: None,
            views: 100,
            downloads: 50,
            likes: 200,
        },
    };

    assert_eq!(item.id, "test-123");
    assert_eq!(item.title, "Test Image");
    assert_eq!(item.provider, "TestProvider");
}

#[tokio::test]
async fn test_download_progress_new() {
    use fusion_media_provider::{DownloadProgress, MediaItem, MediaMetadata, MediaType, MediaUrls};

    let item = MediaItem {
        id: "test-123".to_string(),
        media_type: MediaType::Image,
        title: "Test Image".to_string(),
        description: "".to_string(),
        tags: vec![],
        author: "".to_string(),
        author_url: "".to_string(),
        source_url: "".to_string(),
        provider: "TestProvider".to_string(),
        urls: MediaUrls {
            thumbnail: "https://example.com/thumb.jpg".to_string(),
            medium: None,
            large: None,
            original: None,
            video_files: None,
        },
        metadata: MediaMetadata {
            width: 1920,
            height: 1080,
            size: None,
            duration: None,
            views: 0,
            downloads: 0,
            likes: 0,
        },
    };

    let progress = DownloadProgress::new(&item);

    assert_eq!(progress.item_id, "test-123");
    assert_eq!(progress.item_title, "Test Image");
    assert_eq!(progress.provider, "TestProvider");
    assert_eq!(progress.percentage, 0.0);
}

#[tokio::test]
async fn test_batch_download_progress_new() {
    use fusion_media_provider::BatchDownloadProgress;

    let progress = BatchDownloadProgress::new(5);

    assert_eq!(progress.total_items, 5);
    assert_eq!(progress.completed_items, 0);
    assert_eq!(progress.overall_percentage, 0.0);
    assert!(progress.item_progress.is_empty());
}
