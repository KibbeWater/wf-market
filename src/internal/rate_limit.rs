//! Rate limiting utilities.

use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Default rate limit: 3 requests per second (as per WFM API docs).
pub const DEFAULT_RATE_LIMIT: u32 = 3;

/// Type alias for our rate limiter.
pub type ApiRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Build a rate limiter with the specified requests per second.
pub fn build_rate_limiter(requests_per_second: u32) -> Arc<ApiRateLimiter> {
    let quota = Quota::per_second(
        NonZeroU32::new(requests_per_second)
            .unwrap_or(NonZeroU32::new(DEFAULT_RATE_LIMIT).unwrap()),
    );

    Arc::new(RateLimiter::direct(quota))
}

/// Build a rate limiter with default settings.
pub fn build_default_rate_limiter() -> Arc<ApiRateLimiter> {
    build_rate_limiter(DEFAULT_RATE_LIMIT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_rate_limiter() {
        let limiter = build_rate_limiter(5);
        assert!(Arc::strong_count(&limiter) == 1);
    }

    #[test]
    fn test_default_rate_limiter() {
        let limiter = build_default_rate_limiter();
        assert!(Arc::strong_count(&limiter) == 1);
    }
}
