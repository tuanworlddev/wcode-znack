//! Shared application state: DB connection, HTTP client, and per-API rate limiters.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Simple interval-based rate limiter. Ensures a minimum delay between the
/// starts of consecutive requests to one WB API group.
pub struct RateLimiter {
    last: tokio::sync::Mutex<Option<Instant>>,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(min_interval_ms: u64) -> Self {
        Self {
            last: tokio::sync::Mutex::new(None),
            min_interval: Duration::from_millis(min_interval_ms),
        }
    }

    /// Await until it is allowed to issue the next request.
    pub async fn acquire(&self) {
        let mut guard = self.last.lock().await;
        if let Some(prev) = *guard {
            let elapsed = prev.elapsed();
            if elapsed < self.min_interval {
                tokio::time::sleep(self.min_interval - elapsed).await;
            }
        }
        *guard = Some(Instant::now());
    }
}

pub struct AppState {
    /// Active store's database. Swapped when the user switches stores.
    pub db: Mutex<rusqlite::Connection>,
    /// Active store id (None until a store is created/selected).
    pub active_store: Mutex<Option<String>>,
    /// App data dir, for opening per-store DB files and stores.json.
    pub app_data_dir: PathBuf,
    pub http: reqwest::Client,
    /// Content API: <= 100 req/min -> >= 600 ms between requests.
    pub content_rl: RateLimiter,
    /// Marketplace API: <= 300 req/min -> >= 200 ms between requests.
    pub marketplace_rl: RateLimiter,
    /// Честный ЗНАК token cache (True API + СУЗ), invalidated on store switch.
    pub znack_tokens: crate::znack::auth::TokenCache,
}

impl AppState {
    pub fn new(conn: rusqlite::Connection, active: Option<String>, app_data_dir: PathBuf) -> Self {
        Self {
            db: Mutex::new(conn),
            active_store: Mutex::new(active),
            app_data_dir,
            http: reqwest::Client::builder()
                .user_agent("wcode-wb-label-printer/0.1")
                .build()
                .expect("failed to build reqwest client"),
            content_rl: RateLimiter::new(650),
            marketplace_rl: RateLimiter::new(220),
            znack_tokens: crate::znack::auth::TokenCache::default(),
        }
    }
}
