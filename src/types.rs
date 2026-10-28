use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StorageCapabilities {
    pub batch: bool,
    pub streaming: bool,
    pub ttl: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetOptions {
    pub ttl: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub deletes: u64,
}

pub trait StorageAdapter: Send + Sync {
    fn capabilities(&self) -> StorageCapabilities;
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, KVDockError>;
    fn set(&self, key: &str, value: &[u8], opt: SetOptions) -> Result<(), KVDockError>;
    fn delete(&self, key: &str) -> Result<bool, KVDockError>;
    fn has(&self, key: &str) -> Result<bool, KVDockError>;
    fn clear(&self) -> Result<(), KVDockError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KVDockError {
    EmptyKey,
    InvalidTTL,
    Adapter(String),
}

impl std::fmt::Display for KVDockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyKey => write!(f, "kvdock: key cannot be empty"),
            Self::InvalidTTL => write!(f, "kvdock: invalid ttl"),
            Self::Adapter(msg) => write!(f, "kvdock: adapter error: {msg}"),
        }
    }
}

impl std::error::Error for KVDockError {}
