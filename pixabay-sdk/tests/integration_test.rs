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
    assert!(response.hits.len() <= 5);
    println!(
        "Found {} total images, showing {}",
        response.total_hits,
        response.hits.len()
    );
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
    for image in &response.hits {
        assert!(image.image_width >= 1920);
        assert!(image.image_height >= 1080);
        assert_eq!(image.image_type, "photo");
    }
    println!("All {} images meet filter criteria", response.hits.len());
}

#[tokio::test]
async fn test_get_image_by_id() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let search_result = client.search_images("cat", Some(1), Some(1)).await;
    assert!(search_result.is_ok());

    let images = search_result.unwrap();
    if let Some(first_image) = images.hits.first() {
        let result = client.get_image(first_image.id).await;
        assert!(result.is_ok());

        let image = result.unwrap();
        assert_eq!(image.id, first_image.id);
        println!("Retrieved image: ID={}, Tags={}", image.id, image.tags);
    }
}

#[tokio::test]
async fn test_simple_video_search() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let result = client.search_videos("ocean", Some(5), Some(1)).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.total > 0);
    assert!(response.total_hits > 0);
    assert!(!response.hits.is_empty());
    assert!(response.hits.len() <= 5);
    println!(
        "Found {} total videos, showing {}",
        response.total_hits,
        response.hits.len()
    );
}

#[tokio::test]
#[ignore]
async fn test_advanced_video_search() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let params = SearchVideoParams::new()
        .query("mountain")
        .per_page(5)
        .video_type(VideoType::Film)
        .category(Category::Travel)
        .min_width(1920)
        .safesearch(true)
        .order(Order::Latest);

    let result = client.search_videos_advanced(params).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    for video in &response.hits {
        if let Some(large_video) = &video.videos.large {
            assert!(large_video.width >= 1920);
        }
        assert_eq!(video.video_type, "film");
    }
    println!("All {} videos meet filter criteria", response.hits.len());
}

#[tokio::test]
async fn test_get_video_by_id() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let search_result = client.search_videos("dog", Some(1), Some(1)).await;
    assert!(search_result.is_ok());

    let videos = search_result.unwrap();
    if let Some(first_video) = videos.hits.first() {
        let result = client.get_video(first_video.id).await;
        assert!(result.is_ok());

        let video = result.unwrap();
        assert_eq!(video.id, first_video.id);
        println!(
            "Retrieved video: ID={}, Duration={}s, Views={}",
            video.id, video.duration, video.views
        );
    }
}

#[tokio::test]
async fn test_image_pagination() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let page1_result = client.search_images("landscape", Some(3), Some(1)).await;
    assert!(page1_result.is_ok());
    let page1 = page1_result.unwrap();
    let page2_result = client.search_images("landscape", Some(3), Some(2)).await;
    assert!(page2_result.is_ok());
    let page2 = page2_result.unwrap();

    assert!(!page1.hits.is_empty());
    assert!(!page2.hits.is_empty());

    if let (Some(img1), Some(img2)) = (page1.hits.first(), page2.hits.first()) {
        assert_ne!(
            img1.id, img2.id,
            "Different pages should return different images"
        );
    }

    println!(
        "Page 1: {} images, Page 2: {} images",
        page1.hits.len(),
        page2.hits.len()
    );
}

#[tokio::test]
#[ignore]
async fn test_video_pagination() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let page1_result = client.search_videos("city", Some(3), Some(1)).await;
    assert!(page1_result.is_ok());
    let page1 = page1_result.unwrap();
    let page2_result = client.search_videos("city", Some(3), Some(2)).await;
    assert!(page2_result.is_ok());
    let page2 = page2_result.unwrap();

    assert!(!page1.hits.is_empty());
    assert!(!page2.hits.is_empty());

    if let (Some(vid1), Some(vid2)) = (page1.hits.first(), page2.hits.first()) {
        assert_ne!(
            vid1.id, vid2.id,
            "Different pages should return different videos"
        );
    }

    println!(
        "Page 1: {} videos, Page 2: {} videos",
        page1.hits.len(),
        page2.hits.len()
    );
}

#[tokio::test]
#[ignore]
async fn test_image_search_with_different_categories() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let categories = [Category::Nature, Category::Travel, Category::Business];

    for category in &categories {
        let params = SearchImageParams::new()
            .query("background")
            .per_page(3)
            .category(category.clone());

        let result = client.search_images_advanced(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        println!("Category {:?}: {} results", *category, response.total_hits);
        assert!(!response.hits.is_empty());
    }
}

#[tokio::test]
#[ignore]
async fn test_image_search_with_different_orders() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let orders = [Order::Popular, Order::Latest];

    for order in &orders {
        let params = SearchImageParams::new()
            .query("sunset")
            .per_page(3)
            .order(order.clone());

        let result = client.search_images_advanced(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        println!("Order {:?}: {} results", *order, response.total_hits);
        assert!(!response.hits.is_empty());
    }
}

#[tokio::test]
async fn test_video_search_with_different_video_types() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let video_types = [VideoType::Film, VideoType::Animation];

    for vtype in &video_types {
        let params = SearchVideoParams::new()
            .query("water")
            .per_page(3)
            .video_type(vtype.clone());

        let result = client.search_videos_advanced(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        println!("Video type {:?}: {} results", vtype, response.total_hits);
    }
}

#[tokio::test]
#[ignore]
async fn test_invalid_api_key() {
    dotenv().ok();
    std::env::set_var("PIXABAY_API_KEY", "invalid_key_12345");

    let client = Pixabay::new("invalid_key_12345".to_string());
    let result = client.search_images("test", Some(1), Some(1)).await;

    assert!(result.is_err());
    if let Err(PixabayError::ApiError(msg)) = result {
        println!("Error message: {}", msg);
    }

    std::env::remove_var("PIXABAY_API_KEY");
    println!("Invalid API key correctly returns error");
}

#[tokio::test]
#[ignore]
async fn test_get_nonexistent_image() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let result = client.get_image(999999999).await;
    assert!(result.is_err());
    if let Err(e) = result {
        println!("Non-existent image correctly returns error: {:?}", e);
    }
}

#[tokio::test]
async fn test_get_nonexistent_video() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let result = client.get_video(999999999).await;
    assert!(result.is_err());
    if let Err(e) = result {
        println!("Non-existent video correctly returns error: {:?}", e);
    }
}

#[tokio::test]
async fn test_safesearch_filter() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let params_with_safesearch = SearchImageParams::new()
        .query("nature")
        .per_page(5)
        .safesearch(true);

    let params_without_safesearch = SearchImageParams::new()
        .query("nature")
        .per_page(5)
        .safesearch(false);

    let result_with = client.search_images_advanced(params_with_safesearch).await;
    let result_without = client
        .search_images_advanced(params_without_safesearch)
        .await;

    assert!(result_with.is_ok());
    assert!(result_without.is_ok());

    println!(
        "With safesearch: {} results",
        result_with.unwrap().total_hits
    );
    println!(
        "Without safesearch: {} results",
        result_without.unwrap().total_hits
    );
}

#[tokio::test]
async fn test_color_filter() {
    let client = get_test_client().expect("PIXABAY_API_KEY not set");

    let params = SearchImageParams::new()
        .query("background")
        .per_page(5)
        .colors("blue,grayscale");

    let result = client.search_images_advanced(params).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    println!("Images with blue/grayscale colors: {}", response.total_hits);
}
