use std::time::Duration;
use kvdock::{Config, KVDock, KVDockError, MemoryAdapter, SetOptions};

#[test]
fn test_basic_set_and_get() {
    let dock = KVDock::new(Config {
        adapter: MemoryAdapter::default(),
        prefix: "test".to_string(),
        default_ttl: None,
    })
    .unwrap();

    dock.set("user:1", b"alice", None).unwrap();
    let val = dock.get("user:1").unwrap();
    assert_eq!(val, Some(b"alice".to_vec()));

    let stats = dock.stats();
    assert_eq!(stats.sets, 1);
    assert_eq!(stats.hits, 1);
}

#[test]
fn test_cache_miss() {
    let dock = KVDock::new(Config::default()).unwrap();
    let val = dock.get("nonexistent").unwrap();
    assert_eq!(val, None);

    let stats = dock.stats();
    assert_eq!(stats.misses, 1);
}

#[test]
fn test_get_or_set() {
    let dock = KVDock::new(Config::default()).unwrap();
    let mut calls = 0;

    let v1 = dock
        .get_or_set("calc", None, || {
            calls += 1;
            Ok(b"42".to_vec())
        })
        .unwrap();
    assert_eq!(v1, b"42".to_vec());
    assert_eq!(calls, 1);

    let v2 = dock
        .get_or_set("calc", None, || {
            calls += 1;
            Ok(b"99".to_vec())
        })
        .unwrap();
    assert_eq!(v2, b"42".to_vec());
    assert_eq!(calls, 1);
}

#[test]
fn test_ttl_expiration() {
    let dock = KVDock::new(Config::default()).unwrap();
    dock.set(
        "temp",
        b"val",
        Some(SetOptions {
            ttl: Some(Duration::from_millis(20)),
        }),
    )
    .unwrap();

    assert_eq!(dock.get("temp").unwrap(), Some(b"val".to_vec()));
    std::thread::sleep(Duration::from_millis(40));
    assert_eq!(dock.get("temp").unwrap(), None);
}

#[test]
fn test_lru_eviction() {
    let dock = KVDock::new(Config {
        adapter: MemoryAdapter::new(2),
        prefix: String::new(),
        default_ttl: None,
    })
    .unwrap();

    dock.set("k1", b"v1", None).unwrap();
    dock.set("k2", b"v2", None).unwrap();
    // access k1 so k2 becomes LRU
    dock.get("k1").unwrap();
    dock.set("k3", b"v3", None).unwrap();

    assert_eq!(dock.get("k1").unwrap(), Some(b"v1".to_vec()));
    assert_eq!(dock.get("k2").unwrap(), None); // evicted!
    assert_eq!(dock.get("k3").unwrap(), Some(b"v3".to_vec()));
}

#[test]
fn test_empty_key_rejected() {
    let dock = KVDock::new(Config::default()).unwrap();
    let err = dock.set("", b"val", None).unwrap_err();
    assert_eq!(err, KVDockError::EmptyKey);
}
