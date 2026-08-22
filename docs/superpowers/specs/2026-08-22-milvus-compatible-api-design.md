# Milvus-Compatible RESTful API Design

> **Date:** 2026-08-22
> **Status:** Draft
> **Target Milvus Version:** v2.3+ (`/v2/vectordb/*` path scheme)

## 1. Overview

Provide a Milvus-compatible RESTful API layer on top of remdb-server, allowing
any Milvus SDK or HTTP client to connect to remdb as if it were a Milvus
instance. This enables drop-in replacement of Milvus with remdb for
applications that use Milvus's RESTful API.

### Goals

- Wire-compatible with Milvus v2.3+ RESTful API for core operations
- Zero-code-change migration: existing Milvus HTTP clients work unmodified
- Leverage remdb's existing vector indexes (HNSW, IVF, IVF_PQ, IVF_FLAT)
- API-Key authentication
- Minimal performance overhead over direct remdb API calls

### Non-Goals

- Full Milvus gRPC protocol compatibility (out of scope per decision)
- Partition management, replica management, load/release semantics
- Milvus's distributed sharding, load balancing, or vector log
- Flush/compaction semantics (remdb is in-memory; persistence is via WAL/snapshot)

## 2. Architecture

### Module Layout

```
src/milvus/                  # New module
├── mod.rs                   # Re-exports, module declarations
├── server.rs                # Warp HTTP server, route registration, startup
├── models.rs                # Request/response JSON types (serde)
├── handler.rs               # Route handlers — translate HTTP → remdb API
├── catalog.rs               # Collection catalog: maps names → schemas
├── converter.rs             # Milvus types ↔ remdb types, field conversion
├── auth.rs                  # API-Key Bearer-token authentication filter
└── error.rs                 # Error types → Milvus-format JSON error responses
```

### Integration Points

| Component | How It Integrates |
|-----------|-------------------|
| `AppContext` | The milvus server holds an `Arc<Mutex<&'static mut RemDb>>` reference, same as `JdbcServer` |
| `src/config/` | TOML config extended with `[milvus]` section (port, api_key, enabled) |
| `src/main.rs` | Conditional startup of the Milvus HTTP server alongside the JDBC server |
| `remdb::RemDb` | All backend operations call remdb's table/index APIs directly via `db.get_table()` |

### Data Flow

```
HTTP Request
  │
  ▼
warp::Filter chain
  │
  ├─ auth::verify_token()        ← API-Key check (reject with 401 if invalid)
  │
  ├─ handler::handle_*()         ← Parse JSON body, extract params
  │     │
  │     ├─ catalog::resolve()    ← Look up collection → get remdb table name
  │     │
  │     ├─ converter::to_remdb() ← Convert Milvus types to remdb types
  │     │
  │     ├─ remdb API calls        ← Direct table/index operations
  │     │     ├─ table.insert() / table.delete() / table.update()
  │     │     ├─ index.search()   ← HNSW/IVF search
  │     │     └─ table.scan()     ← Filtered queries
  │     │
  │     └─ converter::to_milvus() ← Convert remdb results to Milvus JSON
  │
  ▼
JSON Response (Milvus format)
```

## 3. Collection Catalog

### Design: Logical Namespace Layer

Each Milvus **Collection** is stored as a logical entry in a system catalog
table, backed by a dedicated remdb **Table** for its data.

#### System Catalog Table: `_milvus_catalog`

| Field | Type | Description |
|-------|------|-------------|
| `collection_id` | Integer (PK, auto-increment) | Internal unique ID |
| `collection_name` | Text (unique) | Milvus collection name |
| `description` | Text | Optional description |
| `schema_json` | JSON | Full schema definition (fields, types, dims) |
| `primary_field` | Text | Name of the primary key field |
| `vector_field` | Text | Name of the vector field |
| `auto_id` | Boolean | Whether IDs are auto-generated |
| `dimension` | Integer | Vector dimension |
| `metric_type` | Text | L2 / IP / COSINE |
| `index_type` | Text | HNSW / IVF / IVF_FLAT / IVF_PQ |
| `index_params` | JSON | Index-specific parameters (M, efConstruction, nlist, etc.) |
| `remdb_table_name` | Text | Actual remdb table that stores the data |
| `created_at` | Integer | Unix timestamp |
| `row_count` | Integer | Approximate row count |

#### Data Table Naming Convention

Each collection gets a remdb table named `_milvus_coll_{collection_id}`. This
avoids naming collisions and makes the catalog the single source of truth.

#### Create Collection Flow

1. Validate schema (must have a primary key field, exactly one vector field,
   valid dimension, valid metric type)
2. Insert entry into `_milvus_catalog`
3. Create a remdb table `_milvus_coll_{id}` with columns mapped from the
   Milvus schema
4. Create a vector index on the vector field column
5. Return success

#### Drop Collection Flow

1. Look up collection in `_milvus_catalog`
2. Drop the remdb table `_milvus_coll_{id}`
3. Remove the catalog entry
4. Return success

## 4. API Endpoints

### 4.1 Collection Operations

#### `POST /v2/vectordb/collections/create`

Create a collection with a schema definition.

**Request:**
```json
{
  "collectionName": "my_collection",
  "description": "example collection",
  "schema": {
    "autoId": true,
    "fields": [
      {"name": "id", "type": "Int64", "isPrimary": true, "autoId": true},
      {"name": "vector", "type": "FloatVector", "params": {"dim": 128}},
      {"name": "metadata", "type": "VarChar", "params": {"max_length": 256}}
    ]
  },
  "indexParams": [
    {
      "fieldName": "vector",
      "indexName": "vector_idx",
      "metricType": "L2",
      "params": {"nlist": 128, "M": 16, "efConstruction": 200}
    }
  ]
}
```

**Response:**
```json
{"code": 0, "data": {"collectionName": "my_collection"}}
```

#### `POST /v2/vectordb/collections/drop`

**Request:** `{"collectionName": "my_collection"}`
**Response:** `{"code": 0, "message": "collection dropped"}`

#### `POST /v2/vectordb/collections/describe`

**Request:** `{"collectionName": "my_collection"}`
**Response:**
```json
{
  "code": 0,
  "data": {
    "collectionName": "my_collection",
    "description": "example collection",
    "schema": {
      "fields": [
        {"name": "id", "type": "Int64", "isPrimary": true, "autoId": true},
        {"name": "vector", "type": "FloatVector", "params": {"dim": 128}},
        {"name": "metadata", "type": "VarChar", "params": {"max_length": 256}}
      ]
    },
    "statistics": {"rowCount": 1000}
  }
}
```

#### `GET /v2/vectordb/collections/list`

**Response:**
```json
{
  "code": 0,
  "data": [
    {"collectionName": "my_collection", "description": "example collection"}
  ]
}
```

#### `POST /v2/vectordb/collections/has`

**Request:** `{"collectionName": "my_collection"}`
**Response:** `{"code": 0, "data": {"has": true}}`

### 4.2 Entity Operations

#### `POST /v2/vectordb/entities/insert`

**Request:**
```json
{
  "collectionName": "my_collection",
  "data": [
    {"vector": [0.12, 0.34, ...], "metadata": "doc1"},
    {"vector": [0.56, 0.78, ...], "metadata": "doc2"}
  ]
}
```

**Response:**
```json
{
  "code": 0,
  "data": {
    "insertCount": 2,
    "insertIds": [1, 2]
  }
}
```

**Implementation:**
1. Resolve collection from catalog → get remdb table
2. For each entity, convert fields to remdb `Value`s
3. Call `table.insert()` with the converted values
4. If `autoId` is true, generate IDs using the table's auto-increment
5. Return the inserted IDs

#### `POST /v2/vectordb/entities/upsert`

Same as insert but uses `table.update()` for existing records and `table.insert()`
for new ones.

#### `POST /v2/vectordb/entities/delete`

**Request:**
```json
{
  "collectionName": "my_collection",
  "filter": "id in [1, 2, 3]"
}
```

**Response:** `{"code": 0, "data": {"deleteCount": 3}}`

**Implementation:**
1. Parse the filter expression (Milvus filter grammar — subset: `id in [...]`,
   `id == x`, `field == val`)
2. Call `table.delete()` for each matching record
3. Return count of deleted records

#### `POST /v2/vectordb/entities/get`

**Request:**
```json
{
  "collectionName": "my_collection",
  "id": 42,
  "outputFields": ["id", "vector", "metadata"]
}
```

**Response:**
```json
{
  "code": 0,
  "data": {"id": 42, "vector": [0.12, ...], "metadata": "doc1"}
}
```

#### `POST /v2/vectordb/entities/query`

**Request:**
```json
{
  "collectionName": "my_collection",
  "filter": "metadata like 'doc%'",
  "outputFields": ["id", "metadata"],
  "limit": 10,
  "offset": 0
}
```

**Response:**
```json
{
  "code": 0,
  "data": [
    {"id": 1, "metadata": "doc1"},
    {"id": 2, "metadata": "doc2"}
  ]
}
```

**Implementation:**
1. Resolve collection → remdb table
2. Call `table.scan()` to iterate over records
3. Apply filter predicate (Milvus filter grammar → simple predicate evaluation)
4. Apply offset/limit
5. Return matching rows

### 4.3 Search (Vector Search)

#### `POST /v2/vectordb/entities/search`

**Request:**
```json
{
  "collectionName": "my_collection",
  "vector": [0.12, 0.34, ...],
  "annsField": "vector",
  "limit": 5,
  "offset": 0,
  "outputFields": ["id", "metadata"],
  "filter": "",
  "params": {
    "ef": 64,
    "nprobe": 10
  }
}
```

**Response:**
```json
{
  "code": 0,
  "data": [
    {"id": 1, "distance": 0.95, "entity": {"id": 1, "metadata": "doc1"}},
    {"id": 5, "distance": 0.87, "entity": {"id": 5, "metadata": "doc5"}}
  ]
}
```

**Implementation:**
1. Resolve collection → remdb table
2. Get the vector index (HNSW/IVF) from the table
3. Call `index.search(query_vector, k)` with the vector
4. Map results back to record IDs and retrieve full entities
5. Apply optional filter (post-filtering for now)
6. Apply offset (skip N results from the top-k)
7. Format response with distance + entity

**Search parameter mapping:**

| Milvus param | remdb equivalent | Notes |
|-------------|------------------|-------|
| `ef` | `VectorMetadata.hnsw_ef_search` | HNSW search width |
| `nprobe` | `IVFIndex.nprobe` | IVF cluster probe count |
| `metric_type` | `DistanceType` | Set at collection creation |
| `limit` | `k` parameter | Number of results |

### 4.4 Index Operations

#### `POST /v2/vectordb/indexes/create`

**Request:**
```json
{
  "collectionName": "my_collection",
  "indexName": "vector_idx",
  "fieldName": "vector",
  "metricType": "L2",
  "params": {
    "index_type": "HNSW",
    "M": 16,
    "efConstruction": 200
  }
}
```

**Response:** `{"code": 0, "data": {"indexName": "vector_idx"}}`

#### `POST /v2/vectordb/indexes/drop`

**Request:** `{"collectionName": "my_collection", "indexName": "vector_idx"}`
**Response:** `{"code": 0, "message": "index dropped"}`

## 5. Type Mapping

### Milvus → remdb Type Conversion

| Milvus Type | remdb `DataType` | remdb `Value` | Notes |
|-------------|------------------|---------------|-------|
| `Int64` | `DataType::Integer` | `Value::Int(i64)` | Primary key |
| `Float` | `DataType::Real` | `Value::Real(f64)` | |
| `Bool` | `DataType::Boolean` | `Value::Bool(bool)` | |
| `VarChar` | `DataType::Text` | `Value::Text(String)` | |
| `FloatVector` | `DataType::Vector` | `Value::Vector(Vec<f32>)` | Dimension stored in table metadata |
| `JSON` | `DataType::JSON` | `Value::Json(String)` | |

### Milvus Metric Type → remdb DistanceType

| Milvus Metric | remdb `DistanceType` |
|---------------|---------------------|
| `L2` | `DistanceType::L2` |
| `IP` | `DistanceType::InnerProduct` |
| `COSINE` | `DistanceType::Cosine` |

### Milvus Index Type → remdb VectorIndexType

| Milvus Index | remdb `VectorIndexType` |
|-------------|------------------------|
| `HNSW` | `VectorIndexType::HNSW` |
| `IVF_FLAT` | `VectorIndexType::IVF` |
| `IVF_PQ` | `VectorIndexType::IVF_PQ` |
| `BIN_IVF_FLAT` | `VectorIndexType::IVF` (binary) |
| `DISKANN` | Not supported (in-memory only) |

## 6. Error Handling

### Error Codes

| HTTP Status | Milvus Code | Message |
|-------------|-------------|---------|
| 200 | 0 | Success |
| 400 | 1001 | Collection not found |
| 400 | 1002 | Invalid schema |
| 400 | 1003 | Invalid dimension |
| 400 | 1004 | Invalid metric type |
| 400 | 1005 | Invalid index type |
| 400 | 1006 | Invalid field name |
| 400 | 1007 | Duplicate collection name |
| 400 | 1008 | Insert failed |
| 400 | 1009 | Search failed |
| 401 | 2001 | Authentication failed |
| 500 | 9999 | Internal server error |

### Error Response Format

```json
{
  "code": 1001,
  "message": "collection 'my_collection' not found"
}
```

## 7. Configuration

### TOML Config Extension (`remdb-master.toml`)

```toml
[milvus]
enabled = true
port = 19530
api_key = "your-api-key-here"
# Optional: restrict to specific interfaces
# bind_address = "0.0.0.0"
```

### CLI Args

```
--milvus-port <PORT>     Milvus RESTful API port (default: 19530)
--milvus-api-key <KEY>   API key for authentication
```

## 8. Authentication

### API-Key Middleware

The `auth.rs` module implements a warp `Filter` that:

1. Extracts the `Authorization` header from each request
2. Parses the `Bearer <token>` scheme
3. Computes SHA-256 hash of the provided token
4. Compares against the configured API-Key hash
5. Rejects with HTTP 401 if mismatch

The API-Key is stored as a SHA-256 hash in the config file (never plaintext).

## 9. Startup Integration

### Changes to `main.rs`

```rust
// In main(), after initializing the database:

if config.milvus.enabled {
    let milvus_server = MilvusServer::new(
        context.db_clone(),
        config.milvus.port,
        config.milvus.api_key,
    );
    tokio::spawn(async move {
        milvus_server.start().await;
    });
}
```

### Changes to `config/`

- Add `MilvusConfig` struct to `Config`
- Add `[milvus]` section parsing to `loader.rs`
- Add `--milvus-port` and `--milvus-api-key` CLI args

## 10. Testing Strategy

### Unit Tests

| Test | What It Covers |
|------|---------------|
| `test_type_conversion` | Milvus → remdb type mapping for all supported types |
| `test_metric_conversion` | L2/IP/COSINE → DistanceType |
| `test_index_type_conversion` | HNSW/IVF/IVF_PQ → VectorIndexType |
| `test_auth_valid_token` | Valid API-Key passes auth filter |
| `test_auth_invalid_token` | Invalid API-Key returns 401 |
| `test_catalog_insert` | Collection entry in `_milvus_catalog` |
| `test_catalog_resolve` | Name → remdb table resolution |
| `test_error_format` | Error types produce correct JSON |

### Integration Tests

| Test | What It Covers |
|------|---------------|
| `test_create_drop_collection` | Full create → describe → drop cycle |
| `test_insert_search` | Insert vectors, search returns them |
| `test_insert_get` | Insert → get by ID |
| `test_search_hnsw` | HNSW search returns correct nearest neighbors |
| `test_search_ivf` | IVF search returns correct nearest neighbors |
| `test_search_with_filter` | Search + post-filter |
| `test_upsert` | Upsert updates existing record |
| `test_delete` | Delete removes record, search excludes it |
| `test_auto_id` | Auto-generated IDs work |
| `test_collection_list` | Multiple collections list correctly |
| `test_error_cases` | Missing collection, bad schema, etc. |

### Testing Approach

1. **Unit tests**: Pure function tests in each module, no DB needed
2. **Integration tests**: Start a real remdb instance in test mode, run HTTP
   requests against the Milvus API, validate responses
3. **Milvus SDK compatibility test**: Use a Python script with the official
   Milvus Python SDK (`pymilvus`) configured to use HTTP (gRPC disabled) to
   verify wire compatibility

## 11. Dependencies

All required dependencies already exist in the workspace:

| Crate | Version | Used For |
|-------|---------|----------|
| `warp` | 0.3.6 | HTTP server, routing, filters |
| `serde` / `serde_json` | 1.0 | JSON serialization/deserialization |
| `tokio` | 1.37 | Async runtime |
| `sha2` | (already in deps) | API-Key hashing |
| `hex` | (already in deps) | Hash encoding |

No new external dependencies are needed.

## 12. Milvus Filter Grammar (Subset)

For the `filter` parameter in `query` and `delete` operations, we support a subset
of Milvus's filter expression grammar:

```
expr     → term ("&&" term)*
term     → comparison | "id in [" int_list "]"
comparison → field "==" value | field "!=" value | field ">" value
           | field "<" value | field ">=" value | field "<=" value
           | field "like" pattern
value    → number | string | bool
```

This is implemented as a simple recursive-descent parser in `converter.rs`.

## 13. Future Considerations

### Post-MVP Enhancements

- **gRPC support**: Add tonic-based gRPC server if needed
- **Full filter grammar**: Support all Milvus filter expressions
- **Partition support**: Add Milvus partition operations
- **Alias support**: Collection aliases
- **Consistency levels**: Strong/eventual consistency settings
- **Bulk insert**: Optimized batch insert path
- **Pre-filtering**: Push scalar filters into vector search (vs. post-filter)
- **Prometheus metrics**: Expose Milvus API metrics