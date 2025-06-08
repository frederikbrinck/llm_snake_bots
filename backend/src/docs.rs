//! OpenAPI documentation for the multiplayer snake game API
//!
//! This module provides comprehensive API documentation using utoipa,
//! making it easy for developers to implement clients in any language.

use crate::types::*;
use utoipa::OpenApi;

/// OpenAPI specification for the Multiplayer Snake Game API
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::server::health_check,
        crate::server::game_stats,
        crate::server::serve_openapi_spec,
        crate::server::serve_index,
        crate::server::serve_api_docs,
        crate::server::serve_swagger_ui,
        crate::server::websocket_documentation,
        crate::server::gui_documentation
    ),
    components(
        schemas(
            Position,
            Direction,
            Snake,
            Fruit,
            GameState,
            LobbyPlayer,
            ClientMessage,
            ServerMessage,
            GameError,
            GameStats,
        )
    ),
    tags(
        (name = "websocket", description = "WebSocket endpoints for real-time game communication"),
        (name = "health", description = "Health check and monitoring endpoints"),
        (name = "game", description = "Game state and statistics endpoints")
    ),
    info(
        title = "Multiplayer Snake Game API",
        version = "1.0.0",
        description = "A real-time multiplayer snake game API built with Rust, Axum, and WebSockets. This API enables multiple players to connect and play snake in a shared grid environment with collision detection, fruit spawning, and real-time updates.",
        contact(
            name = "Snake Game Development Team",
            email = "dev@snakegame.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "https://api.snakegame.com", description = "Production server")
    )
)]
pub struct ApiDoc;

/// Generate OpenAPI JSON specification
pub fn generate_openapi_spec() -> String {
    ApiDoc::openapi().to_pretty_json().unwrap()
}

/// API Documentation content for developers
pub const API_DOCUMENTATION: &str = r#"
# Multiplayer Snake Game API Documentation

## Overview

The Multiplayer Snake Game API provides real-time multiplayer snake gameplay through WebSocket connections. Players can join lobbies, control snakes, and compete in a shared grid environment.

## Game Rules

### Setup
- Grid size: 50x50 cells
- Initial snake length: 1 cell
- Maximum players: 8
- Minimum players to start: 2
- Winning condition: Be the last snake alive OR reach length 300

### Gameplay
- Each game tick (200ms interval), players submit their next move
- Valid directions: UP, DOWN, LEFT, RIGHT
- Grid wraps around (no boundaries)
- Snakes cannot move backward into their own tail

### Collisions
- Head-to-head collision: Both snakes die
- Head-to-tail collision: Moving snake dies
- Head-to-own-tail collision: Snake dies
- Dead snakes remain on grid as obstacles

### Fruit System
- Number of fruits = Number of players - 1
- Fruits spawn every 5 ticks in random empty cells
- Eating fruit increases snake length by 1

## WebSocket Endpoints

### Player Connection: `/lobby`
Connect as a player to join the game lobby and participate in matches.

**Connection Parameters:**
- `player_name` (optional): Your display name (auto-generated if not provided)

**Example Connection:**
```javascript
const ws = new WebSocket('ws://localhost:3000/lobby?player_name=YourName');
```

### GUI Connection: `/gui`
Connect as a spectator/controller to view the game and manage lobby state.

**Example Connection:**
```javascript
const ws = new WebSocket('ws://localhost:3000/gui');
```

## Message Protocol

### Client Messages (Player → Server)

#### JoinLobby
```json
{
  "type": "JoinLobby",
  "player_name": "string"
}
```
Join the game lobby with a specified name.

#### SubmitMove
```json
{
  "type": "SubmitMove",
  "direction": "Up" | "Down" | "Left" | "Right"
}
```
Submit your next move direction for the current game tick.

#### StartGame
```json
{
  "type": "StartGame"
}
```
Start the game (GUI only). All players must be connected and ready.

#### Ping
```json
{
  "type": "Ping"
}
```
Keep connection alive.

### Server Messages (Server → Client)

#### LobbyJoined
```json
{
  "type": "LobbyJoined",
  "player_id": "uuid",
  "player_name": "string"
}
```
Confirmation that you've joined the lobby.

#### LobbyState
```json
{
  "type": "LobbyState",
  "players": [
    {
      "id": "uuid",
      "name": "string",
      "color_index": 0,
      "is_ready": true
    }
  ]
}
```
Current lobby state with all connected players.

#### GameStarted
```json
{
  "type": "GameStarted",
  "game_state": "GameState",
  "your_snake_id": "uuid"
}
```
Game has started with initial state.

#### GameUpdate
```json
{
  "type": "GameUpdate",
  "game_state": "GameState"
}
```
Game state update after each tick.

#### MoveRequest
```json
{
  "type": "MoveRequest",
  "valid_directions": ["Up", "Down", "Left", "Right"],
  "time_limit_ms": 5000
}
```
Request for your next move with valid options.

#### GameEnded
```json
{
  "type": "GameEnded",
  "winner": "LobbyPlayer | null",
  "final_state": "GameState"
}
```
Game has ended with winner information.

#### Error
```json
{
  "type": "Error",
  "message": "string"
}
```
Error message from server.

## Data Structures

### Position
```json
{
  "x": 0,
  "y": 0
}
```
Represents a coordinate on the game grid.

### Snake
```json
{
  "id": "uuid",
  "player_name": "string",
  "body": [{"x": 10, "y": 10}, {"x": 10, "y": 11}],
  "length": 2,
  "is_alive": true,
  "color_index": 0,
  "last_direction": "Up"
}
```
Complete snake state including position, status, and metadata.

### Fruit
```json
{
  "position": {"x": 15, "y": 20},
  "spawn_tick": 100
}
```
Fruit position and spawn information.

### GameState
```json
{
  "snakes": {"uuid": "Snake"},
  "fruits": ["Fruit"],
  "tick": 150,
  "is_running": true,
  "winner": "uuid | null",
  "grid_width": 50,
  "grid_height": 50
}
```
Complete game state including all snakes, fruits, and metadata.

## Error Handling

### Common Errors
- `PlayerNotFound`: Player ID not found in game
- `GameNotRunning`: Attempted action when game is not active
- `InvalidMove`: Submitted move is not valid
- `RoomFull`: Lobby has reached maximum capacity
- `NameTaken`: Player name already in use
- `WebSocket`: WebSocket connection error
- `Serialization`: JSON parsing error

### Error Responses
All errors are sent as Error messages with descriptive text:
```json
{
  "type": "Error",
  "message": "Room is full. Maximum 8 players allowed."
}
```

## Rate Limiting

- Move submissions: 1 per game tick (200ms)
- Connection attempts: 10 per minute per IP
- Message size limit: 16KB

## Implementation Examples

### Basic Player Client (JavaScript)
```javascript
class SnakeGameClient {
  constructor(playerName) {
    this.playerName = playerName;
    this.ws = null;
    this.gameState = null;
  }
  
  connect() {
    const url = `ws://localhost:3000/lobby?player_name=${this.playerName}`;
    this.ws = new WebSocket(url);
    
    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      this.handleMessage(message);
    };
  }
  
  handleMessage(message) {
    switch (message.type) {
      case 'GameUpdate':
        this.gameState = message.game_state;
        break;
      case 'MoveRequest':
        const direction = this.chooseDirection(message.valid_directions);
        this.submitMove(direction);
        break;
      case 'GameEnded':
        console.log('Game ended!', message.winner);
        break;
    }
  }
  
  submitMove(direction) {
    this.ws.send(JSON.stringify({
      type: 'SubmitMove',
      direction: direction
    }));
  }
  
  chooseDirection(validDirections) {
    // Implement your AI logic here
    return validDirections[0];
  }
}
```

### Python Client Example
```python
import asyncio
import websockets
import json

class SnakeGameClient:
    def __init__(self, player_name):
        self.player_name = player_name
        self.game_state = None
    
    async def connect(self):
        uri = f"ws://localhost:3000/lobby?player_name={self.player_name}"
        async with websockets.connect(uri) as websocket:
            await self.game_loop(websocket)
    
    async def game_loop(self, websocket):
        async for message in websocket:
            data = json.loads(message)
            await self.handle_message(websocket, data)
    
    async def handle_message(self, websocket, message):
        if message['type'] == 'GameUpdate':
            self.game_state = message['game_state']
        elif message['type'] == 'MoveRequest':
            direction = self.choose_direction(message['valid_directions'])
            await self.submit_move(websocket, direction)
    
    async def submit_move(self, websocket, direction):
        move_message = {
            'type': 'SubmitMove',
            'direction': direction
        }
        await websocket.send(json.dumps(move_message))
    
    def choose_direction(self, valid_directions):
        # Implement your strategy here
        return valid_directions[0]

# Usage
client = SnakeGameClient("PythonBot")
asyncio.run(client.connect())
```

## Best Practices

### For Client Developers
1. **Always handle all message types** - The server may send unexpected messages
2. **Implement reconnection logic** - WebSocket connections can drop
3. **Validate server messages** - Don't assume perfect JSON structure
4. **Use timeouts for move requests** - Submit moves within the time limit
5. **Handle game state efficiently** - Game updates come frequently (5 FPS)

### For Game Strategy
1. **Avoid walls and other snakes** - Priority #1 is survival
2. **Target fruits efficiently** - Growth increases your chances
3. **Use grid wrapping** - Remember the grid has no boundaries
4. **Plan ahead** - Consider your snake's length when pathfinding
5. **Watch other players** - Predict their moves for better positioning

## Testing

### WebSocket Testing Tools
- [websocat](https://github.com/vi/websocat): Command-line WebSocket client
- [Postman](https://www.postman.com/): GUI-based WebSocket testing
- Browser DevTools: Built-in WebSocket inspection

### Sample Test Commands
```bash
# Connect to lobby
websocat ws://localhost:3000/lobby?player_name=TestPlayer

# Send join message
echo '{"type":"JoinLobby","player_name":"TestBot"}' | websocat ws://localhost:3000/lobby

# Submit a move
echo '{"type":"SubmitMove","direction":"Up"}' | websocat ws://localhost:3000/lobby
```

## Deployment

### Docker Deployment
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/backend /usr/local/bin/snake-game
EXPOSE 3000
CMD ["snake-game"]
```

### Environment Variables
- `RUST_LOG`: Set logging level (debug, info, warn, error)
- `SERVER_PORT`: Override default port (3000)
- `MAX_PLAYERS`: Override maximum players per game (8)

For more information, check the source code documentation and examples in the repository.
"#;
