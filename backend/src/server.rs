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
    time::interval,
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
    Html(r#"
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
    "#)
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
    Html(r#"
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
    "#)
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

    // Broadcast lobby state update
    broadcast_lobby_state(&state).await;

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
                    _ => {}
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
                    _ => {}
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
    broadcast_lobby_state(&state).await;
}

/// Handle GUI WebSocket connection
async fn handle_gui_connection(socket: WebSocket, state: AppState) {
    info!("GUI connected");

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

    // Send initial lobby state
    broadcast_lobby_state(&state).await;

    // Handle incoming messages and events
    let mut event_receiver = state.event_sender.subscribe();
    loop {
        tokio::select! {
            // Handle WebSocket messages
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_gui_message(text, &state).await {
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
                        broadcast_lobby_state(&state).await;
                    }
                    GameEvent::GameStarted => {
                        let engine = state.game_engine.read().await;
                        let _ = tx.send(ServerMessage::GameUpdate {
                            game_state: engine.state.clone(),
                        });
                    }
                    GameEvent::GameTick => {
                        let engine = state.game_engine.read().await;
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
                    _ => {}
                }
            }
        }
    }
}

/// Handle player messages
async fn handle_player_message(text: String, player_id: Uuid, state: &AppState) -> GameResult<()> {
    let message: ClientMessage = serde_json::from_str(&text)?;

    match message {
        ClientMessage::SubmitMove { direction } => {
            let mut room = state.game_room.write().await;
            room.pending_moves.insert(player_id, direction);

            // Check if all moves are submitted
            if room.all_moves_submitted() {
                let _ = state.event_sender.send(GameEvent::MovesSubmitted);
            }
        }
        ClientMessage::Ping => {
            if let Some(connection) = state.connections.read().await.get(&player_id) {
                let _ = connection.sender.send(ServerMessage::Pong);
            }
        }
        _ => {
            return Err(GameError::InvalidMove(
                "Invalid message type for player".to_string(),
            ));
        }
    }

    Ok(())
}

/// Handle GUI messages
async fn handle_gui_message(text: String, state: &AppState) -> GameResult<()> {
    let message: ClientMessage = serde_json::from_str(&text)?;

    match message {
        ClientMessage::StartGame => {
            let room = state.game_room.read().await;
            if room.can_start_game() {
                // Initialize game engine
                let mut engine = state.game_engine.write().await;
                engine.initialize_game(&room.players)?;

                let _ = state.event_sender.send(GameEvent::GameStarted);
            }
        }
        _ => {
            return Err(GameError::InvalidMove(
                "Invalid message type for GUI".to_string(),
            ));
        }
    }

    Ok(())
}

/// Broadcast lobby state to all connections
async fn broadcast_lobby_state(state: &AppState) {
    let room = state.game_room.read().await;
    let players: Vec<LobbyPlayer> = room.players.values().cloned().collect();

    let message = ServerMessage::LobbyState { players };

    let connections = state.connections.read().await;
    for connection in connections.values() {
        let _ = connection.sender.send(message.clone());
    }
}

/// Main game loop that processes ticks
async fn game_loop(state: AppState) {
    let mut interval = interval(Duration::from_millis(GAME_TICK_DURATION_MS));

    info!("Game loop started");

    loop {
        interval.tick().await;

        // Check if game is running and all moves are ready
        let should_process_tick = {
            let room = state.game_room.read().await;
            let engine = state.game_engine.read().await;
            engine.state.is_running && room.all_moves_submitted()
        };

        if should_process_tick {
            // Process game tick
            let moves = {
                let mut room = state.game_room.write().await;
                let moves = room.pending_moves.clone();
                room.pending_moves.clear();
                moves
            };

            let mut engine = state.game_engine.write().await;
            if let Err(e) = engine.process_tick(moves) {
                error!("Error processing game tick: {}", e);
                continue;
            }

            // Check if game ended
            if !engine.state.is_running {
                let winner_id = engine.state.winner;
                let _ = state.event_sender.send(GameEvent::GameEnded(winner_id));
            } else {
                let _ = state.event_sender.send(GameEvent::GameTick);
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
