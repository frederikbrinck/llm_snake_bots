//! Game constants for the multiplayer snake game
//!
//! This module contains all configurable constants for the game,
//! making it easy to modify game parameters without changing core logic.

/// Grid dimensions
pub const GRID_WIDTH: usize = 50;
pub const GRID_HEIGHT: usize = 50;

/// Snake game rules
pub const WINNING_SNAKE_LENGTH: usize = 300;
pub const INITIAL_SNAKE_LENGTH: usize = 1;

/// Fruit spawning rules
pub const FRUIT_SPAWN_DELAY_TICKS: u32 = 5;

/// Game timing
pub const GAME_TICK_DURATION_MS: u64 = 200;

/// Server configuration
pub const SERVER_HOST: &str = "0.0.0.0";
pub const SERVER_PORT: u16 = 3000;

/// WebSocket endpoints
pub const LOBBY_ENDPOINT: &str = "/lobby";
pub const GUI_ENDPOINT: &str = "/gui";

/// Game state limits
pub const MAX_PLAYERS: usize = 8;
pub const MIN_PLAYERS: usize = 2;

/// Colors for snake visualization (HTML color codes)
pub const SNAKE_COLORS: [&str; 8] = [
    "#FF6B6B", // Red
    "#4ECDC4", // Teal
    "#45B7D1", // Blue
    "#96CEB4", // Green
    "#FFEAA7", // Yellow
    "#DDA0DD", // Plum
    "#FFB347", // Orange
    "#87CEEB", // Sky Blue
];

/// Dead snake color modifier (makes colors more gray/transparent)
pub const DEAD_SNAKE_ALPHA: f32 = 0.5;

/// Fruit color
pub const FRUIT_COLOR: &str = "#FF1493"; // Deep Pink

/// Grid styling
pub const GRID_BACKGROUND_COLOR: &str = "#2C3E50";
pub const GRID_LINE_COLOR: &str = "#34495E";
pub const CELL_SIZE_PX: u32 = 12;

/// WebSocket message size limits
pub const MAX_MESSAGE_SIZE: usize = 1024 * 16; // 16KB
pub const MAX_FRAME_SIZE: usize = 1024 * 16; // 16KB

/// Game timing constraints
pub const MOVE_TIMEOUT_MS: u64 = 5000; // 5 seconds to make a move
pub const LOBBY_TIMEOUT_MS: u64 = 300000; // 5 minutes lobby timeout

/// Debug settings
pub const ENABLE_DEBUG_LOGGING: bool = cfg!(debug_assertions);
