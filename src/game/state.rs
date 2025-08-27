use super::player::{Player, PlayerUpdateContext};
use super::map::Map;
use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};
use std::io::{self};
use std::collections::HashMap;
use ratatui::text::Text;
use std::time::{Duration, Instant};
use ratatui::layout::Rect;

fn default_instant() -> Instant {
    Instant::now()
}

fn default_duration() -> Duration {
    Duration::from_millis(50) // Default animation speed
}

use ansi_to_tui::IntoText;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeleportZone {
    pub rect: Rect,
    pub target_map_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub camera_x: u16,
    pub camera_y: u16,
    pub message: String,
    pub show_message: bool,
    pub animated_message_content: String,
    #[serde(skip, default = "default_instant")]
    pub message_animation_start_time: Instant,
    #[serde(skip, default = "default_duration")]
    pub message_animation_speed: Duration,
    #[serde(skip)]
    pub message_animation_finished: bool, // Added
    pub show_debug_panel: bool, // Added this line
    pub sound_error: Option<String>, // New field for sound errors
    #[serde(skip)]
    pub loaded_maps: std::collections::HashMap<(i32, i32), Map>,
    pub debug_mode: bool,
    pub show_collision_box: bool, // Added
    pub current_interaction_box_id: Option<u32>, // Added
    pub current_message_index: usize, // Added
    pub is_drawing_select_box: bool, // Added
    pub select_box_start_coords: Option<(u16, u16)>, // Added
    pub is_drawing_teleport_zone: bool, // Added
    pub teleport_zone_start_coords: Option<(u16, u16)>, // Added
    pub is_text_input_active: bool, // Added
    pub text_input_buffer: String, // Added
    pub pending_select_box: Option<crate::game::map::SelectObjectBox>, // Added
    pub is_event_input_active: bool, // Added
    pub is_map_kind_selection_active: bool, // Added
    pub current_map_name: String,
    pub current_map_row: i32,
    pub current_map_col: i32,
    #[serde(skip)]
    pub wall_history: Vec<Vec<(u32, u32)>>,
    #[serde(skip)]
    pub history_index: usize,
    pub is_creating_map: bool,
    pub is_teleport_input_active: bool, // Added
    pub teleport_zones: Vec<TeleportZone>,
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
            animated_message_content: String::new(),
            message_animation_start_time: Instant::now(),
            message_animation_speed: Duration::from_millis(50), // Default speed
            message_animation_finished: false, // Initialize
            current_map_name: format!("map_{}_{}", current_map_row, current_map_col),
            loaded_maps,
            debug_mode: false,
            show_debug_panel: false, // Added this line
            show_collision_box: false, // Initialize
            current_interaction_box_id: None, // Initialize
            current_message_index: 0, // Initialize
            is_drawing_select_box: false, // Initialize
            select_box_start_coords: None, // Initialize
            is_text_input_active: false, // Initialize
            text_input_buffer: String::new(), // Initialize
            pending_select_box: None, // Initialize
            is_event_input_active: false, // Initialize
            is_map_kind_selection_active: false, // Initialize
            sound_error: None, // Initialize sound_error
            current_map_row,
            current_map_col,
            wall_history: vec![map.walls.clone()], // Initialize with current map walls
            history_index: 0,
            is_creating_map: false,
            is_teleport_input_active: false, // Initialize
            is_drawing_teleport_zone: false, // Initialize
            teleport_zone_start_coords: None, // Initialize
            teleport_zones: map.teleport_zones, // Use map.teleport_zones
        }
    }

    pub fn save_game_state(&self) -> io::Result<()> {
        let save_data = SaveData {
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
                let game_state = GameState::from_map(map); // Player will be initialized at map's spawn
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

    pub fn update(&mut self, key_states: &HashMap<KeyCode, bool>, frame_size: ratatui::layout::Rect, animation_frame_duration: std::time::Duration) {
        // Update animated message content
        if self.show_message {
            let elapsed = self.message_animation_start_time.elapsed();
            let chars_to_show = (elapsed.as_millis() / self.message_animation_speed.as_millis()) as usize;
            self.animated_message_content = self.message.chars().take(chars_to_show).collect();
        } else {
            self.animated_message_content.clear();
        }

        // If the animation is complete, set flag
        if self.animated_message_content.len() == self.message.len() {
            self.message_animation_finished = true;
        } else {
            self.message_animation_finished = false;
        }

        let mut context = PlayerUpdateContext {
            current_map_row: &mut self.current_map_row,
            current_map_col: &mut self.current_map_col,
            loaded_maps: &mut self.loaded_maps,
            debug_mode: self.debug_mode,
            message: &mut self.message,
            show_message: &mut self.show_message,
            animated_message_content: &mut self.animated_message_content,
            message_animation_start_time: &mut self.message_animation_start_time,
            wall_history: &mut self.wall_history,
            history_index: &mut self.history_index,
            current_map_name: &mut self.current_map_name,
            is_drawing_select_box: self.is_drawing_select_box, // Pass the flag
            is_drawing_teleport_zone: self.is_drawing_teleport_zone, // Pass the flag
        };

        // Prevent player movement if a message is being displayed
        if !*context.show_message {
            self.player.update(&mut context, key_states, animation_frame_duration);
        }

        // Teleportation logic
        let (_player_sprite_content, _player_sprite_width, player_sprite_height) = self.player.get_sprite_content();
        const COLLISION_BOX_WIDTH: u16 = 21;
        const COLLISION_BOX_HEIGHT: u16 = 4;
        let player_collision_box = ratatui::layout::Rect::new(
            self.player.x,
            self.player.y.saturating_add(player_sprite_height).saturating_sub(COLLISION_BOX_HEIGHT),
            COLLISION_BOX_WIDTH,
            COLLISION_BOX_HEIGHT,
        );

        let mut teleport_target: Option<(String, u16, u16)> = None;

        for zone in &self.teleport_zones {
            if player_collision_box.intersects(zone.rect) {
                teleport_target = Some((zone.target_map_id.clone(), zone.rect.x, zone.rect.y));
                break;
            }
        }

        if let Some((target_map_id, target_x, target_y)) = teleport_target {
            // Parse map_row and map_col from target_map_id
            let map_parts: Vec<&str> = target_map_id.split('_').collect();
            if map_parts.len() == 3 && map_parts[0] == "map" {
                if let (Ok(map_row), Ok(map_col)) = (map_parts[1].parse::<i32>(), map_parts[2].parse::<i32>()) {
                    self.current_map_row = map_row;
                    self.current_map_col = map_col;
                    self.current_map_name = target_map_id.clone();

                    // Load the new map if not already loaded
                    let new_map_key = (map_row, map_col);
                    if !self.loaded_maps.contains_key(&new_map_key) {
                        match Map::load(&target_map_id) {
                            Ok(new_map) => {
                                self.loaded_maps.insert(new_map_key, new_map);
                            },
                            Err(e) => {
                                self.message = format!("Failed to load map for teleport: {}", e);
                                self.show_message = true;
                                self.message_animation_start_time = Instant::now();
                                self.animated_message_content.clear();
                            }
                        }
                    }

                    // Set player position to the target map's spawn coordinates
                    if let Some(new_map) = self.loaded_maps.get(&new_map_key) {
                        self.player.x = new_map.player_spawn.0 as u16;
                        self.player.y = new_map.player_spawn.1 as u16;
                    } else {
                        // Fallback if new map not found (shouldn't happen if Map::load was successful)
                        self.player.x = target_x;
                        self.player.y = target_y;
                    }

                    
                }
            }
        }

        // Re-implement continuous camera logic
        // Calculate the desired camera position to center the player
        // Consider player sprite's center for more accurate centering
        let (_player_sprite_content, player_sprite_width, player_sprite_height) = self.player.get_sprite_content();

        let player_center_x = self.player.x + player_sprite_width / 2;
        let player_center_y = self.player.y + player_sprite_height / 2;

        let mut new_camera_x = player_center_x.saturating_sub(frame_size.width / 2);
        let mut new_camera_y = player_center_y.saturating_sub(frame_size.height / 2);

        let current_map_key = (self.current_map_row, self.current_map_col);
        if let Some(current_map) = self.loaded_maps.get(&current_map_key) {
            // Clamp camera to map boundaries
            // Ensure camera does not go beyond the map's right/bottom edge
            new_camera_x = new_camera_x.min(current_map.width.saturating_sub(frame_size.width));
            new_camera_y = new_camera_y.min(current_map.height.saturating_sub(frame_size.height));
        }


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
                    self.message_animation_start_time = Instant::now();
                    self.animated_message_content.clear();
                    self.message_animation_finished = false; // Reset
                } else {
                    self.message = "Undid last wall change.".to_string();
                    self.show_message = true;
                    self.message_animation_start_time = Instant::now();
                    self.animated_message_content.clear();
                    self.message_animation_finished = false; // Reset
                }
            }
        } else {
            self.message = "No more changes to undo.".to_string();
            self.show_message = true;
            self.message_animation_start_time = Instant::now();
            self.animated_message_content.clear();
            self.message_animation_finished = false; // Reset
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
                    self.message_animation_start_time = Instant::now();
                    self.animated_message_content.clear();
                    self.message_animation_finished = false; // Reset
                } else {
                    self.message = "Redid last wall change.".to_string();
                    self.show_message = true;
                    self.message_animation_start_time = Instant::now();
                    self.animated_message_content.clear();
                    self.message_animation_finished = false; // Reset
                }
            }
        } else {
            self.message = "No more changes to redo.".to_string();
            self.show_message = true;
            self.message_animation_start_time = Instant::now();
            self.animated_message_content.clear();
            self.message_animation_finished = false; // Reset
        }
    }

    pub fn set_player_spawn_to_current_position(&mut self, player_x: u16, player_y: u16) {
        let current_map_key = (self.current_map_row, self.current_map_col);
        if let Some(map_to_modify) = self.loaded_maps.get_mut(&current_map_key) {
            map_to_modify.player_spawn = (player_x as u32, player_y as u32);
            if let Err(e) = map_to_modify.save_data() {
                self.message = format!("Failed to save map data: {}", e);
                self.show_message = true;
                self.message_animation_start_time = Instant::now();
                self.animated_message_content.clear();
                self.message_animation_finished = false; // Reset
            } else {
                self.message = format!("Spawn point set to ({}, {}) and saved.", player_x, player_y);
                self.show_message = true;
                self.message_animation_start_time = Instant::now();
                self.animated_message_content.clear();
                self.message_animation_finished = false; // Reset
            }
        } else {
            self.message = "Could not find current map to set spawn point.".to_string();
            self.show_message = true;
            self.message_animation_start_time = Instant::now();
            self.animated_message_content.clear();
            self.message_animation_finished = false; // Reset
        }
    }

    pub fn skip_message_animation(&mut self) {
        self.animated_message_content = self.message.clone();
        self.message_animation_start_time = Instant::now() - self.message_animation_speed * (self.message.len() as u32); // Set time to end of animation
    }

    pub fn dismiss_message(&mut self) {
        if self.message_animation_finished { // Only dismiss if animation is finished
            self.show_message = false;
            self.animated_message_content.clear();
            self.message.clear();
        } else { // If animation is not finished, skip it
            self.skip_message_animation();
        }
    }

    pub fn get_teleport_zones(&self) -> &Vec<TeleportZone> {
        &self.teleport_zones
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SaveData {
    current_map_name: String,
}
