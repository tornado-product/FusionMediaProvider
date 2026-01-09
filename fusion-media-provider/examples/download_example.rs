use dotenvy::dotenv;
use fusion_media_provider::{
    BatchDownloadProgress, DownloadConfig, DownloadProgress, DownloadState, ImageQuality,
    MediaDownloader, MediaType, PixabayProvider, SearchParams, VideoQuality,
};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Get API keys from environment
    let pixabay_key = env::var("PIXABAY_API_KEY").expect("PIXABAY_API_KEY must be set");

    // Optional: Pexels support
    let pexels_key = env::var("PEXELS_API_KEY").ok();

    // Create downloader with configuration and progress callback
    let config = DownloadConfig {
        image_quality: ImageQuality::Large,
        video_quality: VideoQuality::Large,
        output_dir: "./downloads".to_string(),
        use_original_names: false,
        max_concurrent: 3,
        progress_callback: Some(Arc::new(|progress: DownloadProgress| {
            match progress.state {
                DownloadState::Starting => {
                    println!("\n🔽 Starting: {}", progress.item_title);
                }
                DownloadState::Downloading => {
                    print!(
                        "\r⬇️  Downloading: {} | {:.1}% | {} | {} | ETA: {}     ",
                        progress.item_title,
                        progress.percentage,
                        DownloadProgress::format_bytes(progress.downloaded_bytes),
                        progress.format_speed(),
                        progress.format_eta()
                    );
                    use std::io::Write;
                    std::io::stdout().flush().unwrap();
                }
                DownloadState::Writing => {
                    print!(
                        "\r💾 Writing to disk: {}...                    ",
                        progress.item_title
                    );
                    use std::io::Write;
                    std::io::stdout().flush().unwrap();
                }
                DownloadState::Completed => {
                    println!(
                        "\r✅ Completed: {} ({:.2}s)                    ",
                        progress.item_title, progress.elapsed_secs
                    );
                }
                DownloadState::Failed(ref err) => {
                    println!(
                        "\r❌ Failed: {} - {}                    ",
                        progress.item_title, err
                    );
                }
            }
        })),
    };

    let mut downloader = MediaDownloader::new()
        .with_config(config)
        .add_provider(Arc::new(PixabayProvider::new(pixabay_key)));

    // Add Pexels if available
    #[cfg(feature = "pexels")]
    if let Some(key) = pexels_key {
        downloader =
            downloader.add_provider(Arc::new(fusion_media_provider::PexelsProvider::new(key)));
    }

    println!("=== Media Downloader Demo ===");
    println!("Active providers: {}", downloader.providers().len());
    for provider in downloader.providers() {
        println!("  - {}", provider.name());
    }

    // Example 1: Search images from all providers
    println!("\n=== Searching for 'nature' images ===");
    let params = SearchParams::new("nature", MediaType::Image).limit(5);

    let results = downloader.search(params).await?;

    println!("📊 Aggregated Results:");
    println!("  Total results available: {}", results.total);
    println!("  Total hits (may be capped): {}", results.total_hits);
    println!(
        "  Total pages across all providers: {}",
        results.total_pages
    );
    println!("  Current page: {}", results.page);
    println!("  Results in this response: {}", results.items.len());

    println!("\n📦 Per-provider breakdown:");
    for provider_result in &results.provider_results {
        println!("  {} Provider:", provider_result.provider);
        println!("    - Total: {}", provider_result.total);
        println!("    - Total hits: {}", provider_result.total_hits);
        println!("    - Total pages: {}", provider_result.total_pages);
        println!("    - Items: {}", provider_result.items.len());
    }

    println!("\n🖼️  Sample results:");
    for (i, item) in results.items.iter().take(3).enumerate() {
        println!(
            "  {}. [{}] {} (ID: {})",
            i + 1,
            item.provider,
            item.title,
            item.id
        );
        println!("     Author: {}", item.author);
        println!(
            "     Size: {}x{}",
            item.metadata.width, item.metadata.height
        );
        println!(
            "     Likes: {}, Downloads: {}",
            item.metadata.likes, item.metadata.downloads
        );
    }

    // Example 2: Search from specific provider
    println!("\n=== Searching Pixabay only ===");
    let pixabay_result = downloader
        .search_from_provider(
            "Pixabay",
            SearchParams::new("mountains", MediaType::Image).limit(3),
        )
        .await?;

    println!("Pixabay Results:");
    println!("  Total: {}", pixabay_result.total);
    println!("  Total pages: {}", pixabay_result.total_pages);
    println!("  Items in this page: {}", pixabay_result.items.len());

    // Example 3: Download images
    if !results.items.is_empty() {
        println!("\n=== Downloading first 3 images ===");
        let to_download = &results.items[..3.min(results.items.len())];

        let download_results = downloader.download_items(to_download).await;

        for (item, result) in to_download.iter().zip(download_results.iter()) {
            match result {
                Ok(path) => println!("✓ Downloaded: {} -> {}", item.title, path),
                Err(e) => eprintln!("✗ Failed to download {}: {}", item.title, e),
            }
        }
    }

    // Example 4: Search for videos with pagination info
    println!("\n=== Searching for 'ocean' videos ===");
    let video_params = SearchParams::new("ocean", MediaType::Video).limit(3);

    let video_results = downloader.search(video_params).await?;

    println!("📊 Video Search Results:");
    println!("  Total videos available: {}", video_results.total);
    println!("  Total pages: {}", video_results.total_pages);
    println!(
        "  Found {} videos in current page",
        video_results.items.len()
    );

    for video in &video_results.items {
        println!("\n{} - {} (ID: {})", video.provider, video.title, video.id);
        println!("  Duration: {}s", video.metadata.duration.unwrap_or(0));

        if let Some(video_files) = &video.urls.video_files {
            println!("  Available qualities:");
            for file in video_files {
                println!(
                    "    - {}: {}x{} ({} MB)",
                    file.quality,
                    file.width,
                    file.height,
                    file.size / 1_000_000
                );
            }
        }
    }

    // Example 5: Download a video
    if !video_results.items.is_empty() {
        println!("\n=== Downloading first video ===");
        match downloader.download_item(&video_results.items[0]).await {
            Ok(path) => println!("✓ Video downloaded: {}", path),
            Err(e) => eprintln!("✗ Failed to download video: {}", e),
        }
    }

    // Example 6: Demonstrate pagination
    println!("\n=== Pagination Example ===");
    let page1 = downloader
        .search(SearchParams::new("cat", MediaType::Image).limit(10).page(1))
        .await?;

    println!("Page 1 of {}:", page1.total_pages);
    println!("  - Total results: {}", page1.total);
    println!("  - Results in this page: {}", page1.items.len());
    println!("  - Providers used: {}", page1.provider_results.len());

    if page1.total_pages > 1 {
        println!("\nYou can fetch more pages like:");
        println!("  SearchParams::new(\"cat\", MediaType::Image).limit(10).page(2)");
        println!("  Available pages: 1 to {}", page1.total_pages);
    }

    // Example 7: Download with batch progress
    println!("\n=== Batch Download with Overall Progress ===");
    if results.items.len() >= 5 {
        let to_download = &results.items[..5];

        let batch_results = downloader
            .download_items_with_batch_progress(
                to_download,
                |batch_progress: BatchDownloadProgress| {
                    println!("\n📊 Batch Progress:");
                    println!("  Overall: {:.1}%", batch_progress.overall_percentage);
                    println!(
                        "  Completed: {}/{}",
                        batch_progress.completed_items, batch_progress.total_items
                    );
                    println!("  Failed: {}", batch_progress.failed_items);
                    println!(
                        "  Currently downloading: {}",
                        batch_progress.downloading_items
                    );
                },
            )
            .await;

        println!("\n\n📋 Batch Download Summary:");
        let successful = batch_results.iter().filter(|r| r.is_ok()).count();
        let failed = batch_results.iter().filter(|r| r.is_err()).count();
        println!("  ✅ Successful: {}", successful);
        println!("  ❌ Failed: {}", failed);
    }

    println!("\n✅ Demo completed!");

    Ok(())
}
