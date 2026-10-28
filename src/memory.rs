use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use crate::types::{KVDockError, SetOptions, StorageAdapter, StorageCapabilities};

struct Entry {
    value: Vec<u8>,
    expires_at: Option<Instant>,
}

pub struct MemoryAdapter {
    max_entries: usize,
    state: Mutex<MemoryState>,
}

struct MemoryState {
    entries: HashMap<String, Entry>,
    order: Vec<String>,
}

impl MemoryAdapter {
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries: if max_entries == 0 { 10_000 } else { max_entries },
            state: Mutex::new(MemoryState {
                entries: HashMap::new(),
                order: Vec::new(),
            }),
        }
    }
}

impl Default for MemoryAdapter {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl StorageAdapter for MemoryAdapter {
    fn capabilities(&self) -> StorageCapabilities {
        StorageCapabilities {
            batch: false,
            streaming: false,
            ttl: true,
        }
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, KVDockError> {
        let mut state = self.state.lock().map_err(|e| KVDockError::Adapter(e.to_string()))?;
        let now = Instant::now();

        let is_expired = match state.entries.get(key) {
            Some(entry) => match entry.expires_at {
                Some(exp) => now >= exp,
                None => false,
            },
            None => return Ok(None),
        };

        if is_expired {
            state.entries.remove(key);
            if let Some(pos) = state.order.iter().position(|k| k == key) {
                state.order.remove(pos);
            }
            return Ok(None);
        }

        let val = state.entries.get(key).unwrap().value.clone();
        if let Some(pos) = state.order.iter().position(|k| k == key) {
            state.order.remove(pos);
        }
        state.order.push(key.to_string());
        Ok(Some(val))
    }

    fn set(&self, key: &str, value: &[u8], opt: SetOptions) -> Result<(), KVDockError> {
        let mut state = self.state.lock().map_err(|e| KVDockError::Adapter(e.to_string()))?;
        let now = Instant::now();
        let expires_at = opt.ttl.map(|d| now + d);

        if !state.entries.contains_key(key) && state.entries.len() >= self.max_entries {
            // evict LRU (index 0)
            if !state.order.is_empty() {
                let oldest = state.order.remove(0);
                state.entries.remove(&oldest);
            }
        }

        state.entries.insert(
            key.to_string(),
            Entry {
                value: value.to_vec(),
                expires_at,
            },
        );

        if let Some(pos) = state.order.iter().position(|k| k == key) {
            state.order.remove(pos);
        }
        state.order.push(key.to_string());

        Ok(())
    }

    fn delete(&self, key: &str) -> Result<bool, KVDockError> {
        let mut state = self.state.lock().map_err(|e| KVDockError::Adapter(e.to_string()))?;
        let removed = state.entries.remove(key).is_some();
        if let Some(pos) = state.order.iter().position(|k| k == key) {
            state.order.remove(pos);
        }
        Ok(removed)
    }

    fn has(&self, key: &str) -> Result<bool, KVDockError> {
        Ok(self.get(key)?.is_some())
    }

    fn clear(&self) -> Result<(), KVDockError> {
        let mut state = self.state.lock().map_err(|e| KVDockError::Adapter(e.to_string()))?;
        state.entries.clear();
        state.order.clear();
        Ok(())
    }
}
