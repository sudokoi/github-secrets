use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Rate limiter for GitHub API requests.
///
/// Implements a simple token bucket algorithm to respect GitHub's rate limits.
/// For authenticated requests, GitHub allows 5000 requests per hour.
pub struct RateLimiter {
    /// Maximum requests per hour.
    max_requests_per_hour: u32,
    /// Time window for rate limiting (1 hour).
    window: Duration,
    /// Timestamps of recent requests.
    request_times: Vec<Instant>,
    /// Maximum concurrent requests.
    max_concurrent: usize,
    /// Current number of in-flight requests.
    current_concurrent: usize,
}

impl RateLimiter {
    /// Create a new rate limiter with default GitHub API limits.
    pub fn new() -> Self {
        Self {
            max_requests_per_hour: crate::constants::rate_limit::REQUESTS_PER_HOUR,
            window: Duration::from_secs(3600),
            request_times: Vec::new(),
            max_concurrent: crate::constants::rate_limit::MAX_CONCURRENT_REQUESTS,
            current_concurrent: 0,
        }
    }

    /// Create a new rate limiter with custom limits (useful for testing).
    pub fn with_limits(
        max_requests_per_hour: u32,
        max_concurrent: usize,
        window_secs: u64,
    ) -> Self {
        Self {
            max_requests_per_hour,
            window: Duration::from_secs(window_secs),
            request_times: Vec::new(),
            max_concurrent,
            current_concurrent: 0,
        }
    }

    /// Wait if necessary to respect rate limits before making a request.
    ///
    /// This method:
    /// 1. Cleans up old request timestamps outside the time window
    /// 2. Waits if we're at the concurrent request limit
    /// 3. Waits if we've exceeded the hourly request limit
    pub async fn wait_if_needed(&mut self) {
        let now = Instant::now();

        // Clean up old request timestamps (outside the 1-hour window)
        self.request_times
            .retain(|&time| now.duration_since(time) < self.window);

        // Wait if we're at the concurrent request limit
        loop {
            if self.current_concurrent < self.max_concurrent {
                break;
            }
            sleep(Duration::from_millis(
                crate::constants::rate_limit::BATCH_DELAY_MS,
            ))
            .await;
        }

        // Wait if we've exceeded the hourly request limit
        if self.request_times.len() >= self.max_requests_per_hour as usize {
            // Calculate how long to wait until the oldest request expires
            if let Some(oldest) = self.request_times.first() {
                let elapsed = now.duration_since(*oldest);
                if elapsed < self.window {
                    let wait_time = self.window - elapsed;
                    sleep(wait_time).await;
                    // Clean up again after waiting
                    let now = Instant::now();
                    self.request_times
                        .retain(|&time| now.duration_since(time) < self.window);
                }
            }
        }

        // Record this request
        self.request_times.push(Instant::now());
        self.current_concurrent += 1;
    }

    /// Mark a request as completed, allowing another concurrent request.
    pub fn release(&mut self) {
        if self.current_concurrent > 0 {
            self.current_concurrent -= 1;
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new();
        assert_eq!(limiter.max_requests_per_hour, 5000);
        assert_eq!(limiter.max_concurrent, 5);
    }

    #[tokio::test]
    async fn test_rate_limiter_wait() {
        let mut limiter = RateLimiter::new();
        // First request should not wait (should be much faster than the 1-hour window)
        let start = Instant::now();
        limiter.wait_if_needed().await;
        limiter.release();
        let elapsed = start.elapsed();
        // Should be very fast (less than 1 second, even on slow CI runners)
        // The actual wait would be 1 hour if we hit the rate limit, so this verifies we don't wait
        assert!(elapsed < Duration::from_secs(1));
    }
}
