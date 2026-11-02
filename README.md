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

## Ecosystem & Other Languages

KVDock is also available natively across multiple languages with matching design principles and zero external dependencies:

- **TypeScript / Node.js**: [`zisam-glitch/kvdock`](https://github.com/zisam-glitch/kvdock) — Reference implementation with Tiered Caching, SQLite, S3, Redis, and Cloudflare KV adapters.
- **Go**: [`zisam-glitch/kvdock-go`](https://github.com/zisam-glitch/kvdock-go) — Generic key-value cache engine with atomic telemetry.
- **Swift**: [`zisam-glitch/kvdock-swift`](https://github.com/zisam-glitch/kvdock-swift) — Actor-isolated concurrent cache with `ContinuousClock`.
- **Kotlin**: [`zisam-glitch/kvdock-kt`](https://github.com/zisam-glitch/kvdock-kt) — Thread-safe $O(1)$ LRU key-value cache for the JVM.

## License

MIT
