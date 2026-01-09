mod client;
mod error;
mod models;

pub use client::Pixabay;
pub use client::SearchImageParams;
pub use client::SearchVideoParams;
pub use error::{PixabayError, Result};
pub use models::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = Pixabay::new("test_key".to_string());
        assert_eq!(client.api_key, "test_key");
    }
}
