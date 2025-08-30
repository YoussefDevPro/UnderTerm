use super::map::Map;
use super::player::{Player, PlayerUpdateContext};
use ansi_to_tui::IntoText;
use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::text::Text;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self};
use std::time::{Duration, Instant};

fn default_instant() -> Instant {
    Instant::now()
}

fn default_duration() -> Duration {
    Duration::from_millis(50)
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TeleportCreationState {
    None,
    DrawingBox,
    EnteringMapName,
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
    pub message_animation_finished: bool,
    pub sound_error: Option<String>,
    #[serde(skip)]
    pub loaded_maps: std::collections::HashMap<(i32, i32), Map>,
    pub debug_mode: bool,
    pub show_collision_box: bool,
    pub current_interaction_box_id: Option<u32>,
    pub current_message_index: usize,
    pub is_drawing_select_box: bool,
    pub select_box_start_coords: Option<(u16, u16)>,
    pub is_confirming_select_box: bool,
    pub block_player_movement_on_message: bool,
    pub is_text_input_active: bool,
    pub text_input_buffer: String,
    pub pending_select_box: Option<crate::game::map::SelectObjectBox>,
    pub is_event_input_active: bool,
    pub is_map_kind_selection_active: bool,
    pub current_map_name: String,
    pub current_map_row: i32,
    pub current_map_col: i32,
    #[serde(skip)]
    pub wall_history: Vec<Vec<(u32, u32)>>,
    #[serde(skip)]
    pub history_index: usize,
    pub is_creating_map: bool,
    pub last_teleport_origin: Option<(u32, u32, i32, i32)>,
    pub recently_teleported_from_box_id: Option<u32>,
    pub teleport_creation_state: TeleportCreationState,
    pub teleport_destination_map_name_buffer: String,

    #[serde(skip)]
    pub esc_press_start_time: Option<Instant>,
    #[serde(skip)]
    pub debug_info: Vec<String>,
    #[serde(skip)]
    pub esc_hold_dots: u8,
    #[serde(skip, default = "default_instant")]
    pub esc_dot_timer: Instant,
}

#[derive(Serialize, Deserialize)]
struct SaveData {
    current_map_name: String,
}

impl GameState {
    pub fn from_map(map: Map) -> Self {
        let player_spawn_x = map.player_spawn.0;
        let player_spawn_y = map.player_spawn.1;

        let map_parts: Vec<&str> = map.name.split('_').collect();
        let current_map_row: i32 = map_parts[1].parse().unwrap_or(0);
        let current_map_col: i32 = map_parts[2].parse().unwrap_or(0);

        let mut loaded_maps = HashMap::new();
        loaded_maps.insert((current_map_row, current_map_col), map.clone());
        GameState {
            player: Player::new(player_spawn_x as u16, player_spawn_y as u16),
            camera_x: 0,
            camera_y: 0,
            message: String::new(),
            show_message: false,
            animated_message_content: String::new(),
            message_animation_start_time: Instant::now(),
            message_animation_speed: Duration::from_millis(50),
            message_animation_finished: false,
            current_map_name: format!("map_{}_{}", current_map_row, current_map_col),
            loaded_maps,
            debug_mode: false,
            show_collision_box: false,
            current_interaction_box_id: None,
            current_message_index: 0,
            is_drawing_select_box: false,
            select_box_start_coords: None,
            is_confirming_select_box: false,
            block_player_movement_on_message: true,
            is_text_input_active: false,
            text_input_buffer: String::new(),
            pending_select_box: None,
            is_event_input_active: false,
            is_map_kind_selection_active: false,
            sound_error: None,
            current_map_row,
            current_map_col,
            wall_history: vec![map.walls.clone()],
            history_index: 0,
            is_creating_map: false,
            last_teleport_origin: None,
            recently_teleported_from_box_id: None,
            teleport_destination_map_name_buffer: String::new(),

            teleport_creation_state: TeleportCreationState::None,
            esc_press_start_time: None,
            debug_info: Vec::new(),
            esc_hold_dots: 0,
            esc_dot_timer: Instant::now(),
        }
    }

    pub fn save_game_state(&mut self) -> io::Result<()> {
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
                let map = Map::load(&deserialized.current_map_name).map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, format!("Failed to load map: {}", e))
                })?;
                let game_state = GameState::from_map(map);
                Ok(game_state)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let map = Map::load("map_0_0").map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to load default map: {}", e),
                    )
                })?;
                let mut default_state = GameState::from_map(map);
                default_state.save_game_state()?;
                Ok(default_state)
            }
            Err(e) => Err(e),
        }
    }

    pub fn dismiss_message(&mut self) {
        self.show_message = false;
        self.message.clear();
        self.animated_message_content.clear();
        self.message_animation_finished = false;
        self.block_player_movement_on_message = false;
    }

    pub fn update(
        &mut self,
        key_states: &HashMap<KeyCode, bool>,
        frame_size: ratatui::layout::Rect,
        animation_frame_duration: std::time::Duration,
    ) {
        if self.show_message && self.message.is_empty() {
            self.dismiss_message();
        }
        if self.is_confirming_select_box
            && (*key_states.get(&KeyCode::Up).unwrap_or(&false)
                || *key_states.get(&KeyCode::Down).unwrap_or(&false)
                || *key_states.get(&KeyCode::Left).unwrap_or(&false)
                || *key_states.get(&KeyCode::Right).unwrap_or(&false))
        {
            self.dismiss_message();
        }

        if self.show_message {
            let elapsed = self.message_animation_start_time.elapsed();
            let chars_to_show =
                (elapsed.as_millis() / self.message_animation_speed.as_millis()) as usize;
            self.animated_message_content = self.message.chars().take(chars_to_show).collect();
        } else {
            self.animated_message_content.clear();
        }

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
            is_drawing_select_box: self.is_drawing_select_box,
            block_player_movement_on_message: &mut self.block_player_movement_on_message,
        };

        if !(*context.show_message && *context.block_player_movement_on_message) {
            self.player
                .update(&mut context, key_states, animation_frame_duration);

            if let Some(teleported_from_id) = self.recently_teleported_from_box_id {
                let current_map_key = (self.current_map_row, self.current_map_col);
                if let Some(current_map) = self.loaded_maps.get(&current_map_key) {
                    if let Some(teleport_box) = current_map
                        .select_object_boxes
                        .iter()
                        .find(|b| b.id == teleported_from_id)
                    {
                        let player_collision_rect = self.player.get_collision_rect();
                        if !teleport_box.to_rect().intersects(player_collision_rect) {
                            self.recently_teleported_from_box_id = None;
                        }
                    } else {
                        self.recently_teleported_from_box_id = None;
                    }
                } else {
                    self.recently_teleported_from_box_id = None;
                }
            }
        }

        let (_player_sprite_content, player_sprite_width, player_sprite_height) =
            self.player.get_sprite_content();

        let player_center_x = self.player.x + player_sprite_width / 2;
        let player_center_y = self.player.y + player_sprite_height / 2;

        let mut new_camera_x = player_center_x.saturating_sub(frame_size.width / 2);
        let mut new_camera_y = player_center_y.saturating_sub(frame_size.height / 2);

        let current_map_key = (self.current_map_row, self.current_map_col);
        if let Some(current_map) = self.loaded_maps.get(&current_map_key) {
            new_camera_x = new_camera_x.min(current_map.width.saturating_sub(frame_size.width));
            new_camera_y = new_camera_y.min(current_map.height.saturating_sub(frame_size.height));
        }

        new_camera_x = new_camera_x.max(0);
        new_camera_y = new_camera_y.max(0);

        self.camera_x = new_camera_x;
        self.camera_y = new_camera_y;

        let mut map_to_insert_after_loop: Option<((i32, i32), Map)> = None;

        if !self.debug_mode {
            let player_collision_rect = self.player.get_collision_rect();
            let mut teleport_destination: Option<(u16, u16, i32, i32, String)> = None;
            let mut interacting_with_box_this_frame = false;
            if let Some(current_map) = self
                .loaded_maps
                .get(&(self.current_map_row, self.current_map_col))
            {
                for select_box in &current_map.select_object_boxes {
                    if select_box.to_rect().intersects(player_collision_rect) {
                        interacting_with_box_this_frame = true;

                        if self.recently_teleported_from_box_id == Some(select_box.id) {
                            continue;
                        }

                        // If we are already interacting with this box, don't re-trigger messages
                        if self.current_interaction_box_id == Some(select_box.id) {
                            // If messages are exhausted, check for events
                            if self.current_message_index >= select_box.messages.len() {
                                for event in &select_box.events {
                                    match event {
                                        crate::game::map::Event::TeleportPlayer {
                                            map_row,
                                            map_col,
                                            dest_x,
                                            dest_y,
                                        } => {
                                            let new_map_name = format!("map_{}_{}", map_row, map_col);
                                            let new_map_key = (*map_row, *map_col);
                                            let mut loaded_map: Option<Map> = None;
                                            if !self.loaded_maps.contains_key(&new_map_key) {
                                                if let Ok(map) = crate::game::map::Map::load(&new_map_name) {
                                                    loaded_map = Some(map);
                                                } else {
                                                    self.message =
                                                        format!("Failed to load map: {}", new_map_name);
                                                    self.show_message = true;
                                                    self.message_animation_start_time = Instant::now();
                                                    self.animated_message_content.clear();
                                                    break;
                                                }
                                            }

                                            let map_is_available =
                                                self.loaded_maps.contains_key(&new_map_key)
                                                    || loaded_map.is_some();

                                            if let Some(map) = loaded_map {
                                                map_to_insert_after_loop = Some((new_map_key, map));
                                            }

                                            if map_is_available {
                                                teleport_destination = Some((
                                                    *dest_x as u16,
                                                    *dest_y as u16,
                                                    *map_row,
                                                    *map_col,
                                                    new_map_name,
                                                ));
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                            // If a teleport event was found, break from this inner loop
                            if teleport_destination.is_some() {
                                break;
                            }
                            continue; // Already interacting, no new message to show
                        }

                        // New interaction or interaction with a different box
                        if !select_box.messages.is_empty() {
                            self.current_interaction_box_id = Some(select_box.id);
                            self.current_message_index = 0;
                            // Messages will be displayed on Enter key press
                            // self.message = select_box.messages[0].clone();
                            // self.show_message = true;
                            // self.message_animation_start_time = Instant::now();
                            // self.animated_message_content.clear();
                            // self.block_player_movement_on_message = true;
                            // break; // Found an interaction with messages, stop checking other boxes
                        } else {
                            // If no messages, check for events immediately
                            for event in &select_box.events {
                                match event {
                                    crate::game::map::Event::TeleportPlayer {
                                        map_row,
                                        map_col,
                                        dest_x,
                                        dest_y,
                                    } => {
                                        let new_map_name = format!("map_{}_{}", map_row, map_col);
                                        let new_map_key = (*map_row, *map_col);
                                        let mut loaded_map: Option<Map> = None;
                                        if !self.loaded_maps.contains_key(&new_map_key) {
                                            if let Ok(map) = crate::game::map::Map::load(&new_map_name) {
                                                loaded_map = Some(map);
                                            } else {
                                                self.message =
                                                    format!("Failed to load map: {}", new_map_name);
                                                self.show_message = true;
                                                self.message_animation_start_time = Instant::now();
                                                self.animated_message_content.clear();
                                                break;
                                            }
                                        }

                                        let map_is_available =
                                            self.loaded_maps.contains_key(&new_map_key)
                                                || loaded_map.is_some();

                                        if let Some(map) = loaded_map {
                                            map_to_insert_after_loop = Some((new_map_key, map));
                                        }

                                        if map_is_available {
                                            teleport_destination = Some((
                                                *dest_x as u16,
                                                *dest_y as u16,
                                                *map_row,
                                                *map_col,
                                                new_map_name,
                                            ));
                                        }
                                        break;
                                    }
                                }
                            }
                            if teleport_destination.is_some() {
                                break;
                            }
                        }
                    }
                }
            }

            // If no interaction box was found intersecting this frame, clear current_interaction_box_id
            if !interacting_with_box_this_frame {
                self.current_interaction_box_id = None;
                self.current_message_index = 0; // Reset message index
            }

            if let Some((x, y, map_row, map_col, new_map_name)) = teleport_destination {
                self.last_teleport_origin = Some((
                    self.player.x as u32,
                    self.player.y as u32,
                    self.current_map_row,
                    self.current_map_col,
                ));

                self.player.x = x;
                self.player.y = y;
                self.current_map_row = map_row;
                self.current_map_col = map_col;
                self.current_map_name = new_map_name;

                let player_collision_rect = self.player.get_collision_rect();
                let dest_map_key = (self.current_map_row, self.current_map_col);

                let dest_map_option = if let Some((key, map)) = &map_to_insert_after_loop {
                    if *key == dest_map_key {
                        Some(map)
                    } else {
                        self.loaded_maps.get(&dest_map_key)
                    }
                } else {
                    self.loaded_maps.get(&dest_map_key)
                };

                if let Some(dest_map) = dest_map_option {
                    let mut landed_in_teleporter = false;
                    for select_box in &dest_map.select_object_boxes {
                        if select_box.to_rect().intersects(player_collision_rect) {
                            let is_tp = select_box
                                .events
                                .iter()
                                .any(|e| matches!(e, crate::game::map::Event::TeleportPlayer { .. }));
                            if is_tp {
                                self.recently_teleported_from_box_id = Some(select_box.id);
                                landed_in_teleporter = true;
                                break;
                            }
                        }
                    }
                    if !landed_in_teleporter {
                        self.recently_teleported_from_box_id = None;
                    }
                } else {
                    self.recently_teleported_from_box_id = None;
                }
            }
        }

        if let Some((key, map)) = map_to_insert_after_loop {
            self.loaded_maps.insert(key, map);
        }

        self.debug_info.clear();
        let player_collision_rect = self.player.get_collision_rect();
        self.debug_info
            .push(format!("Player Collision Box: {:?}", player_collision_rect));
        if let Some(current_map) = self
            .loaded_maps
            .get(&(self.current_map_row, self.current_map_col))
        {
            for select_box in &current_map.select_object_boxes {
                let intersects = select_box.to_rect().intersects(player_collision_rect);
                let is_tp = select_box
                    .events
                    .iter()
                    .any(|e| matches!(e, crate::game::map::Event::TeleportPlayer { .. }));
                self.debug_info.push(format!(
                    "Box ID {}: {:?}, TP: {}, Intersects: {}",
                    select_box.id,
                    select_box.to_rect(),
                    is_tp,
                    intersects
                ));
            }
        }

        if self.esc_press_start_time.is_some() {
            if self.esc_dot_timer.elapsed() >= std::time::Duration::from_secs(1) {
                if self.esc_hold_dots < 4 {
                    self.esc_hold_dots += 1;
                }
                self.esc_dot_timer = Instant::now();
            }
        } else if self.esc_hold_dots > 0 {
            if self.esc_dot_timer.elapsed() >= std::time::Duration::from_secs(1) {
                self.esc_hold_dots -= 1;
                self.esc_dot_timer = Instant::now();
            }
        }
    }

    pub fn undo_wall_change(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            let current_map_key = (self.current_map_row, self.current_map_col);
            if let Some(map) = self.loaded_maps.get_mut(&current_map_key) {
                map.walls = self.wall_history[self.history_index].clone();
            }
        }
    }

    pub fn redo_wall_change(&mut self) {
        if self.history_index < self.wall_history.len() - 1 {
            self.history_index += 1;
            let current_map_key = (self.current_map_row, self.current_map_col);
            if let Some(map) = self.loaded_maps.get_mut(&current_map_key) {
                map.walls = self.wall_history[self.history_index].clone();
            }
        }
    }

    pub fn set_player_spawn_to_current_position(&mut self, x: u16, y: u16) {
        let current_map_key = (self.current_map_row, self.current_map_col);
        if let Some(map) = self.loaded_maps.get_mut(&current_map_key) {
            map.player_spawn = (x as u32, y as u32);
            if let Err(e) = map.save_data() {
                self.message = format!("Failed to save map data: {}", e);
                self.show_message = true;
                self.message_animation_start_time = Instant::now();
                self.animated_message_content.clear();
                self.message_animation_finished = false;
            } else {
                self.message = "Spawn point saved.".to_string();
                self.show_message = true;
                self.message_animation_start_time = Instant::now();
                self.animated_message_content.clear();
                self.message_animation_finished = false;
            }
        }
    }

    pub fn skip_message_animation(&mut self) {
        self.animated_message_content = self.message.clone();
        self.message_animation_finished = true;
    }

    pub fn get_combined_map_text(&self, _frame_size: Rect) -> Text<'static> {
        let current_map_key = (self.current_map_row, self.current_map_col);
        if let Some(map) = self.loaded_maps.get(&current_map_key) {
            map.ansi_sprite.as_bytes().into_text().unwrap()
        } else {
            Text::default()
        }
    }
}