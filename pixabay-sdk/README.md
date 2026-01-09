# Pixabay Rust Client

A Rust wrapper for the Pixabay API with command-line interface.

This project consists of two main components:
- **pixabay-sdk**: A Rust library for interacting with the Pixabay API
- **pixabay-sdk-cli**: A command-line interface for using the pixabay-sdk

## Features

### pixabay-sdk

- Search for images with advanced filtering
- Search for videos with advanced filtering
- Retrieve individual images and videos by ID
- Support for various filters:
    - Image types (photo, illustration, vector)
    - Video types (film, animation)
    - Orientation (horizontal, vertical)
    - Categories (nature, animals, people, etc.)
    - Minimum dimensions
    - Colors
    - Editor's choice
    - Safe search
    - Multiple languages
- Asynchronous operations using tokio
- Type-safe API with proper error handling

### pixabay-sdk-cli

- Command-line interface for searching images and videos
- Retrieve individual media items by their ID
- Support for all API parameters
- JSON output for easy integration

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
pixabay-sdk = "0.1.0"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
dotenvy = "0.15"
```

## Setup

1. Clone the repository:

```bash
git clone https://github.com/yourusername/pixabay.git
cd pixabay
```

2. Build the project:

```bash
cargo build --release
```

3. Set up your API key:

Create a `.env` file in the project root:

```env
PIXABAY_API_KEY=your_api_key_here
```

You can get your API key from [Pixabay API Documentation](https://pixabay.com/api/docs/).

## Usage

### Library Usage

Here's a basic example of using the `pixabay-sdk` library:

```rust
use dotenvy::dotenv;
use pixabay_api::{Pixabay, SearchImageParams, ImageType, Order, Category};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("PIXABAY_API_KEY")?;
    let client = Pixabay::new(api_key);

    // Simple image search
    let images = client.search_images("nature", Some(10), Some(1)).await?;
    for image in images.hits {
        println!("Image: {} - {}", image.id, image.tags);
    }

    // Advanced image search with parameters
    let params = SearchImageParams::new()
        .query("mountains")
        .per_page(20)
        .image_type(ImageType::Photo)
        .orientation(Orientation::Horizontal)
        .category(Category::Nature)
        .min_width(1920)
        .min_height(1080)
        .editors_choice(true)
        .safesearch(true)
        .order(Order::Latest);

    let images = client.search_images_advanced(params).await?;
    println!("Found {} images", images.total_hits);

    // Get a specific image by ID
    let image = client.get_image(736885).await?;
    println!("Image URL: {}", image.large_image_url);

    // Search for videos
    let videos = client.search_videos("ocean", Some(10), Some(1)).await?;
    for video in videos.hits {
        println!("Video: {} - {}", video.id, video.tags);
    }

    // Get a specific video by ID
    let video = client.get_video(31377).await?;
    println!("Video duration: {}s", video.duration);

    Ok(())
}
```

### CLI Usage

#### Search for Images

```bash
# Basic search
cargo run --bin pixabay -- search-images --query "nature" --per-page 10

# Advanced search with filters
cargo run --bin pixabay -- search-images \
    --query "mountains" \
    --per-page 20 \
    --image-type photo \
    --orientation horizontal \
    --category nature \
    --min-width 1920 \
    --min-height 1080 \
    --order latest \
    --editors-choice \
    --safesearch
```

#### Get Image by ID

```bash
cargo run --bin pixabay -- get-image --id 736885
```

#### Search for Videos

```bash
# Basic search
cargo run --bin pixabay -- search-videos --query "ocean" --per-page 10

# Advanced search with filters
cargo run --bin pixabay -- search-videos \
    --query "sunset" \
    --per-page 20 \
    --video-type film \
    --category nature \
    --min-width 1920 \
    --min-height 1080 \
    --order latest \
    --editors-choice
```

#### Get Video by ID

```bash
cargo run --bin pixabay -- get-video --id 31377
```

## API Reference

### Client Methods

- `search_images(query, per_page, page)` - Simple image search
- `search_images_advanced(params)` - Advanced image search with parameters
- `get_image(id)` - Get a specific image by ID
- `search_videos(query, per_page, page)` - Simple video search
- `search_videos_advanced(params)` - Advanced video search with parameters
- `get_video(id)` - Get a specific video by ID

### Types

- `ImageType`: All, Photo, Illustration, Vector
- `VideoType`: All, Film, Animation
- `Orientation`: All, Horizontal, Vertical
- `Order`: Popular, Latest
- `Category`: Backgrounds, Fashion, Nature, Science, Education, Feelings, Health, People, Religion, Places, Animals, Industry, Computer, Food, Sports, Transportation, Travel, Buildings, Business, Music
- `Language`: Multiple language codes supported (En, De, Fr, Es, etc.)

## Error Handling

The library provides comprehensive error handling:

```rust
use pixabay_api::{Pixabay, PixabayError};

match client.search_images("test", Some(10), Some(1)).await {
    Ok(response) => println!("Success: {} results", response.total_hits),
    Err(PixabayError::RateLimitExceeded) => eprintln!("Rate limit exceeded"),
    Err(PixabayError::InvalidApiKey) => eprintln!("Invalid API key"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Rate Limits

By default, the Pixabay API allows up to 100 requests per 60 seconds. The library will return a `RateLimitExceeded` error when you hit this limit.

## Documentation

For detailed API documentation, visit:
- [Pixabay API Documentation](https://pixabay.com/api/docs/)
- [Library Documentation](https://docs.rs/pixabay-sdk)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 or MIT license, shall be dual licensed as above, without any additional terms or conditions.