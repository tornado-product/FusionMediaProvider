use crate::error::{MediaError, Result};
use crate::media_provider::MediaProvider;
use std::sync::Arc;

#[cfg(feature = "pexels")]
use crate::PexelsProvider;
use crate::PixabayProvider;

/// 根据 provider 名称创建对应的 MediaProvider 实例
pub fn create_provider(
    provider_name: &str,
    api_key: &str,
) -> Result<Arc<dyn MediaProvider + Send + Sync>> {
    if api_key.is_empty() {
        return Err(MediaError::ApiKeyIsEmpty);
    }
    match provider_name.to_lowercase().as_str() {
        "pexels" => {
            #[cfg(feature = "pexels")]
            {
                let provider = PexelsProvider::new(api_key.to_string());
                Ok(Arc::new(provider))
            }
            #[cfg(not(feature = "pexels"))]
            Err(MediaError::ProviderNotEnabled(
                "Pexels feature is not enabled".to_string(),
            ))
        }
        "pixabay" => {
            let provider = PixabayProvider::new(api_key.to_string());
            Ok(Arc::new(provider))
        }
        _ => Err(MediaError::UnknownProvider(provider_name.to_string())),
    }
}
