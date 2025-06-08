//! Game API types and data structures
//!
//! This module defines all the data structures used for the multiplayer snake game API.
//! All types are serializable with serde for JSON communication over WebSocket.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a position on the game grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Move position in the given direction, wrapping around grid boundaries
    pub fn move_in_direction(
        &self,
        direction: Direction,
        grid_width: i32,
        grid_height: i32,
    ) -> Position {
        let mut new_x = self.x;
        let mut new_y = self.y;

        match direction {
            Direction::Up => new_y -= 1,
            Direction::Down => new_y += 1,
            Direction::Left => new_x -= 1,
            Direction::Right => new_x += 1,
        }

        // Handle wrapping around boundaries
        if new_x < 0 {
            new_x = grid_width - 1;
        } else if new_x >= grid_width {
            new_x = 0;
        }

        if new_y < 0 {
            new_y = grid_height - 1;
        } else if new_y >= grid_height {
            new_y = 0;
        }

        Position::new(new_x, new_y)
    }
}

/// Movement directions for snakes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Get the opposite direction
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    /// Get all possible directions
    pub fn all() -> [Direction; 4] {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
    }
}

/// Represents a player's snake
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Snake {
    /// Unique identifier for the snake
    pub id: Uuid,
    /// Player name
    pub player_name: String,
    /// Snake body positions (head is first, tail is last)
    pub body: VecDeque<Position>,
    /// Current length of the snake
    pub length: usize,
    /// Whether the snake is alive
    pub is_alive: bool,
    /// Color index for this snake (maps to SNAKE_COLORS)
    pub color_index: usize,
    /// Last direction moved (used to prevent moving backwards)
    pub last_direction: Option<Direction>,
}

impl Snake {
    pub fn new(
        id: Uuid,
        player_name: String,
        initial_position: Position,
        color_index: usize,
    ) -> Self {
        let mut body = VecDeque::new();
        body.push_back(initial_position);

        Self {
            id,
            player_name,
            body,
            length: crate::constants::INITIAL_SNAKE_LENGTH,
            is_alive: true,
            color_index,
            last_direction: None,
        }
    }

    /// Get the head position of the snake
    pub fn head(&self) -> Option<Position> {
        self.body.front().copied()
    }

    /// Get the tail position of the snake (excluding head)
    pub fn tail(&self) -> Vec<Position> {
        self.body.iter().skip(1).copied().collect()
    }

    /// Check if the snake contains a specific position
    pub fn contains_position(&self, pos: Position) -> bool {
        self.body.contains(&pos)
    }

    /// Get valid directions (cannot move backwards into tail)
    pub fn valid_directions(&self) -> Vec<Direction> {
        let mut valid = Direction::all().to_vec();

        // If snake has a tail and we know the last direction, remove opposite
        if self.body.len() > 1 {
            if let Some(last_dir) = self.last_direction {
                valid.retain(|&dir| dir != last_dir.opposite());
            }
        }

        valid
    }

    /// Move the snake in the given direction
    pub fn move_snake(
        &mut self,
        direction: Direction,
        grid_width: i32,
        grid_height: i32,
        grow: bool,
    ) {
        if let Some(head) = self.head() {
            let new_head = head.move_in_direction(direction, grid_width, grid_height);
            self.body.push_front(new_head);
            self.last_direction = Some(direction);

            if grow {
                self.length += 1;
            } else {
                // Remove tail if not growing
                if self.body.len() > self.length {
                    self.body.pop_back();
                }
            }
        }
    }

    /// Kill the snake
    pub fn kill(&mut self) {
        self.is_alive = false;
    }
}

/// Represents a piece of fruit
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Fruit {
    pub position: Position,
    pub spawn_tick: u64,
}

impl Fruit {
    pub fn new(position: Position, spawn_tick: u64) -> Self {
        Self {
            position,
            spawn_tick,
        }
    }
}

/// Current state of the game
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GameState {
    /// All snakes in the game
    pub snakes: HashMap<Uuid, Snake>,
    /// All fruits on the board
    pub fruits: Vec<Fruit>,
    /// Current game tick
    pub tick: u64,
    /// Whether the game is running
    pub is_running: bool,
    /// Winner of the game (if any)
    pub winner: Option<Uuid>,
    /// Grid dimensions
    pub grid_width: i32,
    pub grid_height: i32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            snakes: HashMap::new(),
            fruits: Vec::new(),
            tick: 0,
            is_running: false,
            winner: None,
            grid_width: crate::constants::GRID_WIDTH as i32,
            grid_height: crate::constants::GRID_HEIGHT as i32,
        }
    }

    /// Get all occupied positions on the grid
    pub fn occupied_positions(&self) -> Vec<Position> {
        let mut positions = Vec::new();

        // Add all snake body positions
        for snake in self.snakes.values() {
            positions.extend(snake.body.iter().copied());
        }

        // Add all fruit positions
        for fruit in &self.fruits {
            positions.push(fruit.position);
        }

        positions
    }

    /// Get empty positions on the grid
    pub fn empty_positions(&self) -> Vec<Position> {
        let occupied = self.occupied_positions();
        let mut empty = Vec::new();

        for x in 0..self.grid_width {
            for y in 0..self.grid_height {
                let pos = Position::new(x, y);
                if !occupied.contains(&pos) {
                    empty.push(pos);
                }
            }
        }

        empty
    }

    /// Check if the game is over
    pub fn is_game_over(&self) -> bool {
        let alive_snakes: Vec<_> = self.snakes.values().filter(|s| s.is_alive).collect();

        // Game over if only one snake alive or someone reached winning length
        alive_snakes.len() <= 1
            || alive_snakes
                .iter()
                .any(|s| s.length >= crate::constants::WINNING_SNAKE_LENGTH)
    }

    /// Get the winner of the game
    pub fn get_winner(&self) -> Option<Uuid> {
        let alive_snakes: Vec<_> = self.snakes.values().filter(|s| s.is_alive).collect();

        // Check for length winner first
        for snake in &alive_snakes {
            if snake.length >= crate::constants::WINNING_SNAKE_LENGTH {
                return Some(snake.id);
            }
        }

        // Check for last snake standing
        if alive_snakes.len() == 1 {
            return Some(alive_snakes[0].id);
        }

        None
    }
}

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Join the game lobby
    JoinLobby { player_name: String },
    /// Submit a move for the current tick
    SubmitMove { direction: Direction },
    /// Ready to start the game (from GUI)
    StartGame,
    /// Ping to keep connection alive
    Ping,
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Confirmation of joining lobby
    LobbyJoined {
        player_id: Uuid,
        player_name: String,
    },
    /// Current lobby state
    LobbyState { players: Vec<LobbyPlayer> },
    /// Game has started
    GameStarted {
        game_state: GameState,
        your_snake_id: Uuid,
    },
    /// Game state update
    GameUpdate { game_state: GameState },
    /// Request for next move
    MoveRequest {
        valid_directions: Vec<Direction>,
        time_limit_ms: u64,
    },
    /// Game ended
    GameEnded {
        winner: Option<LobbyPlayer>,
        final_state: GameState,
    },
    /// Error message
    Error { message: String },
    /// Pong response to ping
    Pong,
}

/// Player information in lobby
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LobbyPlayer {
    pub id: Uuid,
    pub name: String,
    pub color_index: usize,
    pub is_ready: bool,
}

/// Game room state for managing connections
#[derive(Debug)]
pub struct GameRoom {
    pub game_state: GameState,
    pub players: HashMap<Uuid, LobbyPlayer>,
    pub pending_moves: HashMap<Uuid, Direction>,
    pub fruit_spawn_counter: u32,
    pub move_deadline: Option<tokio::time::Instant>,
}

impl GameRoom {
    pub fn new() -> Self {
        Self {
            game_state: GameState::new(),
            players: HashMap::new(),
            pending_moves: HashMap::new(),
            fruit_spawn_counter: 0,
            move_deadline: None,
        }
    }

    /// Add a new player to the room
    pub fn add_player(&mut self, id: Uuid, name: String) -> Result<usize, String> {
        if self.players.len() >= crate::constants::MAX_PLAYERS {
            return Err("Room is full".to_string());
        }

        if self.players.values().any(|p| p.name == name) {
            return Err("Player name already taken".to_string());
        }

        let color_index = self.players.len();
        let player = LobbyPlayer {
            id,
            name,
            color_index,
            is_ready: false,
        };

        self.players.insert(id, player);
        Ok(color_index)
    }

    /// Remove a player from the room
    pub fn remove_player(&mut self, id: &Uuid) {
        self.players.remove(id);
        self.game_state.snakes.remove(id);
        self.pending_moves.remove(id);
    }

    /// Check if all players are ready to start
    pub fn can_start_game(&self) -> bool {
        self.players.len() >= crate::constants::MIN_PLAYERS
            && self.players.values().all(|p| p.is_ready)
    }

    /// Check if all alive players have submitted moves
    pub fn all_moves_submitted(&self) -> bool {
        let alive_players: Vec<_> = self
            .game_state
            .snakes
            .values()
            .filter(|s| s.is_alive)
            .map(|s| s.id)
            .collect();

        alive_players
            .iter()
            .all(|id| self.pending_moves.contains_key(id))
    }
}

/// WebSocket connection wrapper
#[derive(Debug)]
pub struct PlayerConnection {
    pub player_id: Uuid,
    pub sender: tokio::sync::mpsc::UnboundedSender<ServerMessage>,
}

/// Game events for internal communication
#[derive(Debug, Clone)]
pub enum GameEvent {
    PlayerJoined(Uuid, String),
    PlayerLeft(Uuid),
    GameStarted,
    MovesSubmitted,
    GameTick,
    GameEnded(Option<Uuid>),
}

/// Error types for the game
#[derive(Debug, thiserror::Error, ToSchema)]
pub enum GameError {
    #[error("Player not found: {0}")]
    PlayerNotFound(Uuid),

    #[error("Game not running")]
    GameNotRunning,

    #[error("Invalid move: {0}")]
    InvalidMove(String),

    #[error("Room full")]
    RoomFull,

    #[error("Name already taken: {0}")]
    NameTaken(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for game operations
pub type GameResult<T> = Result<T, GameError>;

/// Game statistics for monitoring and display
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GameStats {
    /// Current game tick number
    pub tick: u64,
    /// Number of snakes currently alive
    pub alive_snakes: usize,
    /// Total number of snakes in the game
    pub total_snakes: usize,
    /// Number of fruits currently on the board
    pub fruits_on_board: usize,
    /// Length of the longest snake
    pub longest_snake_length: usize,
    /// Whether the game is currently running
    pub is_running: bool,
    /// ID of the winner, if game has ended
    pub winner_id: Option<Uuid>,
}
