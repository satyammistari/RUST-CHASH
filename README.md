# 🔁 Consistent Hash Proxy
### A Rust-based HTTP reverse proxy with a hand-rolled consistent hash ring

> *Inspired by ByteByteGo's system design series — built from scratch in Rust with zero external hashing dependencies.*

---

## 📋 Table of Contents

- [Overview](#-overview)
- [How Consistent Hashing Works](#-how-consistent-hashing-works)
- [Architecture Diagram](#-architecture-diagram)
- [Features](#-features)
- [Tech Stack](#-tech-stack)
- [Project Structure](#-project-structure)
- [Usage](#-usage)
- [API Reference](#-api-reference)
- [Demo Walkthrough](#-demo-walkthrough)
- [Hashing Algorithms](#-hashing-algorithms)
- [Virtual Nodes Explained](#-virtual-nodes-explained)
- [Limitations](#-limitations)
- [Roadmap](#-roadmap)
- [Real-World Use Cases](#-real-world-use-cases)
- [Key Properties Proven By Tests](#-key-properties-proven-by-tests)
- [Learning Outcomes](#-learning-outcomes)

---

##  Overview

This project is a **production-inspired HTTP reverse proxy** written entirely in Rust. Its core innovation is a hand-rolled **consistent hash ring** stored as a sorted `Vec<VNode>` with `O(log N)` binary search lookups — no external hashing libraries.

### The Problem It Solves

Traditional load balancers use **modulo hashing**:

```
server = hash(key) % N
```

When `N` changes (a server is added or removed), **almost every key remaps** to a new server. For caching systems, this is catastrophic — the entire cache cold-starts.

**Consistent hashing** limits remapping to only `K/N` keys on average, where `K` is total keys and `N` is server count.

| Scenario | Modulo Hashing | Consistent Hashing |
|---|---|---|
| Add 1 server to 3 | ~75% of keys remap | ~25% of keys remap |
| Remove 1 server from 4 | ~75% of keys remap | ~25% of keys remap |
| Cache hit rate after change | Near 0% | ~75% preserved |
| Complexity | O(1) | O(log N) |

---

##  How Consistent Hashing Works

### The Library Analogy (from ByteByteGo)

> *Consistent hashing assigns each book (key) to a specific shelf (server) in the library, based on its number.*

Imagine a circular ring numbered `0` to `2³²`. Each server gets placed at positions on this ring. When a key arrives, you hash it to find its position, then walk **clockwise** to the first server you encounter — that server handles the key.

### Step 1 — Initial Ring with 3 Servers

```
        Server A (🟥)
       /              \
    key1             key2
      \               /
   Server C (🟦) — Server B (🟩)
              |
            key3
```

- `key1` is between C and A → routes to **Server A** (next clockwise)
- `key2` is between A and B → routes to **Server B**
- `key3` is between B and C → routes to **Server C**

### Step 2 — Adding New Server X (between C and A)

```
        Server A (🟥)
       /      ↑
    Server X (🟨)   key2
    ↑               /
   key1 (MOVED)   Server B (🟩)
      \           /
       Server C (🟦)
            |
          key3
```

- `key1` was going to A — now routes to **Server X** ✅ (only change!)
- `key2` still routes to **Server B** 🔒 (unchanged)
- `key3` still routes to **Server C** 🔒 (unchanged)

**Only keys between the new server's predecessor and itself were affected.**

### Step 3 — Removing Server A

```
       Server B (🟩)
      /              \
   key1 (rerouted)  key2
   key2              |
      \           Server C (🟦)
       ↖ (wrap)       |
                    key3
```

- Keys from Server A move to the **next clockwise server** (B)
- All other keys are completely undisturbed

---

## 🗺 Architecture Diagram
<img width="531" height="676" alt="Pasted image 20260301163930" src="https://github.com/user-attachments/assets/a3e2be18-0c86-48f6-a25f-e0e375bce0a5" />



### Hash Ring Internal Structure

```
Sorted Vec<VNode> — 3 servers × 150 virtual nodes = 450 entries

Index:  [0]      [1]      [2]      [3]      [4]   ...  [449]
Hash:   892    14,231   29,445   41,892   57,234  ...  4,291,223,441
Owner:   A        B        A        C        B    ...      A

key_hash = 2,847,293,441
partition_point(|v| v.hash < key_hash) → index 387
vnodes[387].server = "http://127.0.0.1:8082"   ← O(log 450) = ~9 comparisons
```

---

##  Features

| Feature | Description |
|---|---|
|  **Consistent Hash Ring** | Hand-rolled sorted Vec + binary search, no libraries |
|  **Virtual Nodes** | 150 vnodes per server for even distribution |
|  **O(log N) Lookup** | `partition_point` binary search on sorted Vec |
|  **Hot Server Management** | Add/remove servers at runtime via REST API |
|  **Ring Visualizer** | Live JSON snapshot showing ring coverage per server |
|  **Multiple Routing Strategies** | Route by URL path, header value, or client IP |
|  **Request Stats** | Per-backend request counters |
|  **Two Hash Algorithms** | FNV-1a (fast) and MurmurHash3 (better distribution) |
|  **Thread Safe** | `Arc<RwLock<T>>` — concurrent reads, exclusive writes |
|  **Structured Logging** | `tracing` with `RUST_LOG` env var control |
|  **Health Endpoints** | `/healthz` and `/admin/health` |
|  **Graceful Shutdown** | Drains in-flight requests on Ctrl+C |

---

## 🛠 Tech Stack

| Layer | Technology | Why |
|---|---|---|
| Language | **Rust 2021** | Memory safety, zero-cost abstractions, no GC |
| Async Runtime | **Tokio** | Industry standard; powers the entire ecosystem |
| HTTP Server | **Axum 0.7** | Ergonomic routing built on Hyper/Tokio |
| HTTP Client | **Hyper 1.x** | Low-level control for request forwarding |
| Serialization | **Serde + serde_json** | Derive macros eliminate boilerplate |
| Config | **TOML** | Human-readable, Rust-native format |
| Logging | **tracing** | Structured, async-aware, zero-overhead |
| Hashing | **Hand-rolled** | FNV-1a + MurmurHash3 — no external deps |

---

## 📁 Project Structure

```
consistent-hash-proxy/
│
├── Cargo.toml                    # Dependencies and binary targets
├── config.toml                   # Runtime configuration
│
├── src/
│   ├── main.rs                   # Entrypoint — builds router, starts server
│   ├── lib.rs                    # Library root (for integration tests)
│   ├── config.rs                 # Config structs + TOML loading
│   │
│   ├── ring/                     # ── THE CORE ALGORITHM ──
│   │   ├── mod.rs                # Re-exports
│   │   ├── algorithms.rs         # FNV-1a + MurmurHash3 from scratch
│   │   ├── vnode.rs              # VNode struct with Ord impl
│   │   └── hash_ring.rs          # HashRing — sorted Vec + binary search
│   │
│   ├── proxy/                    # ── HTTP PROXY LAYER ──
│   │   ├── mod.rs
│   │   ├── client.rs             # Hyper client wrapper
│   │   └── handler.rs            # Request routing + forwarding
│   │
│   └── admin/                    # ── ADMIN API ──
│       ├── mod.rs                # Router builder
│       ├── routes.rs             # Endpoint handlers
│       └── visualizer.rs         # Ring → JSON visualization
│
├── backends/
│   └── dummy_server.rs           # Echo server for testing
│
└── tests/
    └── ring_tests.rs             # Integration tests
```

---

##  Usage

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# On Windows (PowerShell)
winget install Rustlang.Rustup
```

### Build

```bash
git clone <your-repo>
cd consistent-hash-proxy
cargo build
```

### Run (4 terminals)

**Terminal 1, 2, 3 — Backends:**
```bash
cargo run --bin dummy-server -- 8081
cargo run --bin dummy-server -- 8082
cargo run --bin dummy-server -- 8083
```

**Terminal 4 — Proxy:**
```bash
# Windows PowerShell
$env:RUST_LOG="info"; cargo run --bin proxy

# Linux / macOS
RUST_LOG=info cargo run --bin proxy
```

### Run Tests

```bash
cargo test                    # all tests
cargo test -- --nocapture     # show println! output
cargo test ring               # only ring tests
```

---

## 📡 API Reference

### Proxy Endpoint

| Route | Method | Description |
|---|---|---|
| `/*` | ANY | All traffic proxied to backend via consistent hash |
| `/healthz` | GET | Quick liveness check |

### Admin API

| Route | Method | Description |
|---|---|---|
| `/admin/health` | GET | Health + server count |
| `/admin/servers` | GET | List all backends in ring |
| `/admin/servers` | POST | Add a backend server |
| `/admin/servers/:addr` | DELETE | Remove a backend server |
| `/admin/ring/visualize` | GET | Full ring snapshot + distribution analysis |
| `/admin/stats` | GET | Request counts per backend |

### Request / Response Examples

**Add a server:**
```bash
curl -X POST http://localhost:8080/admin/servers \
  -H "Content-Type: application/json" \
  -d '{"address":"http://127.0.0.1:8084"}'
```
```json
{
  "success": true,
  "message": "Added http://127.0.0.1:8084",
  "data": { "total_servers": 4, "total_vnodes": 600 }
}
```

**Remove a server (URL-encode the address):**
```bash
curl -X DELETE "http://localhost:8080/admin/servers/http%3A%2F%2F127.0.0.1%3A8082"
```

**Ring Visualizer response:**
```json
{
  "snapshot": {
    "algorithm": "Fnv1a",
    "total_vnodes": 450,
    "server_count": 3,
    "servers": [
      { "address": "http://127.0.0.1:8081", "ring_coverage_percent": 33.4 },
      { "address": "http://127.0.0.1:8082", "ring_coverage_percent": 32.9 },
      { "address": "http://127.0.0.1:8083", "ring_coverage_percent": 33.7 }
    ],
    "sample_routes": [
      { "key": "/api/users/1",  "hash": 2847293441, "server": "http://127.0.0.1:8082" },
      { "key": "/api/users/42", "hash": 1923847291, "server": "http://127.0.0.1:8081" }
    ]
  },
  "analysis": {
    "is_balanced": true,
    "imbalance_ratio": 1.02,
    "recommendation": "Ring is well-balanced. Imbalance ratio: 1.02x."
  },
  "ascii_ring": "Ring [0----------2^32]\n[AABABCBCABC...]\nLegend:\n  A = :8081 (33.4%)"
}
```

---

## 🎬 Demo Walkthrough

### 1. Prove Consistency — Same key always same server

```bash
curl http://localhost:8080/api/users/42   # → backend 8082
curl http://localhost:8080/api/users/42   # → backend 8082  ✅ same
curl http://localhost:8080/api/users/42   # → backend 8082  ✅ same
```

### 2. Prove Different Keys Route Differently

```bash
curl http://localhost:8080/api/users/1    # → 8081
curl http://localhost:8080/api/users/2    # → 8083
curl http://localhost:8080/api/users/3    # → 8082
```

### 3. Prove Minimal Redistribution — Add Server Live

```bash
# Before: record where each key goes
curl http://localhost:8080/api/users/1   # note the port
curl http://localhost:8080/api/users/10  # note the port
curl http://localhost:8080/api/users/20  # note the port

# Add 4th server
curl -X POST http://localhost:8080/admin/servers \
  -H "Content-Type: application/json" \
  -d '{"address":"http://127.0.0.1:8084"}'

# After: most keys stay on same server
curl http://localhost:8080/api/users/1   # likely unchanged 🔒
curl http://localhost:8080/api/users/10  # likely unchanged 🔒
curl http://localhost:8080/api/users/20  # may have moved to 8084 ✅
```

**~75% of keys stay put — only ~25% move to the new server.**

### 4. Prove Hot Removal — No Restart Needed

```bash
# Remove server 8082
curl -X DELETE "http://localhost:8080/admin/servers/http%3A%2F%2F127.0.0.1%3A8082"

# Traffic immediately reroutes — 8082 never receives another request
curl http://localhost:8080/api/users/42  # now goes to 8081 or 8083
```

---

## #️⃣ Hashing Algorithms

### FNV-1a (Fowler–Noll–Vo)

```rust
fn fnv1a_32(input: &str) -> u32 {
    const FNV_PRIME:  u32 = 16_777_619;
    const FNV_OFFSET: u32 = 2_166_136_261;

    input.bytes().fold(FNV_OFFSET, |hash, byte| {
        (hash ^ byte as u32).wrapping_mul(FNV_PRIME)
    })
}
```

- ✅ Extremely fast — 2 ops per byte
- ✅ Great for short strings (URLs, keys)
- ✅ No setup/state needed
- ⚠️ Weaker for long strings with common prefixes

### MurmurHash3 (32-bit)

- ✅ Excellent avalanche — tiny input change → totally different hash
- ✅ Better for long, similar strings
- ✅ Used in Cassandra, Redis
- ⚠️ Slightly more compute per byte

**Configure in `config.toml`:**
```toml
[proxy]
hash_algorithm = "fnv1a"    # or "murmur3"
```

---

## 🔮 Virtual Nodes Explained

Without virtual nodes, with only 3 servers:

```
Server A → position 1,200,000   (owns 39% of ring) ← HOT
Server B → position 2,500,000   (owns 33% of ring)
Server C → position 3,900,000   (owns 28% of ring) ← COLD
```

Very unequal. Server A handles almost 40% of traffic.

With **150 virtual nodes each** (450 total):

```
Server A has 150 positions spread across the ring → ~33.3% ✅
Server B has 150 positions spread across the ring → ~33.3% ✅
Server C has 150 positions spread across the ring → ~33.3% ✅
```

Standard deviation drops from **~200%** with 1 vnode to **~10%** with 150 vnodes.

| Virtual Nodes | Load Std Deviation | Memory (3 servers) |
|---|---|---|
| 1 | ~200% | 3 entries |
| 10 | ~50% | 30 entries |
| 100 | ~15% | 300 entries |
| 150 | ~10% | 450 entries ← sweet spot |
| 500 | ~5% | 1500 entries |

---

## ⚠️ Limitations

| Limitation | Details |
|---|---|
| **No HTTPS** | Current `HttpConnector` is HTTP only. Add `hyper-rustls` for TLS. |
| **No Health Checks** | Dead backends stay in the ring until manually removed. |
| **No Retry Logic** | A single backend failure returns 502 immediately. |
| **No Weighted Nodes** | All servers get equal vnode counts regardless of capacity. |
| **No Persistence** | Ring state is in-memory — restarting the proxy rebuilds from config. |
| **No Auth on Admin API** | `/admin/*` endpoints have no authentication. |
| **Single Process** | No clustering — multiple proxy instances have separate ring state. |
| **HTTP/1.1 Only** | No HTTP/2 or HTTP/3 support. |
| **Body Buffering** | Request bodies are fully buffered in memory before forwarding. |

---

## 🗺 Roadmap

### v0.2 — Reliability
- [ ] **Active health checks** — periodic pings to backends, auto-remove on failure
- [ ] **Retry with fallback** — on 502, try the next vnode on the ring
- [ ] **Circuit breaker** — stop routing to failing backends temporarily
- [ ] **Connection pooling** — reuse TCP connections to backends

### v0.3 — Features
- [ ] **Weighted virtual nodes** — give powerful servers more vnodes
- [ ] **HTTPS support** — `hyper-rustls` for TLS backends
- [ ] **Admin authentication** — API key or JWT for `/admin/*`
- [ ] **Prometheus metrics** — expose `/metrics` endpoint
- [ ] **WebSocket proxying** — handle `Upgrade: websocket`

### v0.4 — Production
- [ ] **Persistent ring state** — save/restore ring from disk on restart
- [ ] **Distributed mode** — gossip protocol to sync ring across proxy instances
- [ ] **Hot config reload** — watch `config.toml` for changes
- [ ] **Request tracing** — propagate trace IDs through `X-Trace-Id` headers
- [ ] **Docker image** — multi-stage Dockerfile

### v1.0 — Observable
- [ ] **Web dashboard** — React UI showing the ring visually (like the ByteByteGo diagram)
- [ ] **Load test harness** — built-in `wrk`-style benchmarking endpoint
- [ ] **Chaos mode** — randomly kill backends to test redistribution

---

## 🏭 Real-World Use Cases

### 1. Distributed Cache (Primary Use Case)
Hash by cache key. When a cache node joins, only its share of keys miss — the rest stay warm.
```
GET /cache/user:profile:42  → always routes to same backend
```

### 2. Session Affinity (Sticky Sessions)
Hash by User-ID header. Same user always hits same backend — no shared session store needed.
```toml
routing_key_strategy = "header"
routing_header       = "X-User-Id"
```

### 3. Database Sharding
Hash by primary key prefix to route writes to the owning shard.
```
POST /data/tenant-A/records  → shard 1
POST /data/tenant-B/records  → shard 2
```

### 4. Canary Deployments
Add a new "canary" backend with fewer virtual nodes to receive a fraction of traffic:
- 150 vnodes = ~33% traffic (normal)
- 15 vnodes  = ~3% traffic (canary)
- 0 vnodes   = removed from rotation

### 5. Multi-Region Routing
Place backends in different regions. Use geographic hash keys to keep users close to their data.

---

## ✅ Key Properties Proven By Tests

```
cargo test -- --nocapture
```

| Test | What It Proves |
|---|---|
| `ring_routes_consistently` | Same key → same server every time |
| `ring_vnodes_are_sorted` | Internal invariant: Vec always sorted |
| `minimal_redistribution_on_add` | <35% of keys remap when adding server |
| `minimal_redistribution_when_removing` | Only removed server's keys move |
| `load_is_reasonably_even_with_150_vnodes` | Each server gets 25–42% of traffic |
| `more_vnodes_means_better_distribution` | CV decreases as vnode count rises |
| `both_algorithms_distribute_reasonably` | FNV1a and Murmur3 both balance well |
| `coverage_sums_to_roughly_100_percent` | Ring segments cover full hash space |
| `fnv1a_known_value` | Cross-check against reference implementation |
| `avalanche_property` | 1-char change → drastically different hash |

---

## 🎓 Learning Outcomes

By studying this project you will deeply understand:

**Rust Concepts:**
- Ownership, borrowing, and why `Arc<RwLock<T>>` is needed
- `Option<T>` and `Result<T,E>` — eliminating null and exceptions
- Trait implementations (`Ord`, `Display`, `Default`, `Serialize`)
- Iterator chains (`map`, `filter`, `fold`, `partition_point`)
- `async/await` and Tokio's cooperative multitasking
- Closures and the `move` keyword
- The module system and `pub use` re-exports
- Wrapping arithmetic for intentional overflow in hashing

**Distributed Systems Concepts:**
- Why modulo hashing fails when cluster size changes
- How virtual nodes fix hotspot problems
- The relationship between vnode count and load variance
- Why `O(log N)` binary search is the right tradeoff here
- Arc length as a proxy for traffic share
- The "successor node" concept on a circular hash ring
- Reader-writer lock patterns for high-read, low-write workloads

---

## 📖 References

- [ByteByteGo — Consistent Hashing](https://bytebytego.com) — inspiration for this project
- [Amazon DynamoDB Paper](https://www.allthingsdistributed.com/files/amazon-dynamo-sosp2007.pdf) — original consistent hashing in production
- [Apache Cassandra Architecture](https://cassandra.apache.org/doc/latest/cassandra/architecture/overview.html) — uses virtual nodes exactly as implemented here
- [FNV Hash Reference](http://www.isthe.com/chongo/tech/comp/fnv/) — original FNV specification
- [MurmurHash3 Reference](https://github.com/aappleby/smhasher) — Austin Appleby's original implementation

---

## 📄 License

MIT License — use freely, attribution appreciated.

---

<div align="center">

**Built with Rust 🦀 | Inspired by ByteByteGo 📚 | Consistent Hashing from scratch 🔁**

</div>
