//! Main entry point for the multiplayer snake game backend
//!
//! This module sets up and starts the Axum web server with WebSocket support
//! for the multiplayer snake game.

mod constants;
mod docs;
mod game;
mod server;
mod types;

use server::start_server;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    info!("🐍 Starting Multiplayer Snake Game Server");
    info!(
        "Server will be available at http://{}:{}",
        constants::SERVER_HOST,
        constants::SERVER_PORT
    );
    info!("WebSocket endpoints:");
    info!(
        "  - Player lobby: ws://{}:{}{}",
        constants::SERVER_HOST,
        constants::SERVER_PORT,
        constants::LOBBY_ENDPOINT
    );
    info!(
        "  - GUI interface: ws://{}:{}{}",
        constants::SERVER_HOST,
        constants::SERVER_PORT,
        constants::GUI_ENDPOINT
    );

    // Start the server
    if let Err(e) = start_server().await {
        error!("Failed to start server: {}", e);
        std::process::exit(1);
    }
}
