use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug)]
pub struct RateLimiter {
    requests_per_minute: u32,
    last_request: Instant,
    requests_made: u32,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            last_request: Instant::now(),
            requests_made: 0,
        }
    }

    pub async fn wait(&mut self) {
        let minute = Duration::from_secs(60);
        let now = Instant::now();
        
        if now.duration_since(self.last_request) >= minute {
            self.requests_made = 0;
            self.last_request = now;
        } else if self.requests_made >= self.requests_per_minute {
            let wait_time = minute - now.duration_since(self.last_request);
            println!(
                "Wait {} seconds before continuing the download to bypass the blocking...", 
                wait_time.as_secs()
            );
            sleep(wait_time).await;
            self.requests_made = 0;
            self.last_request = Instant::now();
        }

        self.requests_made += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_initial_state() {
        let limiter = RateLimiter::new(60);
        assert_eq!(limiter.requests_per_minute, 60);
        assert_eq!(limiter.requests_made, 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_counting() {
        let mut limiter = RateLimiter::new(60);
        limiter.wait().await;
        assert_eq!(limiter.requests_made, 1);
    }
}