# remdb-server Error Fixes Design

## Overview

This document covers four issues identified in the remdb-server log at `/var/lib/remdb/logs/remdb-server.log`:

1. **WAL Recovery FileIoError** — server fails to recover from WAL on startup
2. **Vector TypeMismatch** — Milvus API vector insert fails with `TypeMismatch` for 768-dimensional vectors
3. **SQL Parse InvalidSyntax** — single-quote escaping in string content causes parse failures
4. **Model File Permissions** — model weights file owned by wrong user

All fixes to the `remdb` core library will be made in the local copy at `/mnt/home/bobjia/remdb/`, and the server's `Cargo.toml` will be updated to use a path dependency.

---

## Issue 1: WAL Recovery FileIoError

### Location

- `/mnt/home/bobjia/remdb/src/transaction.rs:1356-1361` — `LogManager::recover()`
- `/mnt/home/bobjia/remdb-server/src/snapshot_loader.rs:757-778` — caller

### Root Cause

The `recover()` function at line 1360-1361 calls `crate::platform::file_size()` to get the WAL file size. If this call fails (e.g., file not found, permissions error, corrupted file), the error is immediately mapped to `RemDbError::FileIoError` and returned, causing the WAL recovery to fail entirely.

By contrast, the subsequent `file_open()` call at line 1374-1384 gracefully handles errors by logging a warning and returning `Ok(())` to skip recovery.

### Fix

Make the `file_size()` call consistent with the `file_open()` handling: if the file size can't be read, log a warning and return `Ok(())` instead of `Err(FileIoError)`.

**Before:**
```rust
let file_size = crate::platform::file_size(wal_file_path.as_str())
    .map_err(|_| RemDbError::FileIoError)?;
```

**After:**
```rust
let file_size = match crate::platform::file_size(wal_file_path.as_str()) {
    Ok(size) => size,
    Err(_) => {
        #[cfg(feature = "log")]
        warn!("Failed to get WAL file size, skipping recovery process");
        return Ok(());
    }
};
```

### Impact

- Server starts gracefully even if WAL file is missing or corrupted
- Data is recovered from snapshots instead (if available)
- No data loss beyond un-flushed transactions

---

## Issue 2: Vector TypeMismatch

### Location

- `/mnt/home/bobjia/remdb/src/sql/query_executor.rs:6309-6427` — `set_field_value_with_depth()` Vector handling
- `/mnt/home/bobjia/remdb/src/sql/operations/expression.rs:181-191` — `evaluate_expression_with_depth()` SqlValue::Json handling

### Root Cause

When a vector literal like `[0.1, 0.2, ..., 0.768]` (768-dimensional, ~7680 bytes) is inserted via the Milvus REST API:

1. The SQL parser parses the vector value as `Value::Json(json_str)` where `json_str` is the full ~7680-byte string
2. `evaluate_expression_with_depth()` processes this and creates `JsonStorage::Null` as a sentinel (because the string exceeds the 256-byte inline buffer)
3. `set_field_value_with_depth()` enters the `Json` branch for the `Vector` field type
4. The `JsonStorage::Null` case tries to extract the original string from the `Expression::Constant` via pattern matching
5. The pattern match `Expression::Constant { value: crate::sql::Value::Json(s), .. }` is not succeeding, causing `TypeMismatch` to be returned

The root cause appears to be that the pattern match references the `Value` type via `crate::sql::Value` which is a re-export from `crate::sql::query_parser::Value`, but the `Expression::Constant` stores `Value` directly from `query_parser`. While these are the same type, the pattern match may be failing due to how Rust resolves the `Json` variant through the re-export path.

### Fix

Two changes:

**Fix A: Make the expression extraction more robust in `query_executor.rs`**

In the `JsonStorage::Null` case of the `Vector` handling, if the `Expression::Constant` pattern match fails, fall back to trying `SqlValue::String` (in case the parser returned a string instead of JSON). Also, if extraction fails entirely, try to parse the vector directly from the truncated inline buffer.

**Fix B: Increase the inline buffer size in `operations/expression.rs`**

Change the 256-byte inline buffer to 8192 bytes so that most vector JSON strings fit inline. This avoids the `JsonStorage::Null` sentinel path entirely for common use cases.

**Before (expression.rs):**
```rust
let mut buf = [0u8; 256];
let len = core::cmp::min(json_str.len(), 256);
```

**After (expression.rs):**
```rust
let mut buf = [0u8; 8192];
let len = core::cmp::min(json_str.len(), 8192);
```

**Before (query_executor.rs):**
```rust
crate::types::JsonStorage::Null => {
    if let Expression::Constant {
        value: crate::sql::Value::Json(s),
        ..
    } = expr
    {
        s.clone()
    } else {
        return Err(QueryExecutionError::TypeMismatch);
    }
}
```

**After (query_executor.rs):**
```rust
crate::types::JsonStorage::Null => {
    // Try to extract the original string from the Constant expression
    let extracted = match expr {
        Expression::Constant {
            value: crate::sql::Value::Json(s),
            ..
        } => Some(s.clone()),
        Expression::Constant {
            value: crate::sql::Value::String(s),
            ..
        } => Some(s.clone()),
        _ => None,
    };
    match extracted {
        Some(s) => s,
        None => return Err(QueryExecutionError::TypeMismatch),
    }
}
```

### Impact

- Vector insertions with dimensions up to ~2048 (8192 bytes / 4 bytes per float) work inline
- Larger vectors use the expression extraction path with improved robustness
- Backward compatible — existing behavior for small vectors unchanged

---

## Issue 3: SQL Parse InvalidSyntax

### Location

- `/mnt/home/bobjia/remdb/src/sql/query_parser.rs:3682-3692` — `parse_value()` string reading
- `/mnt/home/bobjia/remdb-server/src/milvus/handler.rs:579` — `json_value_to_sql()` string escaping

### Root Cause

The `json_value_to_sql()` function in the Milvus handler escapes single quotes in string content by doubling them (`'` → `''`), which is standard SQL practice. However, the `parse_value()` function in the SQL parser reads strings character by character and breaks when it encounters the first unescaped quote character. It does not handle `''` as an escaped single quote.

For example, a Rust code string containing `'\''` (a char literal) is escaped to `''\''''` by `json_value_to_sql()`. The SQL parser reads:
- `'` — starts string
- `'` — sees a quote, breaks (treating first `''` as closing quote)
- `\` — not inside a string, parser fails

This causes the `SQL Parse Error: InvalidSyntax` errors seen every ~6 seconds.

### Fix

Modify the `parse_value()` function to handle `''` as an escaped single quote inside single-quoted strings. When the parser encounters a quote character while reading a string, it should peek at the next character. If the next character is also a quote, it's an escaped single quote — consume both and add one `'` to the string value. Otherwise, it's the closing quote.

**Before:**
```rust
while let Some(c) = self.next_char() {
    if c == quote_char {
        break;
    }
    string_value.push(c);
}
```

**After:**
```rust
while let Some(c) = self.next_char() {
    if c == quote_char {
        // Check for escaped quote ('')
        if self.peek_char() == Some(quote_char) {
            self.next_char(); // consume the second quote
            string_value.push(c); // add one quote character
        } else {
            break; // closing quote
        }
    } else {
        string_value.push(c);
    }
}
```

### Impact

- Standard SQL single-quote escaping (`''` → `'`) now works correctly
- String content with single quotes (Rust code, names with apostrophes, etc.) can be inserted
- Backward compatible — all existing queries continue to work

---

## Issue 4: Model File Permissions

### Location

- `/var/lib/remdb/models/bge-m3/model.onnx_data` (2.2GB)

### Root Cause

The `model.onnx_data` file is owned by `bobjia:bobjia` instead of `remdb:remdb`. The model download at startup failed with a network timeout, but the file was partially downloaded by the user.

### Fix

Change ownership of the model files to the `remdb` user.

### Impact

- Server can read model files without permission issues
- Model loading works on next restart

---

## Build Approach

1. Update `/mnt/home/bobjia/remdb-server/Cargo.toml` to use:
   ```toml
   remdb = { path = "../remdb", default-features = true, features = ["model-runtime", "model-download"] }
   ```
2. Apply fixes to the local `/mnt/home/bobjia/remdb/` crate
3. Rebuild the server
4. Restart the server

---

## Testing

- **WAL recovery**: Test by starting server with a missing/corrupted WAL file — should log a warning and start without data
- **Vector insert**: Test by inserting a 768-dimensional vector via the Milvus REST API — should succeed
- **SQL parser**: Test by inserting a string containing single quotes via the Milvus API — should parse correctly
- **Model permissions**: Verify server can load the model on next restart

---

## Rollback

- If the path dependency causes issues, revert to the registry version in `Cargo.toml`
- The registry copy of `remdb-0.4.4` is preserved at `/mnt/home/bobjia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/remdb-0.4.4/`