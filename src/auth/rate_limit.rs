// Limite les tentatives de /auth/login par IP, pour ralentir le brute force
// sur les mots de passe. Partage entre les handlers via Extension, comme
// OAuthConfig/MediaClient.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const MAX_ATTEMPTS: u32 = 10;
const WINDOW: Duration = Duration::from_secs(60);

struct Bucket {
    count: u32,
    window_start: Instant,
}

#[derive(Clone)]
pub struct LoginRateLimiter {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
}

impl LoginRateLimiter {
    pub fn new() -> Self {
        LoginRateLimiter { buckets: Arc::new(Mutex::new(HashMap::new())) }
    }

    // Vrai si cette IP est encore sous la limite (et compte la tentative).
    pub fn allow(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().unwrap();
        let now = Instant::now();
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| Bucket { count: 0, window_start: now });

        if now.duration_since(bucket.window_start) >= WINDOW {
            bucket.window_start = now;
            bucket.count = 0;
        }

        if bucket.count >= MAX_ATTEMPTS {
            false
        } else {
            bucket.count += 1;
            true
        }
    }
}
