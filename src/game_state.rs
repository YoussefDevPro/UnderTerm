use crate::player::{Player, PlayerDirection};
use crate::{PLAYER_SPEED, PLAYER_HORIZONTAL_SPEED};
use crate::map::Map;
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};
use std::io::{self};
use std::collections::HashMap;
use ratatui::text::Text;

use ansi_to_tui::IntoText;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub camera_x: u16,
    pub camera_y: u16,
    pub message: String,
    pub show_message: bool,
    pub show_debug_panel: bool, // Added this line
    pub sound_error: Option<String>, // New field for sound errors
    #[serde(skip)]
    pub loaded_maps: std::collections::HashMap<(i32, i32), Map>,
    pub debug_mode: bool,
    pub current_map_name: String,
    pub current_map_row: i32,
    pub current_map_col: i32,
    #[serde(skip)]
    pub wall_history: Vec<Vec<(u32, u32)>>,
    #[serde(skip)]
    pub history_index: usize,
}

impl GameState {
    pub fn from_map(map: Map) -> Self {
        let player_spawn_x = map.player_spawn.0;
        let player_spawn_y = map.player_spawn.1;

        // Parse row and col from map.name
        let map_parts: Vec<&str> = map.name.split('_').collect();
        let current_map_row: i32 = map_parts[1].parse().unwrap_or(0);
        let current_map_col: i32 = map_parts[2].parse().unwrap_or(0);

        let mut loaded_maps = HashMap::new();
        loaded_maps.insert((current_map_row, current_map_col), map.clone()); // Clone map here

        GameState {
            player: Player::new(player_spawn_x as u16, player_spawn_y as u16),
            camera_x: 0,
            camera_y: 0,
            message: String::new(),
            show_message: false,
            current_map_name: format!("map_{}_{}", current_map_row, current_map_col),
            loaded_maps,
            debug_mode: false,
            show_debug_panel: false, // Added this line
            sound_error: None, // Initialize sound_error
            current_map_row,
            current_map_col,
            wall_history: vec![map.walls.clone()], // Initialize with current map walls
            history_index: 0,
        }
    }

    pub fn save_game_state(&self) -> io::Result<()> {
        let save_data = SaveData {
            player_x: self.player.x,
            player_y: self.player.y,
            current_map_name: self.current_map_name.clone(),
        };
        let serialized = serde_json::to_string(&save_data)?;
        std::fs::write("game_data.json", serialized)?;
        Ok(())
    }

    pub fn load_game_state() -> io::Result<Self> {
        match std::fs::read_to_string("game_data.json") {
            Ok(data) => {
                let deserialized: SaveData = serde_json::from_str(&data)?;
                let map = Map::load(&deserialized.current_map_name)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to load map: {}", e)))?;
                let mut game_state = GameState::from_map(map);
                game_state.player.x = deserialized.player_x;
                game_state.player.y = deserialized.player_y;
                Ok(game_state)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let map = Map::load("map_0_0")
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to load default map: {}", e)))?;
                let default_state = GameState::from_map(map);
                default_state.save_game_state()?;
                Ok(default_state)
            }
            Err(e) => Err(e),
        }
    }

    pub fn update(&mut self, key_code: Option<KeyCode>, frame_size: ratatui::layout::Rect, animation_frame_duration: std::time::Duration) {
        self.show_message = false;

        // Get the current map from loaded_maps
        let current_map_key = (self.current_map_row, self.current_map_col);
        let current_map = self.loaded_maps.get(&current_map_key).cloned().unwrap(); // Get the current map

        let mut moved_this_frame = false; // Moved declaration here

        if let Some(key) = key_code {
            let mut new_player_x = self.player.x;
            let mut new_player_y = self.player.y;

            match key {
                KeyCode::Up => {
                    new_player_y = new_player_y.saturating_sub(PLAYER_SPEED);
                    self.player.direction = PlayerDirection::Back;
                    moved_this_frame = true;
                }
                KeyCode::Down => {
                    new_player_y = new_player_y.saturating_add(PLAYER_SPEED);
                    self.player.direction = PlayerDirection::Front;
                    moved_this_frame = true;
                }
                KeyCode::Left => {
                    new_player_x = new_player_x.saturating_sub(PLAYER_HORIZONTAL_SPEED);
                    self.player.direction = PlayerDirection::Left;
                    moved_this_frame = true;
                }
                KeyCode::Right => {
                    new_player_x = new_player_x.saturating_add(PLAYER_HORIZONTAL_SPEED);
                    self.player.direction = PlayerDirection::Right;
                    moved_this_frame = true;
                }
                
                KeyCode::Enter => {
                    if self.debug_mode {
                        let current_map_key = (self.current_map_row, self.current_map_col);
                        if let Some(map_to_modify) = self.loaded_maps.get_mut(&current_map_key) {
                            map_to_modify.toggle_wall(self.player.x as u32, self.player.y as u32);
                            if let Err(e) = map_to_modify.save_data() {
                                self.message = format!("Failed to save map data: {}", e);
                                self.show_message = true;
                            } else {
                                self.message = format!("Toggled wall at ({}, {}) and saved.", self.player.x, self.player.y);
                                self.show_message = true;
                            }
                            // Update history
                            self.wall_history.truncate(self.history_index + 1);
                            self.wall_history.push(map_to_modify.walls.clone());
                            self.history_index = self.wall_history.len() - 1;
                        }
                    }
                }
                _ => {}
            }
            self.player.is_walking = moved_this_frame; // Set is_walking based on movement

            // --- Map Transition Logic ---
            let mut transitioned = false;
            let mut next_map_row = self.current_map_row;
            let mut next_map_col = self.current_map_col;

            // Store original player position before potential map transition
            let original_player_x = self.player.x;
            let original_player_y = self.player.y;

            // Check for transition to new map
            if new_player_x >= current_map.width { // Moving right
                next_map_col += 1;
                new_player_x = 0; // Reset player to left edge of new map
                transitioned = true;
            } else if self.player.x == 0 && key == KeyCode::Left { // Moving left from edge
                if self.current_map_col > 0 {
                    next_map_col -= 1;
                    new_player_x = current_map.width - 1; // Reset player to right edge of new map
                    transitioned = true;
                } else {
                    new_player_x = 0; // Clamp at left edge of map 0_0
                }
            }

            if new_player_y >= current_map.height { // Moving down
                next_map_row += 1;
                new_player_y = 0; // Reset player to top edge of new map
                transitioned = true;
            } else if self.player.y == 0 && key == KeyCode::Up { // Moving up from edge
                if self.current_map_row > 0 {
                    next_map_row -= 1;
                    new_player_y = current_map.height - 1; // Reset player to bottom edge of new map
                    transitioned = true;
                } else {
                    new_player_y = 0; // Clamp at top edge of map 0_0
                }
            }

            if transitioned {
                let new_map_name = format!("map_{}_{}", next_map_row, next_map_col);
                let new_map_key = (next_map_row, next_map_col);

                // Check if the map is already loaded
                if !self.loaded_maps.contains_key(&new_map_key) {
                    match Map::load(&new_map_name) {
                        Ok(new_map) => {
                            self.loaded_maps.insert(new_map_key, new_map);
                        }
                        Err(e) => {
                            // If map doesn't exist, revert player position and show message
                            self.message = format!("Cannot enter {}: {}", new_map_name, e);
                            self.show_message = true;
                            // Revert player position to stay on current map
                            self.player.x = original_player_x;
                            self.player.y = original_player_y;
                            return; // Exit update early to prevent further processing with invalid map
                        }
                    }
                }
                // Update current map coordinates
                self.current_map_name = new_map_name;
                self.current_map_row = next_map_row;
                self.current_map_col = next_map_col;
                self.player.x = new_player_x;
                self.player.y = new_player_y;
                self.message = format!("Entered {}", self.current_map_name);
                self.show_message = true;
            }

            // --- End Map Transition Logic ---

            let (_, player_sprite_width, player_sprite_height) = self.player.get_sprite_content();
            
            if !self.debug_mode {
                let collision_box_height = 3;
                let collision_box_start_y = new_player_y.saturating_add(player_sprite_height).saturating_sub(collision_box_height);

                let mut collision = false;
                for y_offset in 0..collision_box_height {
                    let check_y = collision_box_start_y.saturating_add(y_offset);
                    for x_offset in 0..player_sprite_width {
                        let check_x = new_player_x.saturating_add(x_offset);

                        // Get the map for collision check
                        let collision_map_key = (self.current_map_row, self.current_map_col);
                        if let Some(collision_map) = self.loaded_maps.get(&collision_map_key) {
                            if collision_map.walls.contains(&(check_x as u32, check_y as u32)) {
                                collision = true;
                                break;
                            }
                        }
                    }
                    if collision { break; }
                }

                if collision {
                    self.message = "You hit a wall!".to_string();
                    self.show_message = true;
                    // Do not update player position if collision occurs
                } else {
                    // Clamp player position to map boundaries, considering sprite size
                    self.player.x = new_player_x.min(current_map.width.saturating_sub(player_sprite_width));
                    self.player.y = new_player_y.min(current_map.height.saturating_sub(player_sprite_height));
                }
            } else {
                // In debug mode, allow walking through walls
                self.player.x = new_player_x.min(current_map.width.saturating_sub(player_sprite_width));
                self.player.y = new_player_y.min(current_map.height.saturating_sub(player_sprite_height));
            }
        }

        self.player.update_animation(animation_frame_duration);

        // Re-implement continuous camera logic
        // Calculate the desired camera position to center the player
        // Consider player sprite's center for more accurate centering
        let (_player_sprite_content, player_sprite_width, player_sprite_height) = self.player.get_sprite_content();

        let player_center_x = self.player.x + player_sprite_width / 2;
        let player_center_y = self.player.y + player_sprite_height / 2;

        let mut new_camera_x = player_center_x.saturating_sub(frame_size.width / 2);
        let mut new_camera_y = player_center_y.saturating_sub(frame_size.height / 2);

        // Clamp camera to map boundaries
        // Ensure camera does not go beyond the map's right/bottom edge
        new_camera_x = new_camera_x.min(current_map.width.saturating_sub(frame_size.width));
        new_camera_y = new_camera_y.min(current_map.height.saturating_sub(frame_size.height));

        // Ensure camera does not go below 0
        new_camera_x = new_camera_x.max(0);
        new_camera_y = new_camera_y.max(0);

        self.camera_x = new_camera_x;
        self.camera_y = new_camera_y;
    }

    pub fn get_combined_map_text(&self, _frame_size: ratatui::layout::Rect) -> Text<'static> {
        let current_map_key = (self.current_map_row, self.current_map_col);
        if let Some(map_chunk) = self.loaded_maps.get(&current_map_key) {
            map_chunk.ansi_sprite.as_bytes().into_text().unwrap_or_default()
        } else {
            Text::default()
        }
    }

    pub fn undo_wall_change(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            let current_map_key = (self.current_map_row, self.current_map_col);
            if let Some(map_to_modify) = self.loaded_maps.get_mut(&current_map_key) {
                map_to_modify.walls = self.wall_history[self.history_index].clone();
                if let Err(e) = map_to_modify.save_data() {
                    self.message = format!("Failed to save map data after undo: {}", e);
                    self.show_message = true;
                } else {
                    self.message = "Undid last wall change.".to_string();
                    self.show_message = true;
                }
            }
        } else {
            self.message = "No more changes to undo.".to_string();
            self.show_message = true;
        }
    }

    pub fn redo_wall_change(&mut self) {
        if self.history_index < self.wall_history.len() - 1 {
            self.history_index += 1;
            let current_map_key = (self.current_map_row, self.current_map_col);
            if let Some(map_to_modify) = self.loaded_maps.get_mut(&current_map_key) {
                map_to_modify.walls = self.wall_history[self.history_index].clone();
                if let Err(e) = map_to_modify.save_data() {
                    self.message = format!("Failed to save map data after redo: {}", e);
                    self.show_message = true;
                } else {
                    self.message = "Redid last wall change.".to_string();
                    self.show_message = true;
                }
            }
        } else {
            self.message = "No more changes to redo.".to_string();
            self.show_message = true;
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SaveData {
    player_x: u16,
    player_y: u16,
    current_map_name: String,
}