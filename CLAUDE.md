# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

remdb-server is a high-performance in-memory database system with multiple components:
- **remdb**: Core embedded database library (Rust, supports `no_std`)
- **remdb-server**: Network server with JDBC protocol, PubSub, and High Availability
- **remdb-python**: Python bindings with NumPy/Pandas integration
- **jdbc-driver**: Java JDBC driver for connecting to remdb-server

Key features: ACID transactions, SQL support, time-series data, vector database capabilities, master-slave replication.

## Build Commands

### Rust (remdb and remdb-server)

```bash
# Build the server
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Run tests for specific crate
cargo test --lib -p remdb

# Run a single test
cargo test --lib test_name

# Format code
cargo fmt

# Run clippy (warnings as errors)
cargo clippy -- -D warnings

# Build for no_std/baremetal (remdb only)
cargo build --no-default-features --features=baremetal -p remdb
```

### Python Bindings

```bash
cd remdb-python

# Install in development mode
pip install -e .

# Build C extension (Windows)
python setup.py build_ext --inplace

# Run tests
python -m pytest tests/

# Run tests with verbose output
python run_tests.py --verbose

# Run only unit tests
python run_tests.py --unit
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
# Run with default config (remdb-master.toml)
cargo run --bin remdb-server

# Run with specific config
cargo run --bin remdb-server -- --config remdb-slave.toml

# Run with debug mode
cargo run --bin remdb-server -- --debug

# Run benchmark
cargo run --bin remdb-server -- benchmark --test-type query --query-count 100000
```

## Feature Flags (remdb)

| Feature | Description |
|---------|-------------|
| `std` | Standard library support |
| `posix` | POSIX platform support |
| `baremetal` | Bare metal/no_std support |
| `pubsub` | UDP-based pub/sub messaging |
| `ha` | High availability (master-slave replication) |
| `c-api` | C language API |
| `log` | Logging support |

Default features: `std`, `posix`, `ha`, `pubsub`, `c-api`, `log`

## Architecture

### Server Structure (src/)

```
src/
├── main.rs           # Server entry point, config parsing, initialization
├── lib.rs            # Library exports
├── jdbc_server.rs    # JDBC protocol server
├── handler/          # Request handlers (JDBC, health monitor)
├── sql_engine/       # SQL execution (SELECT, INSERT, UPDATE, DELETE, DDL)
├── config/           # Configuration loading and parsing
├── bootstrap/        # Platform and service initialization
├── pool/             # Connection pooling
├── network/          # Zero-copy network transport
├── scheduler/        # Checkpoint and snapshot scheduling
└── benchmark/        # Built-in benchmark tools
```

### Core Database (remdb/src/)

```
remdb/src/
├── lib.rs            # Main library entry
├── types.rs          # Core type definitions (DataType, Value)
├── table.rs          # MemoryTable - core in-memory storage
├── index.rs          # Index system (Hash, BTree, HNSW, IVF)
├── transaction.rs    # ACID transaction support
├── sql/              # SQL parser and executor
│   ├── query_parser.rs
│   ├── query_executor.rs
│   └── operations/   # DDL, DML, SELECT operations
├── time_series/      # Time-series storage and compression
├── ha/               # High availability and replication
├── pubsub/           # UDP-based pub/sub
├── memory/           # Custom allocator for embedded systems
└── platform/         # Platform abstraction (POSIX/baremetal)
```

### Three Ways to Define Tables

1. **Macro-based**: `remdb::table!` macro for compile-time definitions
2. **Derive macro**: `#[derive(MemdbTable)]` with inline DDL or external file
3. **Runtime DDL**: `DdlExecutor` trait with `create_table()` or SQL

## Key Types

- `RemDb`: Main database instance
- `MemoryTable`: Table storage with row-based layout
- `Value`: Dynamic value type for SQL results
- `DataType`: Schema type definitions
- `TableDef`: Table schema definition
- `DbConfig`: Database configuration

## Error Handling

- Use `Result<T, RemDbError>` for fallible operations
- Use `?` for error propagation, avoid `.unwrap()` in library code
- Server uses `thiserror` for error definitions

## Testing

- Rust tests use `serial_test` crate due to shared global state
- Tests run single-threaded with 16MB stack size
- Python tests support both unittest and pytest frameworks

## Configuration

Server configuration is TOML-based (`remdb-master.toml`, `remdb-slave.toml`):
- JDBC settings: port, max connections, authentication
- WAL settings: log path, checkpoint interval, compression
- HA settings: role (master/slave), replication mode, heartbeat
- PubSub settings: UDP bind address, heartbeat, retransmission

## Common Development Tasks

### Adding a New SQL Function
1. Add implementation in `remdb/src/sql/functions/`
2. Register in `remdb/src/sql/functions/mod.rs`
3. Add parser support in `query_parser.rs` if needed
4. Add tests in `remdb/tests/`

### Adding a New Index Type
1. Implement `SecondaryIndex` trait in `remdb/src/index.rs`
2. Add to `IndexType` enum in `remdb/src/types.rs`
3. Update index builder in `remdb/src/index/builder.rs`

### Adding a New Data Type
1. Add to `DataType` enum in `remdb/src/types.rs`
2. Implement storage in `MemoryTable`
3. Update SQL parser and executor
4. Add `Value` variant if needed