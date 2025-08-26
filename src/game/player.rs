use ansi_to_tui::IntoText;
use ratatui::text::Text;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// bring in new imports
use super::config::{PLAYER_HORIZONTAL_SPEED, PLAYER_SPEED};
use super::map::Map;
use crossterm::event::KeyCode;

// Helper function for serde default
fn default_instant() -> Instant {
    Instant::now()
}

// Enum to represent player direction
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum PlayerDirection {
    Front,
    Back,
    Left,
    Right,
}

// Struct to hold player state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub x: u16,
    pub y: u16,
    pub direction: PlayerDirection,
    pub animation_frame: u8,
    pub is_walking: bool,
    #[serde(skip, default = "default_instant")]
    pub animation_timer: Instant,
}

pub struct PlayerUpdateContext<'a> {
    pub current_map_row: &'a mut i32,
    pub current_map_col: &'a mut i32,
    pub loaded_maps: &'a mut HashMap<(i32, i32), Map>,
    pub debug_mode: bool,
    pub message: &'a mut String,
    pub show_message: &'a mut bool,
    pub animated_message_content: &'a mut String, // Added
    pub message_animation_start_time: &'a mut Instant, // Added
    pub wall_history: &'a mut Vec<Vec<(u32, u32)>>,
    pub history_index: &'a mut usize,
    pub current_map_name: &'a mut String,
}

impl Player {
    pub fn new(x: u16, y: u16) -> Self {
        Player {
            x,
            y,
            direction: PlayerDirection::Front, // Default direction
            animation_frame: 0,
            is_walking: false,
            animation_timer: Instant::now(), // Initialize timer
        }
    }

    pub fn get_sprite_content(&self) -> (Text<'static>, u16, u16) {
        let sprite_path = self.get_sprite_path();
        let content = std::fs::read_to_string(&sprite_path).unwrap_or_else(|_| "P".to_string());
        let text = content.as_bytes().into_text().unwrap();

        let height = text.lines.len() as u16;
        let mut max_width = 0;
        for line in text.lines.iter() {
            let line_width = line.width() as u16;
            if line_width > max_width {
                max_width = line_width;
            }
        }
        (text, max_width, height)
    }

    pub fn get_sprite_path(&self) -> String {
        let base_path = "assets/sprites/frisk/";
        let (sub_dir, anim_file) = match self.direction {
            PlayerDirection::Front => {
                if self.is_walking {
                    match self.animation_frame {
                        0 => ("idle", "frisk_idle_front.ans"),
                        1 => ("walk", "frisk_walk_front_1.ans"),
                        2 => ("idle", "frisk_idle_front.ans"),
                        3 => ("walk", "frisk_walk_front_2.ans"),
                        _ => ("idle", "frisk_idle_front.ans"), // Fallback
                    }
                } else {
                    ("idle", "frisk_idle_front.ans")
                }
            }
            PlayerDirection::Back => {
                if self.is_walking {
                    match self.animation_frame {
                        0 => ("idle", "frisk_idle_back.ans"),
                        1 => ("walk", "frisk_walk_back_1.ans"),
                        2 => ("idle", "frisk_idle_back.ans"),
                        3 => ("walk", "frisk_walk_back_2.ans"),
                        _ => ("idle", "frisk_idle_back.ans"), // Fallback
                    }
                } else {
                    ("idle", "frisk_idle_back.ans")
                }
            }
            PlayerDirection::Left => {
                if self.is_walking {
                    match self.animation_frame {
                        0 => ("idle", "frisk_idle_left.ans"),
                        1 => ("walk", "frisk_walk_left.ans"),
                        _ => ("idle", "frisk_idle_left.ans"), // Fallback
                    }
                } else {
                    ("idle", "frisk_idle_left.ans")
                }
            }
            PlayerDirection::Right => {
                if self.is_walking {
                    match self.animation_frame {
                        0 => ("idle", "frisk_idle_right.ans"),
                        1 => ("walk", "frisk_walk_right.ans"),
                        _ => ("idle", "frisk_idle_right.ans"), // Fallback
                    }
                } else {
                    ("idle", "frisk_idle_right.ans")
                }
            }
        };
        format!("{}{}/{}", base_path, sub_dir, anim_file)
    }

    pub fn update_animation(&mut self, animation_frame_duration: Duration) {
        if self.is_walking {
            if self.animation_timer.elapsed() >= animation_frame_duration {
                match self.direction {
                    PlayerDirection::Front | PlayerDirection::Back => {
                        self.animation_frame = (self.animation_frame + 1) % 4; // 0, 1, 2, 3
                    }
                    PlayerDirection::Left | PlayerDirection::Right => {
                        self.animation_frame = (self.animation_frame + 1) % 2; // 0, 1
                    }
                }
                self.animation_timer = Instant::now(); // Reset timer
            }
        } else {
            self.animation_frame = 0; // reset
            self.animation_timer = Instant::now(); // Reset timer when idle
        }
    }

    pub fn update(
        &mut self,
        context: &mut PlayerUpdateContext,
        key_codes: Vec<KeyCode>,
        animation_frame_duration: Duration,
    ) {

        let current_map_key = (*context.current_map_row, *context.current_map_col);
        let current_map = context.loaded_maps.get(&current_map_key).cloned().unwrap();

        let mut new_player_x = self.x;
        let mut new_player_y = self.y;
        let mut moved_this_frame = false;

        let up = key_codes.contains(&KeyCode::Up);
        let down = key_codes.contains(&KeyCode::Down);
        let left = key_codes.contains(&KeyCode::Left);
        let right = key_codes.contains(&KeyCode::Right);

        let move_speed = if context.debug_mode { 1 } else { PLAYER_SPEED };
        let horizontal_move_speed = if context.debug_mode { 1 } else { PLAYER_HORIZONTAL_SPEED };

        if up && !down {
            new_player_y = new_player_y.saturating_sub(move_speed);
            self.direction = PlayerDirection::Back;
            moved_this_frame = true;
        } else if down && !up {
            new_player_y = new_player_y.saturating_add(move_speed);
            self.direction = PlayerDirection::Front;
            moved_this_frame = true;
        }

        if left && !right {
            new_player_x = new_player_x.saturating_sub(horizontal_move_speed);
            if !up && !down {
                self.direction = PlayerDirection::Left;
            }
            moved_this_frame = true;
        } else if right && !left {
            new_player_x = new_player_x.saturating_add(horizontal_move_speed);
            if !up && !down {
                self.direction = PlayerDirection::Right;
            }
            moved_this_frame = true;
        }

        if key_codes.contains(&KeyCode::Enter) {
            if context.debug_mode {
                let current_map_key = (*context.current_map_row, *context.current_map_col);
                if let Some(map_to_modify) = context.loaded_maps.get_mut(&current_map_key) {
                    map_to_modify.toggle_wall(self.x as u32, self.y as u32);
                    if let Err(e) = map_to_modify.save_data() {
                        *context.message = format!("Failed to save map data: {}", e);
                        *context.show_message = true;
                        *context.message_animation_start_time = Instant::now();
                        context.animated_message_content.clear();
                    } else {
                        *context.message =
                            format!("Toggled wall at ({}, {}) and saved.", self.x, self.y);
                        *context.show_message = true;
                        *context.message_animation_start_time = Instant::now();
                        context.animated_message_content.clear();
                    }
                    context.wall_history.truncate(*context.history_index + 1);
                    context.wall_history.push(map_to_modify.walls.clone());
                    *context.history_index = context.wall_history.len() - 1;
                }
            }
        }

        if moved_this_frame {
            if !self.is_walking {
                self.animation_frame = 1;
                self.animation_timer = Instant::now();
            }
            self.is_walking = true;
        }

        let mut transitioned = false;
        let mut next_map_row = *context.current_map_row;
        let mut next_map_col = *context.current_map_col;

        let original_player_x = self.x;
        let original_player_y = self.y;

        if new_player_x >= current_map.width {
            next_map_col += 1;
            new_player_x = 0;
            transitioned = true;
        } else if self.x == 0 && left {
            if *context.current_map_col > 0 {
                next_map_col -= 1;
                new_player_x = current_map.width - 1;
                transitioned = true;
            } else {
                new_player_x = 0;
            }
        }

        if new_player_y >= current_map.height {
            next_map_row += 1;
            new_player_y = 0;
            transitioned = true;
        } else if self.y == 0 && up {
            if *context.current_map_row > 0 {
                next_map_row -= 1;
                new_player_y = current_map.height - 1;
                transitioned = true;
            } else {
                new_player_y = 0;
            }
        }

        if transitioned {
            let new_map_name = format!("map_{}_{}", next_map_row, next_map_col);
            let new_map_key = (next_map_row, next_map_col);

            if !context.loaded_maps.contains_key(&new_map_key) {
                match Map::load(&new_map_name) {
                    Ok(new_map) => {
                        context.loaded_maps.insert(new_map_key, new_map);
                    }
                    Err(e) => {
                        *context.message = format!("Cannot enter {}: {}", new_map_name, e);
                        *context.show_message = true;
                        *context.message_animation_start_time = Instant::now();
                        context.animated_message_content.clear();
                        self.x = original_player_x;
                        self.y = original_player_y;
                        return;
                    }
                }
            }
            *context.current_map_name = new_map_name;
            *context.current_map_row = next_map_row;
            *context.current_map_col = next_map_col;
            self.x = new_player_x;
            self.y = new_player_y;
            *context.message = format!("Entered {}", *context.current_map_name);
            *context.show_message = true;
            *context.message_animation_start_time = Instant::now();
            context.animated_message_content.clear();
        }

        let (_, player_sprite_width, player_sprite_height) = self.get_sprite_content();

        if !context.debug_mode { // Only apply collision detection if not in debug mode
            let collision_box_height = 4;
            let collision_box_start_y = new_player_y
                .saturating_add(player_sprite_height)
                .saturating_sub(collision_box_height);

            let mut collision = false;
            for y_offset in 0..collision_box_height {
                let check_y = collision_box_start_y.saturating_add(y_offset);
                for x_offset in 0..player_sprite_width {
                    let check_x = new_player_x.saturating_add(x_offset);

                    let collision_map_key = (*context.current_map_row, *context.current_map_col);
                    if let Some(collision_map) = context.loaded_maps.get(&collision_map_key) {
                        if collision_map
                            .walls
                            .contains(&(check_x as u32, check_y as u32))
                        {
                            collision = true;
                            break;
                        }
                    }
                }
                if collision {
                    break;
                }
            }

            if collision {
                *context.message = "You hit a wall!".to_string();
                *context.show_message = true;
                *context.message_animation_start_time = Instant::now();
                context.animated_message_content.clear();
            } else {
                self.x = new_player_x.min(current_map.width.saturating_sub(player_sprite_width));
                self.y = new_player_y.min(current_map.height.saturating_sub(player_sprite_height));
            }
        } else { // In debug mode, just update player position without collision
            self.x = new_player_x;
            self.y = new_player_y;
        }
        self.update_animation(animation_frame_duration);
    }
}
