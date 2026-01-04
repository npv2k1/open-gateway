//! API Key Pool module
//!
//! This module manages API key pools and provides different selection strategies:
//! - Round Robin: Cycles through keys in order
//! - Random: Selects a random key
//! - Weight: Selects keys based on configured weights

use crate::config::{ApiKeyConfig, ApiKeyPool, ApiKeyStrategy};
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// API Key selector that manages a pool of API keys
#[derive(Debug)]
pub struct ApiKeySelector {
    /// List of enabled API keys
    keys: Vec<ApiKeyConfig>,
    /// Selection strategy
    strategy: ApiKeyStrategy,
    /// Header name for the API key
    pub header_name: String,
    /// Query parameter name for the API key (optional)
    pub query_param_name: Option<String>,
    /// Current index for round-robin selection
    round_robin_index: AtomicUsize,
    /// Total weight for weighted selection
    total_weight: u32,
}

impl ApiKeySelector {
    /// Create a new API key selector from a pool configuration
    pub fn new(pool: &ApiKeyPool) -> Self {
        let keys: Vec<ApiKeyConfig> = pool.keys.iter().filter(|k| k.enabled).cloned().collect();
        let total_weight: u32 = keys.iter().map(|k| k.weight).sum();

        Self {
            keys,
            strategy: pool.strategy.clone(),
            header_name: pool.header_name.clone(),
            query_param_name: pool.query_param_name.clone(),
            round_robin_index: AtomicUsize::new(0),
            total_weight,
        }
    }

    /// Get the next API key based on the configured strategy
    pub fn get_key(&self) -> Option<&str> {
        self.get_key_with_name().map(|(key, _)| key)
    }

    /// Get the next API key with its optional name based on the configured strategy
    /// Returns a tuple of (key, optional_name)
    pub fn get_key_with_name(&self) -> Option<(&str, Option<&str>)> {
        if self.keys.is_empty() {
            return None;
        }

        match self.strategy {
            ApiKeyStrategy::RoundRobin => self.get_round_robin_with_name(),
            ApiKeyStrategy::Random => self.get_random_with_name(),
            ApiKeyStrategy::Weight => self.get_weighted_with_name(),
        }
    }

    /// Round-robin selection with name
    fn get_round_robin_with_name(&self) -> Option<(&str, Option<&str>)> {
        let index = self.round_robin_index.fetch_add(1, Ordering::SeqCst) % self.keys.len();
        let key_config = &self.keys[index];
        Some((&key_config.key, key_config.name.as_deref()))
    }

    /// Random selection with name
    fn get_random_with_name(&self) -> Option<(&str, Option<&str>)> {
        let index = rand::thread_rng().gen_range(0..self.keys.len());
        let key_config = &self.keys[index];
        Some((&key_config.key, key_config.name.as_deref()))
    }

    /// Weighted selection with name
    /// Returns a tuple of (key, optional_name)
    /// If total_weight is 0, falls back to random selection
    fn get_weighted_with_name(&self) -> Option<(&str, Option<&str>)> {
        if self.total_weight == 0 {
            return self.get_random_with_name();
        }

        let mut rng = rand::thread_rng();
        let random_weight = rng.gen_range(0..self.total_weight);
        let mut cumulative_weight = 0u32;

        for key in &self.keys {
            cumulative_weight += key.weight;
            if random_weight < cumulative_weight {
                return Some((&key.key, key.name.as_deref()));
            }
        }

        // Fallback to last key (should not happen)
        self.keys
            .last()
            .map(|k| (k.key.as_str(), k.name.as_deref()))
    }

    /// Get the number of keys in the pool
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Check if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Get the strategy name
    pub fn strategy_name(&self) -> &'static str {
        match self.strategy {
            ApiKeyStrategy::RoundRobin => "round_robin",
            ApiKeyStrategy::Random => "random",
            ApiKeyStrategy::Weight => "weight",
        }
    }
}

/// Thread-safe wrapper for ApiKeySelector
pub type SharedApiKeySelector = Arc<ApiKeySelector>;

/// Create a shared API key selector
pub fn create_selector(pool: &ApiKeyPool) -> SharedApiKeySelector {
    Arc::new(ApiKeySelector::new(pool))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pool(strategy: ApiKeyStrategy) -> ApiKeyPool {
        ApiKeyPool {
            keys: vec![
                ApiKeyConfig {
                    key: "key1".to_string(),
                    name: None,
                    weight: 1,
                    enabled: true,
                },
                ApiKeyConfig {
                    key: "key2".to_string(),
                    name: None,
                    weight: 2,
                    enabled: true,
                },
                ApiKeyConfig {
                    key: "key3".to_string(),
                    name: None,
                    weight: 1,
                    enabled: false, // disabled
                },
            ],
            strategy,
            header_name: "X-API-Key".to_string(),
            query_param_name: None,
        }
    }

    #[test]
    fn test_round_robin() {
        let pool = create_test_pool(ApiKeyStrategy::RoundRobin);
        let selector = ApiKeySelector::new(&pool);

        // Only 2 enabled keys
        assert_eq!(selector.len(), 2);

        // Should cycle through keys
        assert_eq!(selector.get_key(), Some("key1"));
        assert_eq!(selector.get_key(), Some("key2"));
        assert_eq!(selector.get_key(), Some("key1"));
        assert_eq!(selector.get_key(), Some("key2"));
    }

    #[test]
    fn test_random() {
        let pool = create_test_pool(ApiKeyStrategy::Random);
        let selector = ApiKeySelector::new(&pool);

        // Should return one of the enabled keys
        for _ in 0..10 {
            let key = selector.get_key().unwrap();
            assert!(key == "key1" || key == "key2");
        }
    }

    #[test]
    fn test_weighted() {
        let pool = create_test_pool(ApiKeyStrategy::Weight);
        let selector = ApiKeySelector::new(&pool);

        // Run multiple times and check distribution
        let mut key1_count = 0;
        let mut key2_count = 0;
        let iterations = 1000;

        for _ in 0..iterations {
            let key = selector.get_key().unwrap();
            if key == "key1" {
                key1_count += 1;
            } else {
                key2_count += 1;
            }
        }

        // key2 has weight 2, key1 has weight 1
        // key2 should be selected roughly 2x more often
        let ratio = key2_count as f64 / key1_count as f64;
        assert!(ratio > 1.5 && ratio < 2.5, "Weighted ratio: {}", ratio);
    }

    #[test]
    fn test_empty_pool() {
        let pool = ApiKeyPool {
            keys: vec![],
            strategy: ApiKeyStrategy::RoundRobin,
            header_name: "X-API-Key".to_string(),
            query_param_name: None,
        };
        let selector = ApiKeySelector::new(&pool);

        assert!(selector.is_empty());
        assert_eq!(selector.get_key(), None);
    }

    #[test]
    fn test_get_key_with_name() {
        let pool = ApiKeyPool {
            keys: vec![
                ApiKeyConfig {
                    key: "secret-key-1".to_string(),
                    name: Some("production-key".to_string()),
                    weight: 1,
                    enabled: true,
                },
                ApiKeyConfig {
                    key: "secret-key-2".to_string(),
                    name: Some("staging-key".to_string()),
                    weight: 1,
                    enabled: true,
                },
                ApiKeyConfig {
                    key: "secret-key-3".to_string(),
                    name: None,
                    weight: 1,
                    enabled: true,
                },
            ],
            strategy: ApiKeyStrategy::RoundRobin,
            header_name: "X-API-Key".to_string(),
            query_param_name: None,
        };
        let selector = ApiKeySelector::new(&pool);

        // First key should have name
        let (key1, name1) = selector.get_key_with_name().unwrap();
        assert_eq!(key1, "secret-key-1");
        assert_eq!(name1, Some("production-key"));

        // Second key should have name
        let (key2, name2) = selector.get_key_with_name().unwrap();
        assert_eq!(key2, "secret-key-2");
        assert_eq!(name2, Some("staging-key"));

        // Third key should have no name
        let (key3, name3) = selector.get_key_with_name().unwrap();
        assert_eq!(key3, "secret-key-3");
        assert_eq!(name3, None);

        // Should cycle back to first key
        let (key4, name4) = selector.get_key_with_name().unwrap();
        assert_eq!(key4, "secret-key-1");
        assert_eq!(name4, Some("production-key"));
    }
}
