---
name: panic-free-policy
description: Enforced panic-free coding policy across the entire codebase
metadata:
  type: project
---

The project enforces a strict panic-free policy. No `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unreachable!()`, `unimplemented!()`, `assert!()`, or unchecked indexing/slicing is allowed anywhere. All errors must propagate via `Result` with `?`. This is documented in both `CLAUDE.md` files (root and `remdb/`) under a "Panic-Free Requirement" section.

**Why:** The database targets embedded systems (`no_std`) where panics mean unrecoverable crash. Even in the server, a panic in any request handler brings down the entire process. The previous phases of work reduced `.unwrap()` count from 257 to ~60; the policy now codifies that no new panics can be introduced.

**How to apply:** When adding or reviewing code, grep for `\.unwrap\(\)`, `\.expect\(`, `panic!`, `todo!`, `unreachable!`, `unimplemented!`, `assert!`, and `\[.\]` indexing patterns. Reject any that aren't provably infallible (and even then prefer `.get()` with explicit handling).