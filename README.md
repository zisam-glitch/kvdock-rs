# kvdock-rs

High-performance, generic key-value caching layer for Rust with zero external dependencies.

## Architecture

- **`KVDock`**: Thread-safe cache engine using `Arc` and atomic counters.
- **`StorageAdapter`**: Trait defining backend operations (`Send + Sync`).
- **`MemoryAdapter`**: In-memory storage with LRU eviction and monotonic `Instant` TTL expiration.
- **Cache-Aside**: `get_or_set` closure evaluated only upon misses.
- **Zero Runtime Dependencies**: Rust standard library only.

## Usage

```rust
use std::time::Duration;
use kvdock::{Config, KVDock, MemoryAdapter, SetOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dock = KVDock::new(Config {
        adapter: MemoryAdapter::new(10_000),
        prefix: "app".to_string(),
        default_ttl: Some(Duration::from_secs(60)),
    })?;

    // set value
    dock.set("user:101", b"Alice", None)?;

    // cache-aside pattern
    let value = dock.get_or_set("user:102", None, || {
        Ok(b"Bob".to_vec())
    })?;

    println!("value: {:?}", String::from_utf8(value)?);
    Ok(())
}
```

## Testing

```sh
cargo test
```

## License

MIT
