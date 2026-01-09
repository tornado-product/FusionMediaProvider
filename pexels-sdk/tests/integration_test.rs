use dotenvy::dotenv;
use pexels_sdk::{
    CuratedBuilder, Pexels, PexelsError, PopularBuilder, SearchBuilder, VideoSearchBuilder,
};
use std::env;

fn get_test_client() -> Option<Pexels> {
    dotenv().ok();
    env::var("PEXELS_API_KEY").ok().map(Pexels::new)
}

#[tokio::test]
#[ignore]
async fn test_search_photos() {
    let client = get_test_client().expect("PEXELS_API_KEY not set");

    let result = client
        .search_photos(
            SearchBuilder::new()
                .query("yellow flowers")
                .per_page(5)
                .page(1),
        )
        .await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.total_results > 0);
    assert!(!response.photos.is_empty());
    println!("Found {} photos", response.photos.len());
}

#[tokio::test]
#[ignore]
async fn test_search_videos() {
    let client = get_test_client().expect("PEXELS_API_KEY not set");

    let result = client
        .search_videos(
            VideoSearchBuilder::new()
                .query("ocean waves")
                .per_page(3)
                .page(1),
        )
        .await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.total_results > 0);
    println!("Found {} videos", response.videos.len());
}

#[tokio::test]
#[ignore]
async fn test_get_photo_by_id() {
    let client = get_test_client().expect("PEXELS_API_KEY not set");

    let result = client.get_photo(12345).await;
    match result {
        Ok(photo) => {
            assert_eq!(photo.id, 12345);
            println!("Photo: {} by {}", photo.id, photo.photographer);
        }
        Err(PexelsError::NotFound(_)) => {
            println!("Photo 12345 not found (expected for non-existent ID)");
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_get_video_by_id() {
    let client = get_test_client().expect("PEXELS_API_KEY not set");

    let result = client.get_video(12345).await;
    match result {
        Ok(video) => {
            assert_eq!(video.id, 12345);
            println!(
                "Video: {} - duration: {}s",
                video.id,
                video.duration.unwrap_or(0)
            );
        }
        Err(PexelsError::NotFound(_)) => {
            println!("Video 12345 not found (expected for non-existent ID)");
        }
        Err(e) => {
            println!("Get video returned error (might be API behavior): {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_curated_photos() {
    let client = get_test_client().expect("PEXELS_API_KEY not set");

    let result = client
        .curated_photo(CuratedBuilder::new().per_page(10).page(1))
        .await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(!response.photos.is_empty());
    println!("Curated photos: {}", response.photos.len());
}

#[tokio::test]
#[ignore]
async fn test_popular_videos() {
    let client = get_test_client().expect("PEXELS_API_KEY not set");

    let result = client
        .popular_videos(PopularBuilder::new().per_page(10).page(1))
        .await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(!response.videos.is_empty());
    println!("Popular videos: {}", response.videos.len());
}

#[tokio::test]
#[ignore]
async fn test_search_with_different_pages() {
    let client = get_test_client().expect("PEXELS_API_KEY not set");

    let result = client
        .search_photos(SearchBuilder::new().query("nature").per_page(5).page(1))
        .await;
    assert!(result.is_ok(), "Page 1 should return valid results");
    let page1 = result.unwrap();
    let result = client
        .search_photos(SearchBuilder::new().query("nature").per_page(5).page(2))
        .await;
    assert!(result.is_ok(), "Page 2 should return valid results");
    let page2 = result.unwrap();

    assert!(!page1.photos.is_empty());
    assert!(!page2.photos.is_empty());

    if let (Some(item1), Some(item2)) = (page1.photos.first(), page2.photos.first()) {
        assert_ne!(
            item1.id, item2.id,
            "Different pages should return different items"
        );
    }

    println!(
        "Page 1 items: {}, Page 2 items: {}",
        page1.photos.len(),
        page2.photos.len()
    );
}

#[tokio::test]
#[ignore]
async fn test_invalid_api_key() {
    dotenv().ok();
    std::env::set_var("PEXELS_API_KEY", "invalid_key");

    let client = Pexels::new("invalid_key".to_string());
    let result = client
        .search_photos(SearchBuilder::new().query("test").page(1))
        .await;

    assert!(result.is_err());
    if let Err(PexelsError::ApiError(msg)) = result {
        assert!(
            msg.contains("401") || msg.contains("Unauthorized"),
            "Invalid API key should return error"
        );
    }

    std::env::remove_var("PEXELS_API_KEY");
}

#[tokio::test]
#[ignore]
async fn test_collection_operations() {
    let client = get_test_client().expect("PEXELS_API_KEY not set");

    let collections = client.search_collections(5, 1).await;
    match collections {
        Ok(cols) => {
            println!("Found {} collections", cols.collections.len());
            for collection in &cols.collections {
                println!("  - {}: {}", collection.id, collection.title);
            }
        }
        Err(e) => println!("Failed to get collections: {:?}", e),
    }
}
