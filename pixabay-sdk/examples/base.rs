use dotenvy::dotenv;
use pixabay_sdk::{
    Category, ImageType, Order, Orientation, Pixabay, SearchImageParams, SearchVideoParams,
    VideoType,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("PIXABAY_API_KEY")
        .expect("PIXABAY_API_KEY must be set in .env file");

    let client = Pixabay::new(api_key);

    println!("=== Simple Image Search ===");
    let images = client.search_images("yellow flowers", Some(5), Some(1)).await?;
    println!("Found {} total images (showing {})", images.total, images.total_hits);
    for image in &images.hits {
        println!("- ID: {}, Tags: {}, Size: {}x{}",
                 image.id, image.tags, image.image_width, image.image_height);
        println!("  URL: {}", image.page_url);
    }

    println!("\n=== Advanced Image Search ===");
    let params = SearchImageParams::new()
        .query("mountains")
        .per_page(5)
        .image_type(ImageType::Photo)
        .orientation(Orientation::Horizontal)
        .category(Category::Nature)
        .min_width(1920)
        .min_height(1080)
        .safesearch(true)
        .order(Order::Latest);

    let images = client.search_images_advanced(params).await?;
    println!("Found {} images with filters (max 500 shown)", images.total_hits);
    for image in &images.hits {
        println!("- {} ({}x{})", image.tags, image.image_width, image.image_height);
        println!("  Views: {}, Downloads: {}, Likes: {}",
                 image.views, image.downloads, image.likes);
    }

    println!("\n=== Get Specific Image by ID ===");
    if let Some(first_image) = images.hits.first() {
        let image = client.get_image(first_image.id).await?;
        println!("Image details:");
        println!("  ID: {}", image.id);
        println!("  Tags: {}", image.tags);
        println!("  Dimensions: {}x{}", image.image_width, image.image_height);
        println!("  File size: {} bytes", image.image_size);
        println!("  User: {} (ID: {})", image.user, image.user_id);
        println!("  Statistics:");
        println!("    - Views: {}", image.views);
        println!("    - Downloads: {}", image.downloads);
        println!("    - Likes: {}", image.likes);
        println!("    - Comments: {}", image.comments);
        if let Some(collections) = image.collections {
            println!("    - Collections: {}", collections);
        }
        println!("  URLs:");
        println!("    - Preview: {}", image.preview_url);
        println!("    - Webformat: {}", image.webformat_url);
        println!("    - Large: {}", image.large_image_url);

        // Full API access fields (maybe None without approval)
        if let Some(full_hd) = &image.full_hd_url {
            println!("    - Full HD: {}", full_hd);
        }
        if let Some(image_url) = &image.image_url {
            println!("    - Original: {}", image_url);
        }
    }

    println!("\n=== Simple Video Search ===");
    let videos = client.search_videos("ocean", Some(3), Some(1)).await?;
    println!("Found {} total videos", videos.total_hits);
    for video in &videos.hits {
        println!("- ID: {}, Tags: {}, Duration: {}s",
                 video.id, video.tags, video.duration);
        println!("  URL: {}", video.page_url);
    }

    println!("\n=== Advanced Video Search ===");
    let video_params = SearchVideoParams::new()
        .query("sunset")
        .per_page(3)
        .video_type(VideoType::Film)
        .category(Category::Nature)
        .min_width(1920)
        .order(Order::Latest);

    let videos = client.search_videos_advanced(video_params).await?;
    println!("Found {} videos with filters", videos.total_hits);
    for video in &videos.hits {
        println!("- {} ({}s)", video.tags, video.duration);
        println!("  User: {}", video.user);
        println!("  Available resolutions:");
        if let Some(large) = &video.videos.large {
            println!("    - Large: {}x{} ({} MB)",
                     large.width, large.height, large.size / 1_000_000);
        }
        if let Some(medium) = &video.videos.medium {
            println!("    - Medium: {}x{} ({} MB)",
                     medium.width, medium.height, medium.size / 1_000_000);
        }
        if let Some(small) = &video.videos.small {
            println!("    - Small: {}x{} ({} MB)",
                     small.width, small.height, small.size / 1_000_000);
        }
        if let Some(tiny) = &video.videos.tiny {
            println!("    - Tiny: {}x{} ({} MB)",
                     tiny.width, tiny.height, tiny.size / 1_000_000);
        }
    }

    println!("\n=== Get Specific Video by ID ===");
    if let Some(first_video) = videos.hits.first() {
        let video = client.get_video(first_video.id).await?;
        println!("Video details:");
        println!("  ID: {}", video.id);
        println!("  Tags: {}", video.tags);
        println!("  Type: {}", video.video_type);
        println!("  Duration: {}s", video.duration);
        println!("  User: {} (ID: {})", video.user, video.user_id);
        println!("  Statistics:");
        println!("    - Views: {}", video.views);
        println!("    - Downloads: {}", video.downloads);
        println!("    - Likes: {}", video.likes);
        println!("    - Comments: {}", video.comments);
    }

    println!("\n✅ All API calls completed successfully!");
    println!("\n📝 Note: Remember to:");
    println!("  1. Cache API responses for 24 hours");
    println!("  2. Show Pixabay as the source when displaying results");
    println!("  3. Download images to your server (don't hotlink)");
    println!("  4. Respect the 100 requests/60 seconds rate limit");

    Ok(())
}