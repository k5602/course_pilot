use std::future::Future;
use std::time::Duration;
use anyhow::Result;
use crate::ui::components::toast::toast;

/// Retry configuration for async operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Initial delay before first retry (in milliseconds)
    pub initial_delay_ms: u64,
    /// Backoff factor for exponential delay increase
    pub backoff_factor: f64,
    /// Maximum delay between retries (in milliseconds)
    pub max_delay_ms: u64,
    /// Whether to show toast notifications for retries
    pub show_notifications: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 500,
            backoff_factor: 2.0,
            max_delay_ms: 10000, // 10 seconds
            show_notifications: true,
        }
    }
}

/// Retry an async operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    operation: F,
    operation_name: &str,
    config: RetryConfig,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut delay_ms = config.initial_delay_ms;
    
    loop {
        attempt += 1;
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt >= config.max_attempts {
                    // Final attempt failed, return the error
                    return Err(error);
                }
                
                // Log the error
                log::warn!(
                    "Operation '{}' failed (attempt {}/{}): {}",
                    operation_name,
                    attempt,
                    config.max_attempts,
                    error
                );
                
                // Show notification if enabled
                if config.show_notifications {
                    toast::warning(&format!(
                        "Operation failed, retrying in {} seconds... ({}/{})",
                        delay_ms / 1000,
                        attempt,
                        config.max_attempts
                    ));
                }
                
                // Wait before retrying
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                
                // Increase delay for next attempt (with maximum cap)
                delay_ms = ((delay_ms as f64) * config.backoff_factor) as u64;
                if delay_ms > config.max_delay_ms {
                    delay_ms = config.max_delay_ms;
                }
            }
        }
    }
}

/// Retry hook for UI components
pub fn use_retry() -> impl Fn<(
    impl Fn() -> impl Future<Output = Result<T>> + 'static,
    &'static str,
    Option<RetryConfig>,
), Output = impl Future<Output = Result<T>>> + Clone
where
    T: 'static,
{
    move |operation, operation_name, config| {
        let config = config.unwrap_or_default();
        async move {
            retry_with_backoff(operation, operation_name, config).await
        }
    }
}