//! WASM GUI for the multiplayer snake game
//! 
//! This module implements a simplified frontend graphical user interface using WASM
//! that displays the game lobby and running game state.

use wasm_bindgen::prelude::*;
use web_sys::{console, window, Document, Element};

mod types;
mod canvas;
mod ui;

// When the `console_error_panic_hook` feature is enabled, we can call the
// `set_panic_hook` function at least once during initialization, and then
// we will get better error messages if our code ever panics.
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    console::log_1(&"Starting Snake Game GUI".into());
    
    // Initialize the application
    if let Err(e) = init_app() {
        console::error_1(&format!("Failed to initialize app: {:?}", e).into());
    }
}

fn init_app() -> Result<(), JsValue> {
    let window = window().unwrap();
    let document = window.document().unwrap();
    
    // Create main app container
    let app_root = match document.get_element_by_id("app-root") {
        Some(element) => element,
        None => {
            let body = document.body().unwrap();
            let app_root = document.create_element("div")?;
            app_root.set_id("app-root");
            body.append_child(&app_root)?;
            app_root
        }
    };
    
    // Set up basic styling
    inject_styles(&document)?;
    
    // Show initial state
    show_connecting(&app_root)?;
    
    // Connect to WebSocket
    connect_websocket()?;
    
    Ok(())
}

fn inject_styles(document: &Document) -> Result<(), JsValue> {
    let head = document.head().unwrap();
    let style = document.create_element("style")?;
    
    let css = r#"
        body {
            font-family: Arial, sans-serif;
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
            text-align: center;
        }
        
        .game-container {
            display: flex;
            gap: 20px;
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
    "#;
    
    style.set_text_content(Some(css));
    head.append_child(&style)?;
    
    Ok(())
}

fn show_connecting(app_root: &Element) -> Result<(), JsValue> {
    app_root.set_inner_html("");
    
    let container = create_element("div", Some("container"))?;
    
    let title = create_element("h1", None)?;
    title.set_text_content(Some("🐍 Multiplayer Snake"));
    
    let status = create_element("div", Some("status-message status-connecting"))?;
    status.set_text_content(Some("Connecting to server..."));
    
    container.append_child(&title)?;
    container.append_child(&status)?;
    
    app_root.append_child(&container)?;
    
    Ok(())
}

fn connect_websocket() -> Result<(), JsValue> {
    let window = window().unwrap();
    let location = window.location();
    let hostname = location.hostname()?;
    let port = location.port()?;
    
    let ws_url = if port.is_empty() {
        format!("ws://{}/gui", hostname)
    } else {
        format!("ws://{}:{}/gui", hostname, port)
    };

    console::log_1(&format!("Connecting to: {}", ws_url).into());
    
    // For now, just show a simple message that WebSocket connection would happen here
    // In a full implementation, you would use gloo-net or web_sys WebSocket API
    
    Ok(())
}

fn create_element(tag: &str, class_name: Option<&str>) -> Result<Element, JsValue> {
    let window = window().unwrap();
    let document = window.document().unwrap();
    let element = document.create_element(tag)?;
    
    if let Some(class) = class_name {
        element.set_class_name(class);
    }
    
    Ok(element)
}

// Export functions for JavaScript interop
#[wasm_bindgen]
pub fn greet(name: &str) {
    console::log_1(&format!("Hello, {}!", name).into());
}

#[wasm_bindgen]
pub fn start_game() {
    console::log_1(&"Starting game from WASM!".into());
}

#[wasm_bindgen]
pub fn start_game_from_js() {
    console::log_1(&"Starting game from JavaScript!".into());
    // This function will be called from the UI when the start button is clicked
    // In a full implementation, this would send a StartGame message via WebSocket
}