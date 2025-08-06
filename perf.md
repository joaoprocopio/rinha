Looking at your Rust HTTP proxy/load balancer code, here are the key areas where you can improve performance and reduce P99 latency from 2ms to under 1ms:

## 1. **Remove Async Locks from Hot Path**

Your biggest bottleneck is using `RwLock` in the request handling path:

**Problem:** In `rinha_worker/mod.rs`, you're acquiring a write lock on storage for every successful request:

```rust
let storage = rinha_storage::get_storage();
let mut storage = storage.write().await;  // This blocks!
```

**Solutions:**

- Use lockless data structures (e.g., `crossbeam::atomic::AtomicCell` or `Arc<AtomicU64>` counters)
- Batch writes to reduce lock contention
- Use a separate writer thread with a channel for storage updates
- Consider using `parking_lot::RwLock` (non-async) with `spawn_blocking` for short critical sections

## 2. **Optimize Connection Pooling**

You're creating new TCP connections for every request:

```rust
let mut sender = rinha_net::create_tcp_socket_sender(upstream.addr).await?;
```

**Solution:** Implement connection pooling:

- Keep persistent connections to upstreams
- Reuse HTTP/1.1 connections with keep-alive
- Consider HTTP/2 for multiplexing

## 3. **Reduce Memory Allocations**

Several allocation hotspots:

**Current issues:**

- String formatting for URIs on every request
- JSON serialization/deserialization on every request
- Multiple `Arc` clones

**Solutions:**

- Pre-compute URI strings and reuse them
- Use `Bytes` instead of `String` where possible
- Pool request/response objects
- Use stack-allocated buffers for small data

## 4. **Optimize Health Check Strategy**

Your health checks run every 5 seconds and could be blocking:

**Improvements:**

- Reduce health check interval to 1-2 seconds for faster failure detection
- Use circuit breaker pattern instead of periodic health checks
- Make health checks non-blocking with separate connection pools

## 5. **Eliminate Unnecessary Boxing**

You're using `BoxBody` everywhere which adds allocation overhead:

**Solution:**

- Use concrete body types where possible
- Minimize trait object usage in hot paths

## 6. **TCP Socket Optimizations**

Your socket configuration is good, but you can tune further:

```rust
// Add these optimizations
socket.set_tcp_user_timeout(Some(Duration::from_millis(100)))?;
socket.set_linger(Some(Duration::ZERO))?;  // For faster connection cleanup
```

## 7. **Async Runtime Optimizations**

Consider switching from `current_thread` to `multi_thread`:

```rust
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
```

For low-latency applications, pinning to specific cores can help.

## 8. **Storage Architecture Redesign**

Your `BTreeMap` storage with `DateTime` keys is expensive:

**Better approach:**

- Use simple atomic counters for metrics
- Store only essential data (count, sum)
- Implement time-windowed storage with fixed-size arrays

## Quick Wins (Implement First):

1. Replace async locks with atomic operations
2. Implement basic connection pooling
3. Pre-compute URI strings
4. Remove unnecessary `BoxBody` usage
5. Switch to multi-threaded runtime

## Example Connection Pool Structure:

```rust
struct ConnectionPool {
    connections: Vec<Arc<Mutex<Option<SendRequest<BoxBody>>>>>,
    next: AtomicUsize,
}
```

These changes should help you achieve sub-1ms P99 latency. Focus on eliminating the async lock contention first, as that's likely your biggest bottleneck.
