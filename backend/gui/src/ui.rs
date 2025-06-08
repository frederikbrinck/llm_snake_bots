//! UI Manager module for the multiplayer snake game GUI
//! 
//! This module handles all DOM manipulation and UI state management
//! for the different phases of the game (lobby, game running, game ended).

use crate::types::*;
use wasm_bindgen::prelude::*;
use web_sys::{
    window, Document, Element, HtmlElement, HtmlButtonElement, 
    MouseEvent
};

/// UI Manager that handles all DOM interactions
pub struct UIManager {
    document: Document,
    root_element: Element,
}

impl UIManager {
    /// Create a new UI manager
    pub fn new() -> Result<Self, JsValue> {
        let window = window().unwrap();
        let document = window.document().unwrap();
        
        // Get or create root element
        let root_element = match document.get_element_by_id("app-root") {
            Some(element) => element,
            None => {
                let body = document.body().unwrap();
                let root = document.create_element("div")?;
                root.set_id("app-root");
                body.append_child(&root)?;
                root
            }
        };

        // Add CSS styles
        Self::inject_styles(&document)?;

        Ok(Self {
            document,
            root_element,
        })
    }

    /// Inject CSS styles into the document
    fn inject_styles(document: &Document) -> Result<(), JsValue> {
        let head = document.head().unwrap();
        let style = document.create_element("style")?;
        
        let css = r#"
            body {
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                margin: 0;
                padding: 20px;
                background-color: #1a1a1a;
                color: #ffffff;
            }
            
            #app-root {
                max-width: 1200px;
                margin: 0 auto;
            }
            
            .container {
                background-color: #2c2c2c;
                border-radius: 10px;
                padding: 20px;
                margin: 20px 0;
                box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
            }
            
            .lobby-container {
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 20px;
            }
            
            .game-container {
                display: flex;
                gap: 20px;
            }
            
            .game-canvas-container {
                display: flex;
                flex-direction: column;
                align-items: center;
            }
            
            .game-info {
                flex: 1;
                min-width: 200px;
            }
            
            .players-list {
                list-style: none;
                padding: 0;
                margin: 0;
            }
            
            .player-item {
                display: flex;
                align-items: center;
                padding: 10px;
                margin: 5px 0;
                background-color: #3c3c3c;
                border-radius: 5px;
                gap: 10px;
            }
            
            .player-color {
                width: 20px;
                height: 20px;
                border-radius: 50%;
                border: 2px solid #ffffff;
            }
            
            .player-name {
                flex: 1;
                font-weight: bold;
            }
            
            .player-length {
                color: #cccccc;
                font-size: 0.9em;
            }
            
            .player-dead {
                text-decoration: line-through;
                opacity: 0.5;
            }
            
            .button {
                background-color: #4CAF50;
                color: white;
                border: none;
                padding: 12px 24px;
                font-size: 16px;
                border-radius: 5px;
                cursor: pointer;
                transition: background-color 0.3s;
            }
            
            .button:hover {
                background-color: #45a049;
            }
            
            .button:disabled {
                background-color: #666666;
                cursor: not-allowed;
            }
            
            .button-danger {
                background-color: #f44336;
            }
            
            .button-danger:hover {
                background-color: #d32f2f;
            }
            
            .status-message {
                text-align: center;
                padding: 10px;
                border-radius: 5px;
                margin: 10px 0;
            }
            
            .status-connecting {
                background-color: #2196F3;
            }
            
            .status-error {
                background-color: #f44336;
            }
            
            .status-success {
                background-color: #4CAF50;
            }
            
            .game-stats {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: 10px;
                margin: 10px 0;
            }
            
            .stat-item {
                background-color: #3c3c3c;
                padding: 10px;
                border-radius: 5px;
                text-align: center;
            }
            
            .stat-label {
                font-size: 0.8em;
                color: #cccccc;
                margin-bottom: 5px;
            }
            
            .stat-value {
                font-size: 1.2em;
                font-weight: bold;
            }
            
            .winner-announcement {
                background-color: #FFD700;
                color: #000000;
                padding: 20px;
                border-radius: 10px;
                text-align: center;
                font-size: 1.2em;
                font-weight: bold;
                margin: 20px 0;
            }
            
            .loading-spinner {
                border: 4px solid #3c3c3c;
                border-top: 4px solid #4CAF50;
                border-radius: 50%;
                width: 40px;
                height: 40px;
                animation: spin 1s linear infinite;
                margin: 20px auto;
            }
            
            @keyframes spin {
                0% { transform: rotate(0deg); }
                100% { transform: rotate(360deg); }
            }
            
            #game-canvas {
                border: 2px solid #4CAF50;
                border-radius: 5px;
            }
            
            h1, h2, h3 {
                text-align: center;
                margin-bottom: 20px;
            }
            
            h1 {
                color: #4CAF50;
                font-size: 2.5em;
                margin-bottom: 10px;
            }
            
            .subtitle {
                color: #cccccc;
                font-size: 1.2em;
                text-align: center;
                margin-bottom: 30px;
            }
        "#;
        
        style.set_text_content(Some(css));
        head.append_child(&style)?;
        
        Ok(())
    }

    /// Show connecting state
    pub fn show_connecting(&self) -> Result<(), JsValue> {
        self.clear_content()?;
        
        let container = self.create_element("div", Some("container"))?;
        
        let title = self.create_element("h1", None)?;
        title.set_text_content(Some("🐍 Multiplayer Snake"));
        
        let subtitle = self.create_element("div", Some("subtitle"))?;
        subtitle.set_text_content(Some("Battle Royale Edition"));
        
        let status = self.create_element("div", Some("status-message status-connecting"))?;
        status.set_text_content(Some("Connecting to server..."));
        
        let spinner = self.create_element("div", Some("loading-spinner"))?;
        
        container.append_child(&title)?;
        container.append_child(&subtitle)?;
        container.append_child(&status)?;
        container.append_child(&spinner)?;
        
        self.root_element.append_child(&container)?;
        
        Ok(())
    }

    /// Show lobby state with player list
    pub fn show_lobby(&self, players: &[LobbyPlayer]) -> Result<(), JsValue> {
        self.clear_content()?;
        
        let container = self.create_element("div", Some("container lobby-container"))?;
        
        let title = self.create_element("h1", None)?;
        title.set_text_content(Some("🐍 Game Lobby"));
        
        let subtitle = self.create_element("div", Some("subtitle"))?;
        subtitle.set_text_content(Some(&format!("Players: {} / {}", players.len(), constants::MAX_PLAYERS)));
        
        // Players list
        let players_title = self.create_element("h3", None)?;
        players_title.set_text_content(Some("Connected Players"));
        
        let players_list = self.create_element("ul", Some("players-list"))?;
        
        for player in players {
            let player_item = self.create_player_item(player, None)?;
            players_list.append_child(&player_item)?;
        }
        
        // Start button
        let start_button = self.create_element("button", Some("button"))?
            .dyn_into::<HtmlButtonElement>()?;
        start_button.set_text_content(Some("Start Game"));
        start_button.set_disabled(players.len() < constants::MIN_PLAYERS);
        
        // Add click handler for start button
        let start_callback = Closure::wrap(Box::new(move |_event: MouseEvent| {
            crate::start_game_from_js();
        }) as Box<dyn FnMut(_)>);
        
        start_button.add_event_listener_with_callback("click", start_callback.as_ref().unchecked_ref())?;
        start_callback.forget();
        
        container.append_child(&title)?;
        container.append_child(&subtitle)?;
        container.append_child(&players_title)?;
        container.append_child(&players_list)?;
        container.append_child(&start_button)?;
        
        self.root_element.append_child(&container)?;
        
        Ok(())
    }

    /// Show game running state
    pub fn show_game(&self) -> Result<(), JsValue> {
        self.clear_content()?;
        
        let container = self.create_element("div", Some("container game-container"))?;
        
        // Canvas container
        let canvas_container = self.create_element("div", Some("game-canvas-container"))?;
        let canvas_title = self.create_element("h2", None)?;
        canvas_title.set_text_content(Some("Game Board"));
        canvas_container.append_child(&canvas_title)?;
        
        // The canvas will be added by the renderer
        
        // Game info panel
        let info_panel = self.create_element("div", Some("game-info"))?;
        let info_title = self.create_element("h3", None)?;
        info_title.set_text_content(Some("Game Info"));
        info_panel.append_child(&info_title)?;
        
        // Game stats
        let stats_container = self.create_element("div", Some("game-stats"))?;
        stats_container.set_id("game-stats");
        info_panel.append_child(&stats_container)?;
        
        // Players info
        let players_title = self.create_element("h3", None)?;
        players_title.set_text_content(Some("Players"));
        let players_list = self.create_element("ul", Some("players-list"))?;
        players_list.set_id("game-players-list");
        info_panel.append_child(&players_title)?;
        info_panel.append_child(&players_list)?;
        
        container.append_child(&canvas_container)?;
        container.append_child(&info_panel)?;
        
        self.root_element.append_child(&container)?;
        
        Ok(())
    }

    /// Update game information during gameplay
    pub fn update_game_info(&self, game_state: &GameState, players: &[LobbyPlayer]) -> Result<(), JsValue> {
        // Update game stats
        if let Some(stats_container) = self.document.get_element_by_id("game-stats") {
            stats_container.set_inner_html("");
            
            let tick_stat = self.create_stat_item("Tick", &game_state.tick.to_string())?;
            let alive_count = game_state.snakes.values().filter(|s| s.is_alive).count();
            let alive_stat = self.create_stat_item("Alive", &format!("{}/{}", alive_count, game_state.snakes.len()))?;
            let fruits_stat = self.create_stat_item("Fruits", &game_state.fruits.len().to_string())?;
            let longest = game_state.snakes.values().map(|s| s.length).max().unwrap_or(0);
            let longest_stat = self.create_stat_item("Longest", &longest.to_string())?;
            
            stats_container.append_child(&tick_stat)?;
            stats_container.append_child(&alive_stat)?;
            stats_container.append_child(&fruits_stat)?;
            stats_container.append_child(&longest_stat)?;
        }
        
        // Update players list
        if let Some(players_list) = self.document.get_element_by_id("game-players-list") {
            players_list.set_inner_html("");
            
            for player in players {
                if let Some(snake) = game_state.snakes.get(&player.id) {
                    let player_item = self.create_player_item(player, Some(snake))?;
                    players_list.append_child(&player_item)?;
                }
            }
        }
        
        Ok(())
    }

    /// Show game ended state
    pub fn show_game_ended(&self, winner: Option<LobbyPlayer>) -> Result<(), JsValue> {
        // Add winner announcement
        let announcement = self.create_element("div", Some("winner-announcement"))?;
        
        let message = match winner {
            Some(player) => format!("🏆 {} Wins! 🏆", player.name),
            None => "Game Ended - No Winner".to_string(),
        };
        
        announcement.set_text_content(Some(&message));
        
        // Insert at the top of the root element
        if let Some(first_child) = self.root_element.first_child() {
            self.root_element.insert_before(&announcement, Some(&first_child))?;
        } else {
            self.root_element.append_child(&announcement)?;
        }
        
        Ok(())
    }

    /// Show error state
    pub fn show_error(&self, message: &str) -> Result<(), JsValue> {
        self.clear_content()?;
        
        let container = self.create_element("div", Some("container"))?;
        
        let title = self.create_element("h1", None)?;
        title.set_text_content(Some("❌ Error"));
        
        let error_msg = self.create_element("div", Some("status-message status-error"))?;
        error_msg.set_text_content(Some(message));
        
        let retry_button = self.create_element("button", Some("button"))?;
        retry_button.set_text_content(Some("Reload Page"));
        
        let retry_callback = Closure::wrap(Box::new(move |_event: MouseEvent| {
            window().unwrap().location().reload().unwrap();
        }) as Box<dyn FnMut(_)>);
        
        retry_button.add_event_listener_with_callback("click", retry_callback.as_ref().unchecked_ref())?;
        retry_callback.forget();
        
        container.append_child(&title)?;
        container.append_child(&error_msg)?;
        container.append_child(&retry_button)?;
        
        self.root_element.append_child(&container)?;
        
        Ok(())
    }

    /// Create a player list item
    fn create_player_item(&self, player: &LobbyPlayer, snake: Option<&Snake>) -> Result<Element, JsValue> {
        let item = self.create_element("li", Some("player-item"))?;
        
        // Player color indicator
        let color_indicator = self.create_element("div", Some("player-color"))?;
        let color = constants::SNAKE_COLORS
            .get(player.color_index % constants::SNAKE_COLORS.len())
            .unwrap_or(&constants::SNAKE_COLORS[0]);
        let style = color_indicator.dyn_ref::<HtmlElement>().unwrap().style();
        style.set_property("background-color", color)?;
        
        // Player name
        let name_element = self.create_element("span", Some("player-name"))?;
        let mut class_list = "player-name".to_string();
        
        if let Some(snake) = snake {
            if !snake.is_alive {
                class_list.push_str(" player-dead");
            }
        }
        
        name_element.set_class_name(&class_list);
        name_element.set_text_content(Some(&player.name));
        
        // Player length (if in game)
        let length_element = self.create_element("span", Some("player-length"))?;
        if let Some(snake) = snake {
            length_element.set_text_content(Some(&format!("Length: {}", snake.length)));
        } else {
            length_element.set_text_content(Some("Ready"));
        }
        
        item.append_child(&color_indicator)?;
        item.append_child(&name_element)?;
        item.append_child(&length_element)?;
        
        Ok(item)
    }

    /// Create a stat item for the game stats grid
    fn create_stat_item(&self, label: &str, value: &str) -> Result<Element, JsValue> {
        let item = self.create_element("div", Some("stat-item"))?;
        
        let label_element = self.create_element("div", Some("stat-label"))?;
        label_element.set_text_content(Some(label));
        
        let value_element = self.create_element("div", Some("stat-value"))?;
        value_element.set_text_content(Some(value));
        
        item.append_child(&label_element)?;
        item.append_child(&value_element)?;
        
        Ok(item)
    }

    /// Helper method to create DOM elements
    fn create_element(&self, tag: &str, class_name: Option<&str>) -> Result<Element, JsValue> {
        let element = self.document.create_element(tag)?;
        
        if let Some(class) = class_name {
            element.set_class_name(class);
        }
        
        Ok(element)
    }

    /// Clear all content from the root element
    fn clear_content(&self) -> Result<(), JsValue> {
        self.root_element.set_inner_html("");
        Ok(())
    }

    /// Get the root element for canvas insertion
    pub fn get_canvas_container(&self) -> Result<Element, JsValue> {
        if let Some(container) = self.document.query_selector(".game-canvas-container")? {
            Ok(container)
        } else {
            Err(JsValue::from_str("Canvas container not found"))
        }
    }
}