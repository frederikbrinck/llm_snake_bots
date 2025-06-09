//! WebSocket server implementation for the multiplayer snake game
//!
//! This module implements the axum web server with WebSocket endpoints
//! for handling player connections and game communication.

use crate::constants::*;
use crate::docs::{ApiDoc, API_DOCUMENTATION};
use crate::game::GameEngine;
use crate::types::*;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

use futures_util::{sink::SinkExt, stream::StreamExt};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast, mpsc, RwLock},

};
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{error, info, warn};
use utoipa::OpenApi;
use uuid::Uuid;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub game_room: Arc<RwLock<GameRoom>>,
    pub connections: Arc<RwLock<HashMap<Uuid, PlayerConnection>>>,
    pub game_engine: Arc<RwLock<GameEngine>>,
    pub event_sender: broadcast::Sender<GameEvent>,
}

impl AppState {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);

        Self {
            game_room: Arc::new(RwLock::new(GameRoom::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            game_engine: Arc::new(RwLock::new(GameEngine::new())),
            event_sender,
        }
    }
}

/// WebSocket connection parameters
#[derive(serde::Deserialize)]
pub struct ConnectParams {
    pub player_name: Option<String>,
    pub is_gui: Option<bool>,
}

/// Create the main application router
pub fn create_app() -> Router {
    let state = AppState::new();

    // Start the game loop
    tokio::spawn(game_loop(state.clone()));

    Router::new()
        .route("/lobby", get(websocket_handler))
        .route("/gui", get(gui_websocket_handler))
        .route("/health", get(health_check))
        .route("/stats", get(game_stats))
        .route("/", get(serve_index))
        .route("/docs", get(serve_api_docs))
        .route("/swagger", get(serve_swagger_ui))
        .route("/api-spec.json", get(serve_openapi_spec))
        .route("/docs/websocket/lobby", get(websocket_documentation))
        .route("/docs/websocket/gui", get(gui_documentation))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Server is healthy", body = String)
    )
)]
async fn health_check() -> impl IntoResponse {
    "OK"
}

/// Game statistics endpoint
/// Get current game statistics
#[utoipa::path(
    get,
    path = "/stats",
    tag = "game",
    responses(
        (status = 200, description = "Current game statistics", body = GameStats)
    )
)]
async fn game_stats(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.game_engine.read().await;
    let stats = engine.get_game_stats();
    axum::Json(stats)
}

/// Serve the main index page
#[utoipa::path(
    get,
    path = "/",
    tag = "game",
    responses(
        (status = 200, description = "Main game interface", content_type = "text/html")
    )
)]
async fn serve_index() -> impl IntoResponse {
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(content) => Html(content),
        Err(_) => Html(
            "<h1>Snake Game Server</h1><p>GUI not found. Please check static files.</p>"
                .to_string(),
        ),
    }
}

/// Serve API documentation with comprehensive guide
#[utoipa::path(
    get,
    path = "/docs",
    tag = "docs",
    responses(
        (status = 200, description = "Interactive API documentation", content_type = "text/html")
    )
)]
async fn serve_api_docs() -> impl IntoResponse {
    match tokio::fs::read_to_string("static/api-docs.html").await {
        Ok(content) => Html(content),
        Err(_) => match tokio::fs::read_to_string("static/swagger-ui.html").await {
            Ok(content) => Html(content),
            Err(_) => Html(format!(
                r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Snake Game API Documentation</title>
                    <style>
                        body {{ font-family: Arial, sans-serif; margin: 40px; line-height: 1.6; }}
                        pre {{ background: #f4f4f4; padding: 10px; border-radius: 5px; overflow-x: auto; }}
                        code {{ background: #f4f4f4; padding: 2px 4px; border-radius: 3px; }}
                        h1, h2, h3 {{ color: #333; }}
                        .nav {{ background: #333; color: white; padding: 20px; margin: -40px -40px 40px -40px; }}
                        .nav a {{ color: #4CAF50; text-decoration: none; margin-right: 20px; }}
                    </style>
                </head>
                <body>
                    <div class="nav">
                        <h1>🐍 Snake Game API Documentation</h1>
                        <a href="/">Home</a>
                        <a href="/api-spec.json">OpenAPI JSON</a>
                        <a href="/health">Health Check</a>
                    </div>
                    <h2>Error: API Documentation not found</h2>
                    <p>Please ensure api-docs.html is in the static directory.</p>
                    <h3>Fallback Documentation:</h3>
                    <pre>{}</pre>
                </body>
                </html>
                "#,
                API_DOCUMENTATION
            )),
        },
    }
}

/// Serve Swagger UI for OpenAPI specification
#[utoipa::path(
    get,
    path = "/swagger",
    tag = "docs",
    responses(
        (status = 200, description = "Swagger UI interface", content_type = "text/html")
    )
)]
async fn serve_swagger_ui() -> impl IntoResponse {
    match tokio::fs::read_to_string("static/swagger-ui.html").await {
        Ok(content) => Html(content),
        Err(_) => Html(format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Swagger UI - Snake Game API</title>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 40px; line-height: 1.6; }}
                </style>
            </head>
            <body>
                <h1>🐍 Swagger UI</h1>
                <p>Swagger UI not found. Please ensure swagger-ui.html is in the static directory.</p>
                <p><a href="/docs">← Back to API Documentation</a></p>
            </body>
            </html>
            "#
        )),
    }
}

/// Serve OpenAPI specification as JSON
#[utoipa::path(
    get,
    path = "/api-spec.json",
    tag = "docs",
    responses(
        (status = 200, description = "OpenAPI specification in JSON format", content_type = "application/json")
    )
)]
async fn serve_openapi_spec() -> impl IntoResponse {
    axum::Json(ApiDoc::openapi())
}

/// WebSocket lobby endpoint documentation
#[utoipa::path(
    get,
    path = "/docs/websocket/lobby",
    tag = "websocket",
    responses(
        (status = 200, description = "WebSocket lobby endpoint documentation", content_type = "text/html")
    )
)]
async fn websocket_documentation() -> impl IntoResponse {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>WebSocket Lobby Endpoint</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; line-height: 1.6; }
                .endpoint { background: #f4f4f4; padding: 15px; border-radius: 5px; margin: 10px 0; }
                .method { background: #4CAF50; color: white; padding: 4px 8px; border-radius: 3px; }
            </style>
        </head>
        <body>
            <h1>🎮 WebSocket Lobby Endpoint</h1>
            <div class="endpoint">
                <p><span class="method">WebSocket</span> <strong>/lobby</strong></p>
                <p><strong>Description:</strong> Player connection endpoint for joining game lobby and real-time gameplay</p>
                <p><strong>URL:</strong> <code>ws://localhost:3000/lobby?player_name=YourName</code></p>
                <p><strong>Parameters:</strong></p>
                <ul>
                    <li><code>player_name</code> (optional): Your display name in the game</li>
                </ul>
                <p><strong>Protocol:</strong> WebSocket with JSON message exchange</p>
                <p><strong>Supported Messages:</strong> JoinLobby, SubmitMove, StartGame, Ping</p>
            </div>
            <p><a href="/docs">← Back to API Documentation</a></p>
        </body>
        </html>
    "#,
    )
}

/// WebSocket GUI endpoint documentation
#[utoipa::path(
    get,
    path = "/docs/websocket/gui",
    tag = "websocket",
    responses(
        (status = 200, description = "WebSocket GUI endpoint documentation", content_type = "text/html")
    )
)]
async fn gui_documentation() -> impl IntoResponse {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>WebSocket GUI Endpoint</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; line-height: 1.6; }
                .endpoint { background: #f4f4f4; padding: 15px; border-radius: 5px; margin: 10px 0; }
                .method { background: #2196F3; color: white; padding: 4px 8px; border-radius: 3px; }
            </style>
        </head>
        <body>
            <h1>👁️ WebSocket GUI Endpoint</h1>
            <div class="endpoint">
                <p><span class="method">WebSocket</span> <strong>/gui</strong></p>
                <p><strong>Description:</strong> Spectator and control interface for game observation and lobby management</p>
                <p><strong>URL:</strong> <code>ws://localhost:3000/gui</code></p>
                <p><strong>Protocol:</strong> WebSocket with JSON message exchange</p>
                <p><strong>Purpose:</strong> Read-only game state monitoring and lobby control</p>
                <p><strong>Supported Messages:</strong> StartGame (send), all server messages (receive)</p>
            </div>
            <p><a href="/docs">← Back to API Documentation</a></p>
        </body>
        </html>
    "#,
    )
}

/// WebSocket handler for player connections
async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<ConnectParams>,
    State(state): State<AppState>,
) -> Response {
    let player_name = params
        .player_name
        .unwrap_or_else(|| format!("Player_{}", Uuid::new_v4()));

    ws.on_upgrade(move |socket| handle_player_connection(socket, player_name, state))
}

/// WebSocket handler for GUI connections
async fn gui_websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_gui_connection(socket, state))
}

/// Handle a player WebSocket connection
async fn handle_player_connection(socket: WebSocket, player_name: String, state: AppState) {
    let player_id = Uuid::new_v4();
    info!("Player {} ({}) connected", player_name, player_id);

    // Try to add player to the game room
    let _color_index = {
        let mut room = state.game_room.write().await;
        match room.add_player(player_id, player_name.clone()) {
            Ok(color_index) => color_index,
            Err(error) => {
                warn!("Failed to add player {}: {}", player_name, error);
                return;
            }
        }
    };

    // Set up connection
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Store connection
    {
        let mut connections = state.connections.write().await;
        connections.insert(
            player_id,
            PlayerConnection {
                player_id,
                sender: tx.clone(),
            },
        );
    }

    // Send lobby joined confirmation
    let _ = tx.send(ServerMessage::LobbyJoined {
        player_id,
        player_name: player_name.clone(),
    });

    // Notify that a player joined
    let _ = state.event_sender.send(GameEvent::PlayerJoined(player_id, player_name.clone()));

    // Spawn task to handle outgoing messages
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&message) {
                if ws_sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages
    let mut event_receiver = state.event_sender.subscribe();
    loop {
        tokio::select! {
            // Handle WebSocket messages
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_player_message(text, player_id, &state).await {
                            error!("Error handling player message: {}", e);
                            let _ = tx.send(ServerMessage::Error {
                                message: format!("Error processing message: {}", e),
                            });
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("Player {} disconnected", player_name);
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error for player {}: {}", player_name, e);
                        break;
                    }
                    Some(Ok(Message::Binary(_))) | Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {
                        // Ignore binary, ping, and pong messages
                    }
                }
            }

            // Handle game events
            Ok(event) = event_receiver.recv() => {
                match event {
                    GameEvent::GameStarted => {
                        let engine = state.game_engine.read().await;
                        if engine.state.snakes.contains_key(&player_id) {
                            let _ = tx.send(ServerMessage::GameStarted {
                                game_state: engine.state.clone(),
                                your_snake_id: player_id,
                            });
                            
                            // Send initial move request
                            if engine.is_snake_alive(&player_id) {
                                let valid_directions = engine.get_valid_moves(&player_id);
                                info!("🎯 Sending initial move request to player {}", player_name);
                                let _ = tx.send(ServerMessage::MoveRequest {
                                    valid_directions,
                                    time_limit_ms: MOVE_TIMEOUT_MS,
                                });
                            }
                        }
                    }
                    GameEvent::GameTick => {
                        let engine = state.game_engine.read().await;
                        let _ = tx.send(ServerMessage::GameUpdate {
                            game_state: engine.state.clone(),
                        });

                        // Request next move if snake is alive
                        if engine.is_snake_alive(&player_id) {
                            let valid_directions = engine.get_valid_moves(&player_id);
                            let _ = tx.send(ServerMessage::MoveRequest {
                                valid_directions,
                                time_limit_ms: MOVE_TIMEOUT_MS,
                            });
                        }
                    }
                    GameEvent::GameEnded(winner_id) => {
                        let room = state.game_room.read().await;
                        let winner = winner_id.and_then(|id| room.players.get(&id).cloned());
                        let engine = state.game_engine.read().await;

                        let _ = tx.send(ServerMessage::GameEnded {
                            winner,
                            final_state: engine.state.clone(),
                        });
                    }
                    GameEvent::PlayerJoined(_, _) | GameEvent::PlayerLeft(_) => {
                        // These events don't affect individual player connections
                    }
                }
            }
        }
    }

    // Clean up connection
    {
        let mut connections = state.connections.write().await;
        connections.remove(&player_id);
    }

    {
        let mut room = state.game_room.write().await;
        room.remove_player(&player_id);
    }

    let _ = state.event_sender.send(GameEvent::PlayerLeft(player_id));
}

/// Handle GUI WebSocket connection
async fn handle_gui_connection(socket: WebSocket, state: AppState) {
    info!("🎮 GUI connected - initializing interface");

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Spawn task to handle outgoing messages
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&message) {
                if ws_sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });


    // Send initial lobby state directly to this GUI connection
    {
        let room = state.game_room.read().await;
        let players: Vec<LobbyPlayer> = room.players.values().cloned().collect();
        info!("📤 Sending initial lobby state with {} players", players.len());
        let message = ServerMessage::LobbyState { players };
        let _ = tx.send(message);
    }

    // Handle incoming messages and events
    let mut event_receiver = state.event_sender.subscribe();
    loop {
        tokio::select! {
            // Handle WebSocket messages
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_gui_message(text, &state, &tx).await {
                            error!("Error handling GUI message: {}", e);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("GUI disconnected");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error for GUI: {}", e);
                        break;
                    }
                    _ => {}
                }
            }

            // Handle game events
            Ok(event) = event_receiver.recv() => {
                match event {
                    GameEvent::PlayerJoined(_, _) | GameEvent::PlayerLeft(_) => {
                        let room = state.game_room.read().await;
                        let players: Vec<LobbyPlayer> = room.players.values().cloned().collect();
                        info!("👥 Lobby updated: {} players", players.len());
                        let message = ServerMessage::LobbyState { players };
                        let _ = tx.send(message);
                    }
                    GameEvent::GameStarted => {
                        info!("🚀 Game started! Sending initial game state to GUI");
                        let engine = state.game_engine.read().await;
                        let _ = tx.send(ServerMessage::GameUpdate {
                            game_state: engine.state.clone(),
                        });
                    }
                    GameEvent::GameTick => {
                        let engine = state.game_engine.read().await;
                        if engine.state.tick % 10 == 0 {
                            info!("⏱️  Game tick {}", engine.state.tick);
                        }
                        let _ = tx.send(ServerMessage::GameUpdate {
                            game_state: engine.state.clone(),
                        });
                    }
                    GameEvent::GameEnded(winner_id) => {
                        let room = state.game_room.read().await;
                        let winner = winner_id.and_then(|id| room.players.get(&id).cloned());
                        let engine = state.game_engine.read().await;

                        let _ = tx.send(ServerMessage::GameEnded {
                            winner,
                            final_state: engine.state.clone(),
                        });
                    }
                }
            }
        }
    }
}

/// Handle player messages
async fn handle_player_message(text: String, player_id: Uuid, state: &AppState) -> GameResult<()> {
    let message: ClientMessage = serde_json::from_str(&text)?;

    match message {
        ClientMessage::JoinLobby { player_name } => {
            // Add or update player in the game room
            let mut room = state.game_room.write().await;
            match room.add_player(player_id, player_name.clone()) {
                Ok(_) => {
                    // Player successfully added or updated
                }
                Err(error) => {
                    // Send error
                    if let Some(connection) = state.connections.read().await.get(&player_id) {
                        let _ = connection.sender.send(ServerMessage::Error {
                            message: format!("Error {}", error),
                        });
                    }
                }
            }
            drop(room);

            // Broadcast updated lobby state to all connections (only once)
            broadcast_lobby_state(state).await;
        }
        ClientMessage::SubmitMove { direction } => {
            let mut room = state.game_room.write().await;
            room.pending_moves.insert(player_id, direction);
            info!("🎮 Player {} submitted move: {:?}", player_id, direction);

            // Note: We don't send MovesSubmitted event anymore, 
            // the game loop uses polling to check for all moves
        }
        ClientMessage::Ping => {
            if let Some(connection) = state.connections.read().await.get(&player_id) {
                let _ = connection.sender.send(ServerMessage::Pong);
            }
        }
        message => {
            return Err(GameError::InvalidMove(format!(
                "Invalid message type for player: {:?}",
                message
            )));
        }
    }

    Ok(())
}

/// Handle GUI messages
async fn handle_gui_message(text: String, state: &AppState, tx: &mpsc::UnboundedSender<ServerMessage>) -> GameResult<()> {
    let message: ClientMessage = serde_json::from_str(&text)?;

    match message {
        ClientMessage::StartGame => {
            let room = state.game_room.read().await;
            info!("🎮 GUI requested game start. Current players: {}", room.players.len());
            
            // Check if we have enough players to start (players are ready by default)
            if room.players.len() >= MIN_PLAYERS {
                drop(room);
                
                let room = state.game_room.read().await;
                // Initialize game engine
                {
                    let mut engine = state.game_engine.write().await;
                    info!("🎯 Initializing game with {} players", room.players.len());
                    engine.initialize_game(&room.players)?;
                    info!("🐍 Game engine initialized successfully");
                }

                let _ = state.event_sender.send(GameEvent::GameStarted);
                info!("📡 GameStarted event sent");
            } else {
                let error_msg = format!("Need at least {} players to start (current: {})", MIN_PLAYERS, room.players.len());
                info!("❌ {}", error_msg);
                let _ = tx.send(ServerMessage::Error {
                    message: error_msg,
                });
            }
        }
        ClientMessage::JoinLobby { .. } => {
            // GUI should not be able to add players - only real clients can join
            let _ = tx.send(ServerMessage::Error {
                message: "GUI cannot add players directly. Use bot.py or other clients to join.".to_string(),
            });
        }
        _ => {
            return Err(GameError::InvalidMove(
                "Invalid message type for GUI".to_string(),
            ));
        }
    }

    Ok(())
}

/// Broadcast lobby state to all connections including GUI
async fn broadcast_lobby_state(state: &AppState) {
    let room = state.game_room.read().await;
    let players: Vec<LobbyPlayer> = room.players.values().cloned().collect();

    let message = ServerMessage::LobbyState { players };

    // Send to all player connections
    let connections = state.connections.read().await;
    for connection in connections.values() {
        let _ = connection.sender.send(message.clone());
    }
}

/// Main game loop that processes ticks
async fn game_loop(state: AppState) {
    let mut event_receiver = state.event_sender.subscribe();
    
    info!("Game loop started - waiting for game events");

    loop {
        if let Ok(event) = event_receiver.recv().await {
            match event {
                GameEvent::GameStarted => {
                    info!("🚀 Game started - beginning tick processing");
                    
                    // Run the game loop
                    loop {
                        let tick_start_time = tokio::time::Instant::now();
                        
                        // Check if game is still running
                        let is_running = {
                            let engine = state.game_engine.read().await;
                            engine.state.is_running
                        };
                        
                        if !is_running {
                            break;
                        }
                        
                        info!("⏳ Waiting for player moves (5 second timeout)...");
                        
                        // Wait for moves with 5-second timeout
                        let moves = loop {
                            // Check if all moves are submitted
                            let all_submitted = {
                                let room = state.game_room.read().await;
                                let engine = state.game_engine.read().await;
                                room.all_moves_submitted(&engine.state)
                            };
                            
                            if all_submitted {
                                info!("✅ All moves submitted");
                                let mut room = state.game_room.write().await;
                                let moves = room.pending_moves.clone();
                                room.pending_moves.clear();
                                break moves;
                            }
                            
                            // Check for timeout
                            if tick_start_time.elapsed() >= Duration::from_millis(MOVE_TIMEOUT_MS) {
                                info!("⏰ Move timeout - processing with available moves");
                                let mut room = state.game_room.write().await;
                                let moves = room.pending_moves.clone();
                                room.pending_moves.clear();
                                break moves;
                            }
                            
                            // Wait a bit before checking again
                            tokio::time::sleep(Duration::from_millis(50)).await;
                        };
                        
                        // Ensure minimum 200ms delay for UI visibility
                        let elapsed = tick_start_time.elapsed();
                        if elapsed < Duration::from_millis(GAME_TICK_DURATION_MS) {
                            let remaining = Duration::from_millis(GAME_TICK_DURATION_MS) - elapsed;
                            info!("⏱️ Waiting {}ms for minimum tick duration", remaining.as_millis());
                            tokio::time::sleep(remaining).await;
                        }
                        
                        // Log submitted moves
                        info!("🎮 Processing tick with {} moves submitted", moves.len());
                        for (player_id, direction) in &moves {
                            info!("  - Player {}: {:?}", player_id, direction);
                        }
                        
                        // Process the game tick
                        {
                            let mut engine = state.game_engine.write().await;
                            if let Err(e) = engine.process_tick(moves) {
                                error!("❌ Error processing game tick: {}", e);
                                break;
                            }
                            
                            // Check if game ended
                            if !engine.state.is_running {
                                let winner_id = engine.state.winner;
                                info!("🏁 Game ended! Winner: {:?}", winner_id);
                                let _ = state.event_sender.send(GameEvent::GameEnded(winner_id));
                                break;
                            }
                        }
                        
                        // Send game update
                        let _ = state.event_sender.send(GameEvent::GameTick);
                    }
                }
                GameEvent::GameEnded(_) => {
                    info!("🏁 Game ended - stopping game loop");
                    // Game ended, continue listening for new games
                }
                _ => {
                    // Ignore other events
                }
            }
        }
    }
}

/// Start the server
pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .init();

    let app = create_app();
    let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));

    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
