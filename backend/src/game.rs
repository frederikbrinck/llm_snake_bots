//! Core game logic for the multiplayer snake game
//!
//! This module implements the main game mechanics including snake movement,
//! collision detection, fruit spawning, and game state management.

use crate::constants::*;
use crate::types::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Game engine that manages all game logic
pub struct GameEngine {
    /// Current game state
    pub state: GameState,
    /// Random number generator
    rng: StdRng,
    /// Tracks when fruits should spawn
    fruit_spawn_timer: HashMap<usize, u32>,
}

impl GameEngine {
    /// Create a new game engine
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            rng: StdRng::from_entropy(),
            fruit_spawn_timer: HashMap::new(),
        }
    }

    /// Initialize the game with players
    pub fn initialize_game(&mut self, players: &HashMap<Uuid, LobbyPlayer>) -> GameResult<()> {
        self.state.snakes.clear();
        self.state.fruits.clear();
        self.state.tick = 0;
        self.state.is_running = true;
        self.state.winner = None;

        // Place snakes at random positions
        let mut occupied_positions = HashSet::new();

        for player in players.values() {
            let position = self.find_random_empty_position(&occupied_positions)?;
            occupied_positions.insert(position);

            let snake = Snake::new(player.id, player.name.clone(), position, player.color_index);

            self.state.snakes.insert(player.id, snake);
        }

        // Initialize fruit spawning
        self.initialize_fruit_spawning();

        Ok(())
    }

    /// Initialize fruit spawning timers
    fn initialize_fruit_spawning(&mut self) {
        let player_count = self.state.snakes.len();
        let fruit_count = if player_count > 1 {
            player_count - 1
        } else {
            0
        };

        self.fruit_spawn_timer.clear();
        for i in 0..fruit_count {
            // Stagger initial fruit spawning
            self.fruit_spawn_timer.insert(i, i as u32);
        }
    }

    /// Find a random empty position on the grid
    fn find_random_empty_position(
        &mut self,
        additional_occupied: &HashSet<Position>,
    ) -> GameResult<Position> {
        let mut occupied = self.state.occupied_positions();
        occupied.extend(additional_occupied.iter().copied());

        let mut attempts = 0;
        let max_attempts = GRID_WIDTH * GRID_HEIGHT;

        loop {
            if attempts >= max_attempts {
                return Err(GameError::Internal(
                    "No empty positions available".to_string(),
                ));
            }

            let x = self.rng.gen_range(0..GRID_WIDTH as i32);
            let y = self.rng.gen_range(0..GRID_HEIGHT as i32);
            let position = Position::new(x, y);

            if !occupied.contains(&position) {
                return Ok(position);
            }

            attempts += 1;
        }
    }

    /// Process a game tick with player moves
    pub fn process_tick(&mut self, moves: HashMap<Uuid, Direction>) -> GameResult<()> {
        if !self.state.is_running {
            return Err(GameError::GameNotRunning);
        }

        // Move all snakes
        self.move_snakes(moves)?;

        // Handle collisions and deaths
        self.handle_collisions()?;

        // Handle fruit consumption
        self.handle_fruit_consumption()?;

        // Spawn new fruits
        self.spawn_fruits()?;

        // Check for game end conditions
        self.check_game_end()?;

        // Increment tick counter
        self.state.tick += 1;

        Ok(())
    }

    /// Move all snakes based on player input
    fn move_snakes(&mut self, moves: HashMap<Uuid, Direction>) -> GameResult<()> {
        let mut snakes_to_update = Vec::new();

        // Collect moves for alive snakes
        for (snake_id, snake) in &self.state.snakes {
            if snake.is_alive {
                if let Some(&direction) = moves.get(snake_id) {
                    // Validate move
                    let valid_directions = snake.valid_directions();
                    if !valid_directions.contains(&direction) {
                        // Invalid move - snake dies
                        snakes_to_update.push((*snake_id, None));
                        continue;
                    }
                    snakes_to_update.push((*snake_id, Some(direction)));
                } else {
                    // No move submitted - snake dies
                    snakes_to_update.push((*snake_id, None));
                }
            }
        }

        // Apply moves
        for (snake_id, direction_opt) in snakes_to_update {
            if let Some(snake) = self.state.snakes.get_mut(&snake_id) {
                match direction_opt {
                    Some(direction) => {
                        // Move the snake (will check for fruit consumption later)
                        snake.move_snake(
                            direction,
                            self.state.grid_width,
                            self.state.grid_height,
                            false, // We'll handle growth separately
                        );
                    }
                    None => {
                        // Kill snake for invalid/missing move
                        snake.kill();
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle all collision detection and deaths
    fn handle_collisions(&mut self) -> GameResult<()> {
        let mut snakes_to_kill = Vec::new();

        // Collect all head positions for collision detection
        let mut head_positions: HashMap<Position, Vec<Uuid>> = HashMap::new();
        for (snake_id, snake) in &self.state.snakes {
            if snake.is_alive {
                if let Some(head_pos) = snake.head() {
                    head_positions
                        .entry(head_pos)
                        .or_insert_with(Vec::new)
                        .push(*snake_id);
                }
            }
        }

        // Check for head-to-head collisions
        for (_pos, snake_ids) in &head_positions {
            if snake_ids.len() > 1 {
                // Multiple snakes moved to same position - all die
                snakes_to_kill.extend(snake_ids.iter().copied());
            }
        }

        // Check for head-to-body collisions
        for (snake_id, snake) in &self.state.snakes {
            if snake.is_alive && !snakes_to_kill.contains(&snake_id) {
                if let Some(head_pos) = snake.head() {
                    // Check collision with own tail
                    let tail_positions = snake.tail();
                    if tail_positions.contains(&head_pos) {
                        snakes_to_kill.push(*snake_id);
                        continue;
                    }

                    // Check collision with other snakes' bodies
                    for (other_id, other_snake) in &self.state.snakes {
                        if *other_id != *snake_id {
                            // Check collision with other snake's tail
                            if other_snake.tail().contains(&head_pos) {
                                snakes_to_kill.push(*snake_id);
                                break;
                            }
                            // Also check collision with other snake's head if they didn't move to same spot
                            if let Some(other_head) = other_snake.head() {
                                if other_head == head_pos
                                    && !head_positions.get(&head_pos).unwrap().contains(other_id)
                                {
                                    snakes_to_kill.push(*snake_id);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Kill all snakes that collided
        for snake_id in snakes_to_kill {
            if let Some(snake) = self.state.snakes.get_mut(&snake_id) {
                snake.kill();
            }
        }

        Ok(())
    }

    /// Handle fruit consumption and snake growth
    fn handle_fruit_consumption(&mut self) -> GameResult<()> {
        let mut fruits_to_remove = Vec::new();
        let mut snakes_to_grow = Vec::new();

        // Check each fruit against each snake head
        for (fruit_idx, fruit) in self.state.fruits.iter().enumerate() {
            for (snake_id, snake) in &self.state.snakes {
                if snake.is_alive {
                    if let Some(head_pos) = snake.head() {
                        if head_pos == fruit.position {
                            fruits_to_remove.push(fruit_idx);
                            snakes_to_grow.push(*snake_id);
                            break; // Fruit can only be eaten by one snake
                        }
                    }
                }
            }
        }

        // Remove consumed fruits (in reverse order to maintain indices)
        fruits_to_remove.sort_unstable();
        fruits_to_remove.reverse();
        for idx in fruits_to_remove {
            self.state.fruits.remove(idx);
        }

        // Grow snakes that ate fruit
        for snake_id in snakes_to_grow {
            if let Some(snake) = self.state.snakes.get_mut(&snake_id) {
                snake.length += 1;
                // The snake already moved, so we need to not remove the tail
                // We'll handle this by extending the body
                if let Some(tail_pos) = snake.body.back().copied() {
                    snake.body.push_back(tail_pos);
                }
            }
        }

        Ok(())
    }

    /// Spawn new fruits according to game rules
    fn spawn_fruits(&mut self) -> GameResult<()> {
        let player_count = self.state.snakes.len();
        let max_fruits = if player_count > 1 {
            player_count - 1
        } else {
            0
        };

        // Update fruit spawn timers
        for timer in self.fruit_spawn_timer.values_mut() {
            *timer += 1;
        }

        // Spawn fruits that are ready
        let mut new_fruits = Vec::new();
        let timer_snapshot: Vec<(usize, u32)> = self
            .fruit_spawn_timer
            .iter()
            .map(|(&k, &v)| (k, v))
            .collect();

        for (fruit_id, timer) in timer_snapshot {
            if timer >= FRUIT_SPAWN_DELAY_TICKS && self.state.fruits.len() < max_fruits {
                if let Ok(position) = self.find_random_empty_position(&HashSet::new()) {
                    new_fruits.push((fruit_id, position));
                }
            }
        }

        // Add new fruits and reset timers
        for (fruit_id, position) in new_fruits {
            self.state
                .fruits
                .push(Fruit::new(position, self.state.tick));
            self.fruit_spawn_timer.insert(fruit_id, 0);
        }

        // Ensure we maintain the right number of fruit spawn timers
        while self.fruit_spawn_timer.len() < max_fruits {
            let new_id = self.fruit_spawn_timer.len();
            self.fruit_spawn_timer.insert(new_id, 0);
        }

        Ok(())
    }

    /// Check if the game should end and set winner
    fn check_game_end(&mut self) -> GameResult<()> {
        if self.state.is_game_over() {
            self.state.winner = self.state.get_winner();
            self.state.is_running = false;
        }
        Ok(())
    }

    /// Get valid moves for a specific snake
    pub fn get_valid_moves(&self, snake_id: &Uuid) -> Vec<Direction> {
        if let Some(snake) = self.state.snakes.get(snake_id) {
            if snake.is_alive {
                return snake.valid_directions();
            }
        }
        Vec::new()
    }

    /// Check if a specific snake is alive
    pub fn is_snake_alive(&self, snake_id: &Uuid) -> bool {
        self.state
            .snakes
            .get(snake_id)
            .map(|s| s.is_alive)
            .unwrap_or(false)
    }

    /// Get current game state snapshot
    pub fn get_game_state(&self) -> &GameState {
        &self.state
    }

    /// Get game statistics
    pub fn get_game_stats(&self) -> GameStats {
        let alive_count = self.state.snakes.values().filter(|s| s.is_alive).count();
        let total_count = self.state.snakes.len();
        let longest_snake = self
            .state
            .snakes
            .values()
            .max_by_key(|s| s.length)
            .map(|s| s.length)
            .unwrap_or(0);

        GameStats {
            tick: self.state.tick,
            alive_snakes: alive_count,
            total_snakes: total_count,
            fruits_on_board: self.state.fruits.len(),
            longest_snake_length: longest_snake,
            is_running: self.state.is_running,
            winner_id: self.state.winner,
        }
    }
}

/// Game statistics for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameStats {
    pub tick: u64,
    pub alive_snakes: usize,
    pub total_snakes: usize,
    pub fruits_on_board: usize,
    pub longest_snake_length: usize,
    pub is_running: bool,
    pub winner_id: Option<Uuid>,
}

impl Default for GameEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_initialization() {
        let mut engine = GameEngine::new();
        let mut players = HashMap::new();

        players.insert(
            Uuid::new_v4(),
            LobbyPlayer {
                id: Uuid::new_v4(),
                name: "Player1".to_string(),
                color_index: 0,
                is_ready: true,
            },
        );

        let result = engine.initialize_game(&players);
        assert!(result.is_ok());
        assert_eq!(engine.state.snakes.len(), 1);
        assert!(engine.state.is_running);
    }

    #[test]
    fn test_position_wrapping() {
        let pos = Position::new(0, 0);
        let new_pos = pos.move_in_direction(Direction::Left, 50, 50);
        assert_eq!(new_pos, Position::new(49, 0));

        let pos = Position::new(49, 49);
        let new_pos = pos.move_in_direction(Direction::Right, 50, 50);
        assert_eq!(new_pos, Position::new(0, 49));
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
    }

    #[test]
    fn test_snake_valid_directions() {
        let snake = Snake::new(Uuid::new_v4(), "Test".to_string(), Position::new(5, 5), 0);

        // New snake should be able to move in any direction
        let valid_dirs = snake.valid_directions();
        assert_eq!(valid_dirs.len(), 4);
    }
}
