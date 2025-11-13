use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct CacheEntry<T> {
    value: T,
    inserted_at: Instant,
    ttl: Duration,
}

impl<T: Clone> CacheEntry<T> {
    pub fn new(value: T, ttl: Duration) -> Self {
        CacheEntry {
            value,
            inserted_at: Instant::now(),
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.inserted_at.elapsed() > self.ttl
    }

    pub fn value(&self) -> T {
        self.value.clone()
    }
}

pub struct Cache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    entries: Arc<Mutex<HashMap<K, CacheEntry<V>>>>,
    default_ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(default_ttl: Duration) -> Self {
        Cache {
            entries: Arc::new(Mutex::new(HashMap::new())),
            default_ttl,
        }
    }

    pub fn insert(&self, key: K, value: V) {
        let entry = CacheEntry::new(value, self.default_ttl);

        if let Ok(mut entries) = self.entries.lock() {
            entries.insert(key, entry);
        }
    }

    pub fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let entry = CacheEntry::new(value, ttl);

        if let Ok(mut entries) = self.entries.lock() {
            entries.insert(key, entry);
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        if let Ok(mut entries) = self.entries.lock() {
            if let Some(entry) = entries.get(key) {
                if entry.is_expired() {
                    entries.remove(key);
                    return None;
                }

                return Some(entry.value());
            }
        }

        None
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        if let Ok(mut entries) = self.entries.lock() {
            return entries.remove(key).map(|entry| entry.value());
        }

        None
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }

    pub fn cleanup_expired(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.retain(|_, entry| !entry.is_expired());
        }
    }

    pub fn len(&self) -> usize {
        if let Ok(entries) = self.entries.lock() {
            return entries.len();
        }

        0
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V> Default for Cache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new(Duration::from_secs(300)) // 5 minutes default TTL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_insert_and_get() {
        let cache: Cache<String, i32> = Cache::new(Duration::from_secs(60));

        cache.insert("key1".to_string(), 42);
        cache.insert("key2".to_string(), 100);

        assert_eq!(cache.get(&"key1".to_string()), Some(42));
        assert_eq!(cache.get(&"key2".to_string()), Some(100));
        assert_eq!(cache.get(&"key3".to_string()), None);
    }

    #[test]
    fn test_cache_expiration() {
        let cache: Cache<String, i32> = Cache::new(Duration::from_millis(100));

        cache.insert("key1".to_string(), 42);

        assert_eq!(cache.get(&"key1".to_string()), Some(42));

        thread::sleep(Duration::from_millis(150));

        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_remove() {
        let cache: Cache<String, i32> = Cache::new(Duration::from_secs(60));

        cache.insert("key1".to_string(), 42);
        assert_eq!(cache.get(&"key1".to_string()), Some(42));

        cache.remove(&"key1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache: Cache<String, i32> = Cache::new(Duration::from_secs(60));

        cache.insert("key1".to_string(), 42);
        cache.insert("key2".to_string(), 100);

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert_eq!(cache.get(&"key1".to_string()), None);
    }
}
