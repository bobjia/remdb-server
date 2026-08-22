# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This workspace contains two Rust crates and two sibling projects:

- **remdb** (`remdb/`): The core embedded in-memory database library — ACID transactions, SQL parsing/execution, vector indexes (HNSW, IVF), time-series storage, JSON, RBAC, PubSub (UDP-based), HA master-slave replication, and ONNX model inference. Supports `no_std`/baremetal.
- **remdb-server** (root `src/`): A network server wrapping remdb — JDBC protocol, CLI, snapshot/checkpoint scheduling, Prometheus monitoring, Milvus-compatible REST API.
- **remdb-python** (`remdb-python/`): Python bindings via pybind11, with NumPy/Pandas integration.
- **jdbc-driver** (`jdbc-driver/`): Java JDBC driver for connecting to remdb-server.

## Build Commands

```bash
# Build everything
cargo build

# Release build (optimized for size: opt-level="z", LTO, panic=abort)
cargo build --release

# Build remdb alone (core library)
cargo build -p remdb

# Build for no_std/baremetal
cargo build --no-default-features --features=baremetal -p remdb

# Build with specific features
cargo build -p remdb --features "pubsub ha"

# Run all tests (single-threaded, 16MB stack)
cargo test

# Test a specific crate
cargo test --lib -p remdb

# Run a single test
cargo test --lib test_name

# Run integration tests
cargo test --test integration_tests

# Run benchmarks
cargo bench

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

## Running

```bash
# Server with default config
cargo run --bin remdb-server

# Server with custom config
cargo run --bin remdb-server -- --config remdb-slave.toml

# Debug mode
cargo run --bin remdb-server -- --debug

# CLI mode (standalone, no JDBC)
cargo run --bin remdbcli

# Benchmark mode
cargo run --bin remdb-server -- benchmark --test-type query --query-count 100000
```

## Architecture

### remdb (Core Library, `remdb/src/`)

The core library supports `no_std` with `#![cfg_attr(not(feature = "std"), no_std)]`. Uses `alloc` crate and platform abstraction.

```
lib.rs → RemDb struct (main entry point)
├── types.rs        → DataType, Value, TableDef, FieldDef, RemDbError, IndexType
├── table.rs        → MemoryTable (row storage), RecordRef (zero-copy), RecordCursor
├── index.rs        → Index traits + PrimaryIndex, BTreeIndex, TTreeIndex
├── index/          → builder.rs, hnsw.rs, ivf.rs
├── transaction.rs  → ACID transactions, LogManager, WAL
├── sql/            → query_parser.rs, query_executor.rs, result_set.rs
│   ├── functions/  → aggregate, string, math, time, json
│   └── operations/ → ddl, dml, expression, comparison, vector
├── time_series/    → partitioning, compression, lifecycle/retention
├── json/           → document store, path queries, memory pool
├── ha/             → master-slave replication, heartbeat, failover
├── pubsub/         → UDP-based publish/subscribe
├── rbac/           → role-based access control
├── model/          → ONNX model inference
├── memory/         → custom allocator for embedded systems
├── platform/       → POSIX/baremetal abstraction
├── monitor.rs      → DbMetrics, HealthCheckResult
├── system_tables.rs→ built-in system tables
└── config.rs       → DbConfig, WALConfig, MemoryAllocator trait
```

### remdb-server (Server, `src/`)

```
main.rs → loads config, init DB, start JDBC server + timers
├── lib.rs           → library exports + debug mode flag
├── context.rs       → AppContext (shared server state)
├── jdbc_server.rs   → TCP-based JDBC protocol server
├── cli.rs           → interactive CLI (rustyline)
├── remdbcli.rs      → standalone CLI binary
├── ddl_compiler.rs  → DDL file → TableDef compilation
├── snapshot_loader.rs→ full/incremental snapshot load/save
├── sql_engine/      → server-level SQL execution
│   ├── parser.rs, select.rs, insert.rs, update.rs, delete.rs, ddl.rs
├── handler/         → jdbc_handler.rs, health_monitor.rs, safe_database_ops.rs
├── bootstrap/       → platform & service initialization
├── config/          → TOML config loading + clap CLI args
├── network/         → zero_copy_transport.rs
├── pool/            → connection_pool.rs
├── scheduler/       → checkpoint.rs, snapshot.rs
├── benchmark/       → built-in benchmark harness
├── milvus/          → Milvus-compatible REST API
└── tuning/          → dynamic system tuner
```

### Key Design Decisions

- **Global state**: DB instance, memory allocator, and HA manager are all global statics. Tests must run single-threaded (`serial_test`).
- **Memory management**: Fixed-size pool allocated at startup via `Vec<u8>` + `forget()`, managed by custom allocator. No heap allocation after init.
- **Zero-copy**: `RecordRef` provides zero-copy read views. Network layer has `zero_copy_transport`.
- **Platform abstraction**: `Platform` trait enables baremetal (no_std) or POSIX operation.
- **Three ways to define tables**: Macro-based (`remdb::table!`), derive macro (`#[derive(MemdbTable)]`), or runtime DDL.

## Panic-Free Requirement

**Panic is not allowed anywhere in the codebase.** The following are strictly forbidden:

- `unwrap()` / `expect()` / `unwrap_unchecked()` / `unwrap_or_default()` on `Result` or `Option`
- `panic!()` / `todo!()` / `unreachable!()` / `unimplemented!()`
- `assert!()` / `debug_assert!()` (use `if`-based checks with `?` instead)
- `[i]` indexing on `Vec`, `[T]`, or `[T; N]` without explicit bounds check (use `.get(i)` / `.get_mut(i)` and handle the `None` case)
- `[i..j]` slicing that could fail (validate bounds first)
- Integer overflow that would panic (use `checked_*` / `wrapping_*` / `saturating_*` as appropriate)
- `mem::uninitialized()` / `transmute()` that could produce invalid state

Always propagate errors with `?` or handle them explicitly. Every match on `Result` or `Option` must handle the error/`None` arm — do not use `if let Ok(v)` as a substitute for full match (it silently drops the error).

## Feature Flags (remdb)

| Feature | Description |
|---------|-------------|
| `std` | Standard library support (default) |
| `posix` | POSIX platform support (default, implies `std`) |
| `baremetal` | no_std bare metal (uses `heapless`) |
| `pubsub` | UDP-based pub/sub messaging (implies `std`) |
| `ha` | High availability (implies `pubsub`) |
| `c-api` | C language API |
| `log` | Logging support |
| `wal-compression-lz4` | LZ4 WAL compression |
| `wal-compression-zstd` | Zstd WAL compression |
| `model-runtime` | ONNX model inference (ort, ndarray, tokio) |
| `model-download` | Model download via HTTP (reqwest) |

Default features: `std`, `posix`, `ha`, `pubsub`, `c-api`, `log`

## Key Types

- `RemDb`: Main database instance (in `remdb/src/lib.rs`)
- `MemoryTable`: Row-based table storage (in `table.rs`)
- `Value` / `DataType`: Dynamic value and schema types (in `types.rs`)
- `TableDef` / `FieldDef`: Table and column schema definitions
- `RecordRef`: Zero-copy record view
- `RecordCursor`: Scan iterator
- `RemDbError` / `Result<T>`: Error handling via `thiserror`
- `SecondaryIndex` trait: Interface for all secondary indexes
- `DdlExecutor` trait: Runtime DDL operations
- `AppContext`: Server-level shared state (in `src/context.rs`)
- `ServerError` / `ServerResult<T>`: Server-level error types

## Important Patterns

### Error Handling

- remdb uses `RemDbError` (thiserror enum) and `Result<T>` (type alias for `Result<T, RemDbError>`)
- remdb-server uses `ServerError` (thiserror enum) and `ServerResult<T>` (type alias for `Result<T, ServerError>`)
- `ServerError` has `From<RemDbError>` impl — use `?` to convert between them
- The `try_lock!`, `try_read!`, `try_write!` macros handle poisoned mutexes safely

### Conditional Compilation

Heavy use of `#[cfg(feature = "std")]`, `#[cfg(feature = "ha")]`, `#[cfg(feature = "pubsub")]`, `#[cfg(feature = "log")]` gates throughout remdb.

### Testing

Tests use `serial_test` due to global state. Configured in `[tool.cargo.test]` with `test-threads = 1` and `stack-size = 16777216`. Integration tests live in `tests/` at workspace root.

### Server Entry Point

The server (`main.rs`) follows this startup sequence:
1. Parse CLI args and config files
2. Initialize platform (POSIX)
3. Compile DDL or load table definitions from snapshots
4. Allocate memory pool via `Vec<u8>` + `forget()`
5. Initialize global allocator and database
6. Restore data from WAL or snapshots
7. Start JDBC server, checkpoint timer, snapshot timer
8. Optionally start PubSub/HA/Milvus REST API
9. Enter interactive CLI or wait for Ctrl+C