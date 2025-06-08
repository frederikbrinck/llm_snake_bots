//! Canvas rendering module for the multiplayer snake game GUI
//! 
//! This module handles all canvas-based rendering including the game grid,
//! snakes, fruits, and visual effects.

use crate::types::*;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, window
};

/// Game renderer that handles all canvas drawing operations
pub struct GameRenderer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    canvas_width: f64,
    canvas_height: f64,
}

impl GameRenderer {
    /// Create a new game renderer
    pub fn new() -> Result<Self, JsValue> {
        let window = window().unwrap();
        let document = window.document().unwrap();
        
        // Get or create the game canvas
        let canvas = match document.get_element_by_id("game-canvas") {
            Some(element) => element
                .dyn_into::<HtmlCanvasElement>()
                .map_err(|_| JsValue::from_str("Failed to cast to HtmlCanvasElement"))?,
            None => {
                // Create canvas if it doesn't exist
                let canvas = document
                    .create_element("canvas")?
                    .dyn_into::<HtmlCanvasElement>()?;
                canvas.set_id("game-canvas");
                canvas
            }
        };

        // Set canvas dimensions
        let canvas_width = (constants::GRID_WIDTH as u32 * constants::CELL_SIZE_PX) as f64;
        let canvas_height = (constants::GRID_HEIGHT as u32 * constants::CELL_SIZE_PX) as f64;
        
        canvas.set_width(canvas_width as u32);
        canvas.set_height(canvas_height as u32);
        
        // Set CSS dimensions for proper scaling
        let style = canvas.style();
        style.set_property("width", &format!("{}px", canvas_width))?;
        style.set_property("height", &format!("{}px", canvas_height))?;
        style.set_property("border", "2px solid #34495E")?;
        style.set_property("background-color", constants::GRID_BACKGROUND_COLOR)?;

        // Get 2D rendering context
        let context = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        // Set initial context properties
        context.set_line_width(1.0);
        context.set_text_align("center");
        context.set_text_baseline("middle");

        Ok(Self {
            canvas,
            context,
            canvas_width,
            canvas_height,
        })
    }

    /// Get the canvas element for insertion into DOM
    pub fn get_canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    /// Render the complete game state
    pub fn render(&self, game_state: &GameState, players: &[LobbyPlayer]) -> Result<(), JsValue> {
        // Clear the canvas
        self.clear_canvas()?;
        
        // Draw grid lines
        self.draw_grid()?;
        
        // Draw fruits
        for fruit in &game_state.fruits {
            self.draw_fruit(&fruit.position)?;
        }
        
        // Draw snakes
        for snake in game_state.snakes.values() {
            let player = players.iter().find(|p| p.id == snake.id);
            self.draw_snake(snake, player)?;
        }

        Ok(())
    }

    /// Clear the entire canvas
    fn clear_canvas(&self) -> Result<(), JsValue> {
        self.context.set_fill_style(&JsValue::from_str(constants::GRID_BACKGROUND_COLOR));
        self.context.fill_rect(0.0, 0.0, self.canvas_width, self.canvas_height);
        Ok(())
    }

    /// Draw grid lines
    fn draw_grid(&self) -> Result<(), JsValue> {
        self.context.set_stroke_style(&JsValue::from_str(constants::GRID_LINE_COLOR));
        self.context.set_line_width(0.5);
        self.context.begin_path();

        let cell_size = constants::CELL_SIZE_PX as f64;

        // Draw vertical lines
        for x in 0..=constants::GRID_WIDTH {
            let x_pos = x as f64 * cell_size;
            self.context.move_to(x_pos, 0.0);
            self.context.line_to(x_pos, self.canvas_height);
        }

        // Draw horizontal lines
        for y in 0..=constants::GRID_HEIGHT {
            let y_pos = y as f64 * cell_size;
            self.context.move_to(0.0, y_pos);
            self.context.line_to(self.canvas_width, y_pos);
        }

        self.context.stroke();
        Ok(())
    }

    /// Draw a single fruit
    fn draw_fruit(&self, position: &Position) -> Result<(), JsValue> {
        let cell_size = constants::CELL_SIZE_PX as f64;
        let x = position.x as f64 * cell_size + cell_size / 2.0;
        let y = position.y as f64 * cell_size + cell_size / 2.0;
        let radius = cell_size / 3.0;

        self.context.set_fill_style(&JsValue::from_str(constants::FRUIT_COLOR));
        self.context.begin_path();
        self.context.arc(x, y, radius, 0.0, 2.0 * std::f64::consts::PI)?;
        self.context.fill();

        // Add a small white highlight
        self.context.set_fill_style(&JsValue::from_str("#FFFFFF"));
        self.context.begin_path();
        self.context.arc(x - radius / 3.0, y - radius / 3.0, radius / 4.0, 0.0, 2.0 * std::f64::consts::PI)?;
        self.context.fill();

        Ok(())
    }

    /// Draw a complete snake
    fn draw_snake(&self, snake: &Snake, player: Option<&LobbyPlayer>) -> Result<(), JsValue> {
        if snake.body.is_empty() {
            return Ok(());
        }

        // Get snake color
        let color = self.get_snake_color(snake, player);
        let alpha = if snake.is_alive { 1.0 } else { constants::DEAD_SNAKE_ALPHA };

        // Draw snake body
        for (index, position) in snake.body.iter().enumerate() {
            if index == 0 {
                // Draw head
                self.draw_snake_head(position, &color, alpha.into())?;
            } else {
                // Draw body segment
                self.draw_snake_body(position, &color, alpha.into(), false)?;
            }
        }

        Ok(())
    }

    /// Draw snake head with special styling
    fn draw_snake_head(&self, position: &Position, color: &str, alpha: f64) -> Result<(), JsValue> {
        let cell_size = constants::CELL_SIZE_PX as f64;
        let x = position.x as f64 * cell_size;
        let y = position.y as f64 * cell_size;
        let padding = 1.0;

        // Set color with alpha
        let rgba_color = self.hex_to_rgba(color, alpha);
        self.context.set_fill_style(&JsValue::from_str(&rgba_color));

        // Draw head as rectangle (fallback for roundRect)
        self.context.fill_rect(
            x + padding,
            y + padding,
            cell_size - 2.0 * padding,
            cell_size - 2.0 * padding,
        );

        // Add eyes for alive snakes
        if alpha > 0.8 {
            self.draw_snake_eyes(x, y, cell_size)?;
        }

        Ok(())
    }

    /// Draw snake body segment
    fn draw_snake_body(&self, position: &Position, color: &str, alpha: f64, _is_tail: bool) -> Result<(), JsValue> {
        let cell_size = constants::CELL_SIZE_PX as f64;
        let x = position.x as f64 * cell_size;
        let y = position.y as f64 * cell_size;
        let padding = 2.0;

        // Set color with alpha
        let rgba_color = self.hex_to_rgba(color, alpha);
        self.context.set_fill_style(&JsValue::from_str(&rgba_color));

        // Draw body as rectangle (fallback for roundRect)
        self.context.fill_rect(
            x + padding,
            y + padding,
            cell_size - 2.0 * padding,
            cell_size - 2.0 * padding,
        );

        Ok(())
    }

    /// Draw eyes on snake head
    fn draw_snake_eyes(&self, x: f64, y: f64, cell_size: f64) -> Result<(), JsValue> {
        let eye_size = cell_size / 8.0;
        let eye_offset_x = cell_size / 4.0;
        let eye_offset_y = cell_size / 3.0;

        self.context.set_fill_style(&JsValue::from_str("#000000"));

        // Left eye
        self.context.begin_path();
        self.context.arc(
            x + eye_offset_x,
            y + eye_offset_y,
            eye_size,
            0.0,
            2.0 * std::f64::consts::PI,
        )?;
        self.context.fill();

        // Right eye
        self.context.begin_path();
        self.context.arc(
            x + cell_size - eye_offset_x,
            y + eye_offset_y,
            eye_size,
            0.0,
            2.0 * std::f64::consts::PI,
        )?;
        self.context.fill();

        Ok(())
    }

    /// Get color for a snake based on its color index
    fn get_snake_color(&self, snake: &Snake, player: Option<&LobbyPlayer>) -> String {
        let color_index = player
            .map(|p| p.color_index)
            .unwrap_or(snake.color_index);
        
        constants::SNAKE_COLORS
            .get(color_index % constants::SNAKE_COLORS.len())
            .unwrap_or(&constants::SNAKE_COLORS[0])
            .to_string()
    }

    /// Convert hex color to rgba string with alpha
    fn hex_to_rgba(&self, hex: &str, alpha: f64) -> String {
        // Remove # if present
        let hex = hex.trim_start_matches('#');
        
        if hex.len() != 6 {
            return format!("rgba(255, 255, 255, {})", alpha);
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

        format!("rgba({}, {}, {}, {})", r, g, b, alpha)
    }

    /// Highlight a specific position (for debugging or effects)
    pub fn highlight_position(&self, position: &Position, color: &str) -> Result<(), JsValue> {
        let cell_size = constants::CELL_SIZE_PX as f64;
        let x = position.x as f64 * cell_size;
        let y = position.y as f64 * cell_size;

        self.context.set_stroke_style(&JsValue::from_str(color));
        self.context.set_line_width(3.0);
        self.context.stroke_rect(x, y, cell_size, cell_size);

        Ok(())
    }

    /// Add animation effect for fruit consumption
    pub fn animate_fruit_consumption(&self, position: &Position) -> Result<(), JsValue> {
        let cell_size = constants::CELL_SIZE_PX as f64;
        let x = position.x as f64 * cell_size + cell_size / 2.0;
        let y = position.y as f64 * cell_size + cell_size / 2.0;

        // Draw expanding circle effect
        self.context.set_stroke_style(&JsValue::from_str("#FFD700"));
        self.context.set_line_width(2.0);
        
        for radius in [cell_size / 2.0, cell_size * 0.75, cell_size] {
            self.context.begin_path();
            self.context.arc(x, y, radius, 0.0, 2.0 * std::f64::consts::PI)?;
            self.context.stroke();
        }

        Ok(())
    }

    /// Add death effect for snake
    pub fn animate_snake_death(&self, snake: &Snake) -> Result<(), JsValue> {
        if let Some(head_pos) = snake.head() {
            let cell_size = constants::CELL_SIZE_PX as f64;
            let x = head_pos.x as f64 * cell_size + cell_size / 2.0;
            let y = head_pos.y as f64 * cell_size + cell_size / 2.0;

            // Draw X mark
            self.context.set_stroke_style(&JsValue::from_str("#FF0000"));
            self.context.set_line_width(3.0);
            
            let size = cell_size / 3.0;
            self.context.begin_path();
            self.context.move_to(x - size, y - size);
            self.context.line_to(x + size, y + size);
            self.context.move_to(x + size, y - size);
            self.context.line_to(x - size, y + size);
            self.context.stroke();
        }

        Ok(())
    }

    /// Draw debug information
    pub fn draw_debug_info(&self, info: &str) -> Result<(), JsValue> {
        self.context.set_fill_style(&JsValue::from_str("#FFFFFF"));
        self.context.set_font("12px Arial");
        self.context.fill_text(info, 10.0, 20.0)?;
        Ok(())
    }
}

// Helper methods for canvas rendering
impl GameRenderer {
    /// Draw a rounded rectangle using path operations as fallback
    fn draw_rounded_rect(&self, x: f64, y: f64, width: f64, height: f64, radius: f64) -> Result<(), JsValue> {
        self.context.begin_path();
        self.context.move_to(x + radius, y);
        self.context.line_to(x + width - radius, y);
        self.context.quadratic_curve_to(x + width, y, x + width, y + radius);
        self.context.line_to(x + width, y + height - radius);
        self.context.quadratic_curve_to(x + width, y + height, x + width - radius, y + height);
        self.context.line_to(x + radius, y + height);
        self.context.quadratic_curve_to(x, y + height, x, y + height - radius);
        self.context.line_to(x, y + radius);
        self.context.quadratic_curve_to(x, y, x + radius, y);
        self.context.close_path();
        Ok(())
    }
}