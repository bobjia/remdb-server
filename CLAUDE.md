# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

remdb-server is a high-performance in-memory database system. The workspace has two crates:

- **remdb** (`remdb/`): The core embedded database library (Rust, supports `no_std`). Features ACID transactions, SQL parsing/execution, vector indexes (HNSW, IVF), time-series storage, JSON, RBAC, PubSub (UDP-based), HA master-slave replication, and ONNX model inference.
- **remdb-server** (root `src/`): A network server wrapping remdb, providing JDBC protocol, a CLI, snapshot/checkpoint scheduling, and Prometheus monitoring.

Two sibling projects live alongside the Rust workspace:

- **remdb-python** (`remdb-python/`): Python bindings via pybind11, with NumPy/Pandas integration.
- **jdbc-driver** (`jdbc-driver/`): Java JDBC driver for connecting to remdb-server.

## Build Commands

### Rust (workspace)

```bash
# Build everything
cargo build

# Release build (optimized for size: opt-level="z", LTO, panic=abort)
cargo build --release

# Run all tests (single-threaded, 16MB stack per test)
cargo test

# Test a specific crate
cargo test --lib -p remdb

# Test a single test function
cargo test --lib test_name

# Run integration tests
cargo test --test integration_tests

# Run benchmarks
cargo bench

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt

# Build for no_std/baremetal (remdb only)
cargo build --no-default-features --features=baremetal -p remdb
```

Note: `Cargo.toml` sets `[tool.cargo.test]` with `test-threads = 1` and `stack-size = 16777216`. Tests use `serial_test` crate due to shared global state.

### Python Bindings

```bash
cd remdb-python

# Install in development mode
pip install -e .

# Build C extension (Windows)
python setup.py build_ext --inplace

# Run all tests
python run_tests.py

# Run unit tests only
python run_tests.py --unit

# Run tests with pytest
python run_tests.py --pytest

# Run a specific test file
python -m pytest tests/unit/test_data_types.py -v

# Run a specific test class
python -m unittest tests.unit.test_data_types.TestDataTypeINTEGER
```

### JDBC Driver

```bash
cd jdbc-driver

# Build JAR
mvn clean package

# Install to local Maven repository
mvn clean install
```

## Running the Server

```bash
# Default config (remdb-master.toml)
cargo run --bin remdb-server

# Custom config
cargo run --bin remdb-server -- --config remdb-slave.toml

# Debug mode
cargo run --bin remdb-server -- --debug

# Benchmark mode
cargo run --bin remdb-server -- benchmark --test-type query --query-count 100000

# CLI mode (standalone, no JDBC)
cargo run --bin remdbcli
```

## Architecture

### Server Binary (`src/`)

Entry point: `src/main.rs` — loads TOML config, initializes the global memory allocator, creates tables from DDL or snapshots, starts the JDBC server, checkpoint timer, snapshot timer, and optional PubSub/HA subsystems.

```
src/
├── main.rs              # Server entry point, config parsing
├── lib.rs               # Library exports + debug mode flag
├── jdbc_server.rs       # TCP-based JDBC protocol server
├── remdbcli.rs          # Standalone CLI binary
├── cli.rs               # Interactive CLI (rustyline)
├── context.rs           # AppContext (shared server state)
├── ddl_compiler.rs      # DDL file → TableDef compilation
├── snapshot_loader.rs   # Full/incremental snapshot load/save
├── error.rs             # ServerError, ServerResult
├── macros.rs            # Internal macros
├── bootstrap/           # Platform and service initialization
│   ├── mod.rs, platform.rs, service.rs
├── config/              # TOML config loading + CLI args (clap)
│   ├── mod.rs, loader.rs, tests.rs
├── handler/             # Request handlers
│   ├── jdbc_handler.rs, health_monitor.rs, safe_database_ops.rs
├── network/             # Zero-copy network transport
│   ├── mod.rs, zero_copy_transport.rs
├── pool/                # Connection pooling
│   ├── mod.rs, connection_pool.rs
├── proto/               # Protocol buffer definitions
│   ├── mod.rs
├── scheduler/           # Periodic checkpoint & snapshot tasks
│   ├── mod.rs, checkpoint.rs, snapshot.rs
├── sql_engine/          # Server-level SQL execution (delegates to remdb)
│   ├── mod.rs, parser.rs, select.rs, insert.rs, update.rs, delete.rs, ddl.rs
├── tuning/              # Dynamic system tuning (CPU/memory load)
│   ├── mod.rs, system_tuner.rs
├── benchmark/           # Built-in benchmark harness
│   ├── mod.rs, jdbc_benchmark.rs
```

### Core Database Library (`remdb/src/`)

The core library supports `no_std` (via `#![cfg_attr(not(feature = "std"), no_std)]`). It uses `alloc` crate and platform abstraction for portability.

```
remdb/src/
├── lib.rs               # Library entry, re-exports, module declarations
├── types.rs             # Core types: DataType, Value, TableDef, FieldDef,
│                        #   IndexType, VectorIndexType, DistanceType,
│                        #   VectorMetadata, RemDbError, RecordStatus
├── table.rs             # MemoryTable — row-based in-memory storage,
│                        #   RecordRef (zero-copy view), RecordCursor
├── index.rs             # Index trait + PrimaryIndex, BTreeIndex, TTreeIndex,
│                        #   SecondaryIndex trait
├── index/
│   ├── builder.rs       # Index build thread pool
│   ├── hnsw.rs          # HNSW vector index
│   └── ivf.rs           # IVF / IVF_PQ / IVF_FLAT vector indexes
├── transaction.rs       # ACID transactions: IsolationLevel, Transaction,
│                        #   LogManager, WAL
├── sql/
│   ├── mod.rs           # SqlQuery, SqlError, result types
│   ├── query_parser.rs  # SQL → SqlQuery AST (SELECT, INSERT, UPDATE, DELETE)
│   ├── query_executor.rs# AST → execution (delegates to operations/)
│   ├── result_set.rs    # ResultRow, ResultSet, ResultRowIter
│   ├── functions/       # SQL functions: aggregate, string, math, time, json
│   ├── operations/      # DDL, DML, expression eval, comparison, vector ops
│   └── error.rs, utils.rs
├── time_series/         # Time-series: TimeSeriesTable, partitioning,
│                        #   compression (delta), lifecycle/retention
├── json/                # JSON document store: document, path, memory_pool
├── ha/                  # High Availability: master-slave replication,
│                        #   heartbeat, role management, sync protocol
├── pubsub/              # UDP-based pub/sub: publisher, subscriber,
│                        #   topics, TTL ringbuffer, protocol
├── model/               # ONNX model inference: model_manager, onnx_runtime,
│                        #   model_udf, cache, worker_manager, downloader
├── rbac/                # Role-based access control: User, Role, Permission
├── memory/              # Custom allocator for embedded systems
├── platform/            # Platform abstraction (POSIX/baremetal)
├── config.rs            # DbConfig, WALConfig, MemoryAllocator trait
├── compression.rs       # CompressionScheme
├── system_tables.rs     # Built-in system tables
├── monitor.rs           # DbMetrics, HealthCheckResult
├── c_api.rs             # C language API
├── log.rs               # Logging (optional, feature-gated)
├── utf8.rs              # UTF-8 processing
└── wal_compression.rs   # WAL compression (LZ4/Zstd)
```

## Key Types and Traits

### Core Types (`remdb/src/types.rs`)

- `DataType`: Integer, Real, Text, Boolean, Timestamp, Vector, JSON, etc.
- `Value`: Dynamic value type for SQL results (with TypedValue variants)
- `TableDef`: Schema definition (fields, indexes, constraints)
- `FieldDef`: Column definition (name, type, nullable, default, etc.)
- `IndexType`: Hash, BTree, TTree, HNSW, IVF, etc.
- `VectorIndexType`: HNSW, HNSW_SQ, HNSW_BQ, IVF, IVF_FLAT, IVF_PQ
- `DistanceType`: L2, InnerProduct, Cosine
- `RemDbError` / `Result<T>`: Error handling (thiserror)
- `RecordStatus`: Active, Deleted

### Core Traits

- `SecondaryIndex`: Interface for all secondary indexes (BTree, TTree, HNSW, IVF)
- `DdlExecutor`: Runtime DDL operations (create_table, drop_table, alter_table)
- `Platform`: Abstracted OS primitives (spinlock, memcpy, file I/O, timers)
- `MemoryAllocator`: Custom memory allocation interface

### Three Ways to Define Tables

1. **Macro-based**: `remdb::table!` macro for compile-time definitions
2. **Derive macro**: `#[derive(MemdbTable)]` with inline DDL or external file
3. **Runtime DDL**: `DdlExecutor` trait with `create_table()` or SQL

## Feature Flags (remdb)

| Feature | Description |
|---------|-------------|
| `std` | Standard library support (default) |
| `posix` | POSIX platform support (default, implies `std`) |
| `baremetal` | no_std bare metal (uses `heapless`, no `std` or `posix`) |
| `pubsub` | UDP-based pub/sub messaging (implies `std`) |
| `ha` | High availability (implies `pubsub`) |
| `c-api` | C language API |
| `log` | Logging support |
| `wal-compression-lz4` | LZ4 WAL compression |
| `wal-compression-zstd` | Zstd WAL compression |
| `model-runtime` | ONNX model inference (ort, ndarray, tokio) |
| `model-download` | Model download via HTTP (reqwest) |

Default features: `std`, `posix`, `ha`, `pubsub`, `c-api`, `log`

## Configuration

Server uses TOML config files (`remdb-master.toml`, `remdb-slave.toml`). Key sections:

- **JDBC**: port, max_connections, timeout, auth (username/password_hash)
- **WAL**: log_path, log_mode (sync/async), checkpoint_interval, file_size_limit
- **HA**: role (master/slave), replication_mode (sync/async), heartbeat, failure_detection
- **PubSub**: udp_bind_address, heartbeat_interval, retransmission_timeout
- **Snapshot**: snapshot_dir, snapshot_interval, snapshot_type, max_incremental_snapshots

## Key Design Decisions

- **Global state**: The database instance, memory allocator, and HA manager are all global statics (`init_global_db`, `init_global_allocator`). This is why tests must run single-threaded with `serial_test`.
- **Memory management**: A fixed-size memory pool is allocated at startup via `Vec<u8>` + `forget()`, then managed by the custom allocator. No heap allocation after init.
- **Zero-copy**: `RecordRef` provides zero-copy read views into table memory. The network layer has a `zero_copy_transport` module.
- **Platform abstraction**: The `Platform` trait allows remdb to run on bare metal (no_std) or POSIX. The server uses the POSIX implementation.
- **Error handling**: `RemDbError` in remdb (thiserror), `ServerError` in remdb-server (thiserror).

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