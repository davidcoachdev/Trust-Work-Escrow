use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

static CACHE: OnceLock<Mutex<HashMap<String, (String, Instant)>>> = OnceLock::new();

fn cache() -> &'static Mutex<HashMap<String, (String, Instant)>> {
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_cached(key: &str) -> Option<String> {
    let map = cache().lock().ok()?;
    let (val, at) = map.get(key)?;
    if at.elapsed() < Duration::from_secs(30) {
        Some(val.clone())
    } else {
        None
    }
}

pub fn set_cached(key: String, val: String) {
    if let Ok(mut map) = cache().lock() {
        map.insert(key, (val, Instant::now()));
    }
}
