//! Shared types for the GUI frontend
//!
//! This module contains type definitions that are shared between the backend
//! and frontend, ensuring consistency in data structures.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

/// Represents a position on the game grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Movement directions for snakes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Represents a player's snake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snake {
    pub id: Uuid,
    pub player_name: String,
    pub body: VecDeque<Position>,
    pub length: usize,
    pub is_alive: bool,
    pub color_index: usize,
    pub last_direction: Option<Direction>,
}

impl Snake {
    pub fn head(&self) -> Option<Position> {
        self.body.front().copied()
    }

    pub fn tail(&self) -> Vec<Position> {
        self.body.iter().skip(1).copied().collect()
    }
}

/// Represents a piece of fruit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fruit {
    pub position: Position,
    pub spawn_tick: u64,
}

/// Current state of the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub snakes: HashMap<Uuid, Snake>,
    pub fruits: Vec<Fruit>,
    pub tick: u64,
    pub is_running: bool,
    pub winner: Option<Uuid>,
    pub grid_width: i32,
    pub grid_height: i32,
}

/// Player information in lobby
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayer {
    pub id: Uuid,
    pub name: String,
    pub color_index: usize,
    pub is_ready: bool,
}

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    JoinLobby { player_name: String },
    SubmitMove { direction: Direction },
    StartGame,
    Ping,
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    LobbyJoined {
        player_id: Uuid,
        player_name: String,
    },
    LobbyState {
        players: Vec<LobbyPlayer>,
    },
    GameStarted {
        game_state: GameState,
        your_snake_id: Uuid,
    },
    GameUpdate {
        game_state: GameState,
    },
    MoveRequest {
        valid_directions: Vec<Direction>,
        time_limit_ms: u64,
    },
    GameEnded {
        winner: Option<LobbyPlayer>,
        final_state: GameState,
    },
    Error {
        message: String,
    },
    Pong,
}

/// Game constants (mirrored from backend)
pub mod constants {
    pub const GRID_WIDTH: usize = 50;
    pub const GRID_HEIGHT: usize = 50;
    pub const CELL_SIZE_PX: u32 = 12;
    pub const WINNING_SNAKE_LENGTH: usize = 50;
    pub const MAX_PLAYERS: usize = 8;
    pub const MIN_PLAYERS: usize = 2;

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

    pub const DEAD_SNAKE_ALPHA: f32 = 0.5;
    pub const FRUIT_COLOR: &str = "#FF1493";
    pub const GRID_BACKGROUND_COLOR: &str = "#2C3E50";
    pub const GRID_LINE_COLOR: &str = "#34495E";
}

/// Utility functions for WASM/JS interop
#[wasm_bindgen]
pub struct JsGameState {
    inner: GameState,
}

#[wasm_bindgen]
impl JsGameState {
    #[wasm_bindgen(getter)]
    pub fn tick(&self) -> u64 {
        self.inner.tick
    }

    #[wasm_bindgen(getter)]
    pub fn is_running(&self) -> bool {
        self.inner.is_running
    }

    #[wasm_bindgen(getter)]
    pub fn grid_width(&self) -> i32 {
        self.inner.grid_width
    }

    #[wasm_bindgen(getter)]
    pub fn grid_height(&self) -> i32 {
        self.inner.grid_height
    }
}

impl From<GameState> for JsGameState {
    fn from(state: GameState) -> Self {
        Self { inner: state }
    }
}
