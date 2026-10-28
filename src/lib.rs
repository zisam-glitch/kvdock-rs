pub mod memory;
pub mod types;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub use memory::MemoryAdapter;
pub use types::{CacheStats, KVDockError, SetOptions, StorageAdapter, StorageCapabilities};

pub struct KVDock<A: StorageAdapter = MemoryAdapter> {
    adapter: Arc<A>,
    prefix: String,
    default_ttl: Option<Duration>,

    hits: AtomicU64,
    misses: AtomicU64,
    sets: AtomicU64,
    deletes: AtomicU64,
}

pub struct Config<A: StorageAdapter> {
    pub adapter: A,
    pub prefix: String,
    pub default_ttl: Option<Duration>,
}

impl Default for Config<MemoryAdapter> {
    fn default() -> Self {
        Self {
            adapter: MemoryAdapter::default(),
            prefix: String::new(),
            default_ttl: None,
        }
    }
}

impl<A: StorageAdapter> KVDock<A> {
    pub fn new(config: Config<A>) -> Result<Self, KVDockError> {
        Ok(Self {
            adapter: Arc::new(config.adapter),
            prefix: config.prefix,
            default_ttl: config.default_ttl,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            sets: AtomicU64::new(0),
            deletes: AtomicU64::new(0),
        })
    }

    fn resolve_key(&self, key: &str) -> Result<String, KVDockError> {
        if key.is_empty() {
            return Err(KVDockError::EmptyKey);
        }
        if self.prefix.is_empty() {
            Ok(key.to_string())
        } else {
            Ok(format!("{}:{}", self.prefix, key))
        }
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            sets: self.sets.load(Ordering::Relaxed),
            deletes: self.deletes.load(Ordering::Relaxed),
        }
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>, KVDockError> {
        let resolved = self.resolve_key(key)?;
        match self.adapter.get(&resolved)? {
            Some(val) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Ok(Some(val))
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                Ok(None)
            }
        }
    }

    pub fn set(&self, key: &str, value: &[u8], opt: Option<SetOptions>) -> Result<(), KVDockError> {
        let resolved = self.resolve_key(key)?;
        let options = opt.unwrap_or(SetOptions {
            ttl: self.default_ttl,
        });

        self.adapter.set(&resolved, value, options)?;
        self.sets.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn get_or_set<F>(&self, key: &str, opt: Option<SetOptions>, fetcher: F) -> Result<Vec<u8>, KVDockError>
    where
        F: FnOnce() -> Result<Vec<u8>, KVDockError>,
    {
        if let Some(val) = self.get(key)? {
            return Ok(val);
        }

        let fresh = fetcher()?;
        self.set(key, &fresh, opt)?;
        Ok(fresh)
    }

    pub fn delete(&self, key: &str) -> Result<bool, KVDockError> {
        let resolved = self.resolve_key(key)?;
        let deleted = self.adapter.delete(&resolved)?;
        if deleted {
            self.deletes.fetch_add(1, Ordering::Relaxed);
        }
        Ok(deleted)
    }

    pub fn has(&self, key: &str) -> Result<bool, KVDockError> {
        let resolved = self.resolve_key(key)?;
        self.adapter.has(&resolved)
    }

    pub fn clear(&self) -> Result<(), KVDockError> {
        self.adapter.clear()
    }
}
