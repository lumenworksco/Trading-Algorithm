//! Data caching.

use std::collections::HashMap;
use std::path::PathBuf;
use trading_core::types::{Bar, Timeframe};


/// Simple in-memory data cache.
pub struct DataCache {
    cache: HashMap<String, Vec<Bar>>,
    cache_dir: PathBuf,
}

impl DataCache {
    /// Create a new data cache.
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache: HashMap::new(),
            cache_dir,
        }
    }

    /// Generate cache key.
    fn cache_key(symbol: &str, timeframe: Timeframe) -> String {
        format!("{}_{}", symbol, timeframe)
    }

    /// Get cached bars.
    pub fn get(&self, symbol: &str, timeframe: Timeframe) -> Option<&Vec<Bar>> {
        let key = Self::cache_key(symbol, timeframe);
        self.cache.get(&key)
    }

    /// Store bars in cache.
    pub fn put(&mut self, symbol: &str, timeframe: Timeframe, bars: Vec<Bar>) {
        let key = Self::cache_key(symbol, timeframe);
        self.cache.insert(key, bars);
    }

    /// Clear cache for a symbol.
    pub fn clear(&mut self, symbol: &str) {
        self.cache.retain(|k, _| !k.starts_with(symbol));
    }

    /// Clear all cached data.
    pub fn clear_all(&mut self) {
        self.cache.clear();
    }

    /// Get cache directory.
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }
}
