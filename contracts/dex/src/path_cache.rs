use ink::prelude::collections::BTreeMap;

/// Default number of cached routes before LRU eviction kicks in.
pub const PATH_CACHE_DEFAULT_CAPACITY: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PathKey {
    pub from_token: u64,
    pub to_token: u64,
}

/// Fixed-capacity LRU cache for multi-hop routing between token pairs.
///
/// Replaces the previous unbounded, linear-scan `Vec`: lookups are
/// BTreeMap-backed (bounded) and insertion over capacity evicts the
/// least-recently-used entry, so adopting the cache for route discovery
/// cannot grow storage without limit.
pub struct PathCache {
    entries: BTreeMap<PathKey, (Vec<u64>, u64)>,
    tick: u64,
    capacity: usize,
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl PathCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            tick: 0,
            capacity,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn hits(&self) -> u64 {
        self.hits
    }

    pub fn misses(&self) -> u64 {
        self.misses
    }

    pub fn evictions(&self) -> u64 {
        self.evictions
    }

    /// Look up a cached route, promoting it to most-recently-used on hit.
    pub fn get(&mut self, key: &PathKey) -> Option<Vec<u64>> {
        let Some((path, _)) = self.entries.get(key).cloned() else {
            self.misses += 1;
            return None;
        };
        self.tick = self.tick.wrapping_add(1);
        self.entries.insert(key.clone(), (path.clone(), self.tick));
        self.hits += 1;
        Some(path)
    }

    /// Insert (or overwrite) a route, promoting it to most-recently-used.
    pub fn insert(&mut self, key: PathKey, path: Vec<u64>) {
        if path.is_empty() {
            return;
        }
        if self.capacity == 0 {
            return;
        }
        if !self.entries.contains_key(&key) && self.len() >= self.capacity {
            self.evict_lru();
        }
        self.tick = self.tick.wrapping_add(1);
        self.entries.insert(key, (path, self.tick));
    }

    /// Return the cached route or compute and cache it on miss. This is the
    /// entry point used by route discovery so warm pairs skip recomputation.
    pub fn get_or_compute(
        &mut self,
        key: &PathKey,
        compute: impl FnOnce() -> Vec<u64>,
    ) -> Option<Vec<u64>> {
        if let Some(path) = self.get(key) {
            return Some(path);
        }
        let path = compute();
        if path.is_empty() {
            return None;
        }
        self.insert(key.clone(), path.clone());
        Some(path)
    }

    fn evict_lru(&mut self) {
        let lru = self
            .entries
            .iter()
            .min_by_key(|(_, (_, last_used))| *last_used)
            .map(|(k, _)| k.clone());
        if let Some(key) = lru {
            self.entries.remove(&key);
            self.evictions += 1;
        }
    }
}

#[cfg(test)]
mod path_cache_tests {
    use super::*;

    #[test]
    fn miss_then_hit() {
        let mut c = PathCache::new(4);
        let k = PathKey { from_token: 1, to_token: 5 };
        assert!(c.get(&k).is_none());
        c.insert(k.clone(), vec![1, 2, 3, 4, 5]);
        assert_eq!(c.get(&k), Some(vec![1, 2, 3, 4, 5]));
        assert_eq!(c.hits(), 1);
        assert_eq!(c.misses(), 1);
    }

    #[test]
    fn insert_overwrites_existing_key() {
        let mut c = PathCache::new(4);
        let k = PathKey { from_token: 1, to_token: 3 };
        c.insert(k.clone(), vec![1, 2, 3]);
        c.insert(k.clone(), vec![1, 3]);
        assert_eq!(c.get(&k), Some(vec![1, 3]));
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn empty_path_is_not_cached() {
        let mut c = PathCache::new(4);
        c.insert(
            PathKey { from_token: 1, to_token: 2 },
            Vec::new(),
        );
        assert_eq!(c.len(), 0);
        assert!(c.get(&PathKey { from_token: 1, to_token: 2 }).is_none());
    }

    #[test]
    fn eviction_removes_least_recently_used_first() {
        let mut c = PathCache::new(2);
        let k1 = PathKey { from_token: 1, to_token: 2 };
        let k2 = PathKey { from_token: 2, to_token: 3 };
        let k3 = PathKey { from_token: 3, to_token: 4 };
        c.insert(k1.clone(), vec![1, 2]);
        c.insert(k2.clone(), vec![2, 3]);
        // Touch k1 so it is now the most recent; k2 becomes LRU.
        let _ = c.get(&k1);
        c.insert(k3.clone(), vec![3, 4]);
        assert_eq!(c.len(), 2);
        assert_eq!(c.get(&k1), Some(vec![1, 2]));
        assert_eq!(c.get(&k3), Some(vec![3, 4]));
        assert!(c.get(&k2).is_none());
        assert_eq!(c.evictions(), 1);
    }

    #[test]
    fn capacity_is_never_exceeded_under_large_load() {
        let mut c = PathCache::new(100);
        // 10k distinct routes: the cache must stay bounded and evict.
        for i in 0..10_000u64 {
            c.insert(
                PathKey { from_token: i, to_token: i + 1 },
                vec![i, i + 1],
            );
        }
        assert!(c.len() <= 100);
        assert_eq!(c.evictions(), 10_000 - 100);
        assert!(c
            .get(&PathKey { from_token: 9_999, to_token: 10_000 })
            .is_some(), "most recent route must survive eviction");
    }

    #[test]
    fn get_or_compute_memoizes_on_hit() {
        let mut c = PathCache::new(4);
        let k = PathKey { from_token: 7, to_token: 9 };
        let mut computes = 0u32;
        let route = c.get_or_compute(&k, || {
            computes += 1;
            vec![7, 8, 9]
        });
        assert_eq!(route, Some(vec![7, 8, 9]));
        let warm = c.get_or_compute(&k, || {
            computes += 1;
            vec![7, 8, 9]
        });
        assert_eq!(warm, Some(vec![7, 8, 9]));
        assert_eq!(computes, 1, "compute must run only on the cold miss");
    }

    #[test]
    fn zero_capacity_is_a_fuse() {
        let mut c = PathCache::new(0);
        c.insert(PathKey { from_token: 1, to_token: 2 }, vec![1, 2]);
        assert_eq!(c.len(), 0);
        assert!(c.get(&PathKey { from_token: 1, to_token: 2 }).is_none());
    }
}