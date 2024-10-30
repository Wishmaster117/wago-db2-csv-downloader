#[derive(Debug)]
pub struct AppConfig {
    pub base_url: String,
    pub requests_per_minute: u32,
    pub max_retries: u32,
    pub retry_delay_secs: u64,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            base_url: "https://wago.tools/db2".to_string(),
            requests_per_minute: 100,
            max_retries: 3,
            retry_delay_secs: 5,
        }
    }
}