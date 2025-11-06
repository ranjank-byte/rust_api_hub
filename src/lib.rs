//! Library root for rust_api_hub used by integration tests.
//!
//! This file re-exports the internal modules as a library API so integration
//! tests under `tests/` can import `rust_api_hub` and exercise the public
//! functions and types.

pub mod handlers;
pub mod models;
pub mod routes;
pub mod utils;

// Optionally re-export commonly used types here in the future.
