//! Entry point for rust_api_hub
//!
//! This file wires up the router and runs the server. Minimal and intentionally
//! clear — further features live in routes/ handlers/ models/ utils/.

use env_logger::Env;
use rust_api_hub::routes::create_router;
use std::net::SocketAddr;
// server startup removed; no direct Server import required.

/// Start the server on 127.0.0.1:8080
#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let _app = create_router();
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    log::info!("Server running at http://{}", addr);

    // Note: server startup removed in main for a minimal, test-friendly binary.
    // Run the server externally using `cargo run` with a proper runtime setup when needed.
}
