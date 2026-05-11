use dashmap::DashMap;
use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::Arc;

/// The Dark Energy Cache: A high-performance, concurrent state cache
/// designed for ZK-Rollup throughput.
pub struct DarkEnergyCache<K, V> {
    /// Hot data stored in a concurrent hash map for lock-free parallel reads
    hot_storage: Arc<DashMap<K, V>>,
    /// LRU manager for eviction logic
    eviction_manager: Arc<Mutex<LruCache<K, ()>>>,
    /// Capacity of the cache
    capacity: usize,
}

impl<K, V> DarkEnergyCache<K, V> 
where 
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static
{
    pub fn new(capacity: usize) -> Self {
        Self {
            hot_storage: Arc::new(DashMap::with_capacity(capacity)),
            eviction_manager: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(capacity).unwrap()))),
            capacity,
        }
    }

    /// Insert a value into the cache with "Dark Energy" priority
    pub fn put(&self, key: K, value: V) {
        self.hot_storage.insert(key.clone(), value);
        
        let mut lru = self.eviction_manager.lock();
        if lru.put(key.clone(), ()).is_none() {
            // If we exceeded capacity, DashMap might still have the old entry
            if lru.len() > self.capacity {
                if let Some((old_key, _)) = lru.pop_lru() {
                    self.hot_storage.remove(&old_key);
                }
            }
        }
    }

    /// Retrieve a value from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        // Read from DashMap (No lock contention)
        let val = self.hot_storage.get(key).map(|v| v.value().clone());
        
        if val.is_some() {
            // Update LRU position (Minor lock contention)
            let mut lru = self.eviction_manager.lock();
            lru.get(key);
        }
        
        val
    }

    /// Clear the entire cache state
    pub fn purge(&self) {
        self.hot_storage.clear();
        self.eviction_manager.lock().clear();
    }

    pub fn len(&self) -> usize {
        self.hot_storage.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_energy_flow() {
        let cache = DarkEnergyCache::new(2);
        cache.put("key1", "val1");
        cache.put("key2", "val2");
        cache.put("key3", "val3"); // Should evict key1

        assert!(cache.get(&"key1").is_none());
        assert!(cache.get(&"key2").is_some());
        assert!(cache.get(&"key3").is_some());
    }
}
