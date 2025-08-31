use ansi_to_tui::IntoText;
use ratatui::text::Text;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const PLAYER_COLLISION_WIDTH: u16 = 21;
const PLAYER_COLLISION_HEIGHT: u16 = 5;

use super::config::{
    PLAYER_HORIZONTAL_SPEED, PLAYER_INTERACTION_BOX_HEIGHT, PLAYER_INTERACTION_BOX_WIDTH,
    PLAYER_SPEED,
};
use super::map::Map;
use crossterm::event::KeyCode;

fn default_instant() -> Instant {
    Instant::now()
}

fn default_walking_stop_delay() -> Duration {
    Duration::from_millis(100)
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum PlayerDirection {
    Front,
    Back,
    Left,
    Right,
    FrontLeft,
    FrontRight,
    BackLeft,
    BackRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub direction: PlayerDirection,
    pub animation_frame: u8,
    pub is_walking: bool,
    #[serde(skip, default = "default_instant")]
    pub animation_timer: Instant,
    #[serde(skip, default = "default_instant")]
    pub walking_stop_timer: Instant,
    #[serde(skip, default = "default_walking_stop_delay")]
    pub walking_stop_delay: Duration,
}

pub struct PlayerUpdateContext<'a> {
    pub current_map_row: &'a mut i32,
    pub current_map_col: &'a mut i32,
    pub loaded_maps: &'a mut HashMap<(i32, i32), Map>,
    pub debug_mode: bool,
    pub message: &'a mut String,
    pub show_message: &'a mut bool,
    pub animated_message_content: &'a mut String,
    pub message_animation_start_time: &'a mut Instant,
    pub wall_history: &'a mut Vec<Vec<(u32, u32)>>,
    pub history_index: &'a mut usize,
    pub is_drawing_select_box: bool,
    pub block_player_movement_on_message: &'a mut bool,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Player {
            x,
            y,
            direction: PlayerDirection::Front,
            animation_frame: 0,
            is_walking: false,
            animation_timer: Instant::now(),
            walking_stop_timer: Instant::now(),
            walking_stop_delay: Duration::from_millis(100),
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
                        _ => ("idle", "frisk_idle_front.ans"),
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
                        _ => ("idle", "frisk_idle_back.ans"),
                    }
                } else {
                    ("idle", "frisk_idle_back.ans")
                }
            }
            PlayerDirection::Left | PlayerDirection::FrontLeft | PlayerDirection::BackLeft => {
                if self.is_walking {
                    match self.animation_frame {
                        0 => ("idle", "frisk_idle_left.ans"),
                        1 => ("walk", "frisk_walk_left.ans"),
                        _ => ("idle", "frisk_idle_left.ans"),
                    }
                } else {
                    ("idle", "frisk_idle_left.ans")
                }
            }
            PlayerDirection::Right | PlayerDirection::FrontRight | PlayerDirection::BackRight => {
                if self.is_walking {
                    match self.animation_frame {
                        0 => ("idle", "frisk_idle_right.ans"),
                        1 => ("walk", "frisk_walk_right.ans"),
                        _ => ("idle", "frisk_idle_right.ans"),
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
                        self.animation_frame = (self.animation_frame + 1) % 4;
                    }
                    PlayerDirection::Left
                    | PlayerDirection::Right
                    | PlayerDirection::FrontLeft
                    | PlayerDirection::FrontRight
                    | PlayerDirection::BackLeft
                    | PlayerDirection::BackRight => {
                        self.animation_frame = (self.animation_frame + 1) % 2;
                    }
                }
                self.animation_timer = Instant::now();
            }
        } else {
            self.animation_frame = 0;
            self.animation_timer = Instant::now();
        }
    }

    pub fn get_collision_rect(&self) -> ratatui::layout::Rect {
        let (_, player_sprite_width, player_sprite_height) = self.get_sprite_content();

        let collision_box_x = (self.x as u16)
            .saturating_add(player_sprite_width / 2)
            .saturating_sub(PLAYER_COLLISION_WIDTH / 2);
        let collision_box_y = (self.y as u16)
            .saturating_add(player_sprite_height)
            .saturating_sub(PLAYER_COLLISION_HEIGHT);

        ratatui::layout::Rect::new(
            collision_box_x,
            collision_box_y,
            PLAYER_COLLISION_WIDTH,
            PLAYER_COLLISION_HEIGHT,
        )
    }

    pub fn get_interaction_rect(&self) -> ratatui::layout::Rect {
        let (_, player_sprite_width, player_sprite_height) = self.get_sprite_content();

        let (select_box_x, select_box_y) = match self.direction {
            PlayerDirection::Front | PlayerDirection::FrontLeft | PlayerDirection::FrontRight => (
                (self.x as u16)
                    .saturating_add(player_sprite_width / 2)
                    .saturating_sub(PLAYER_INTERACTION_BOX_WIDTH / 2),
                (self.y as u16).saturating_add(player_sprite_height),
            ),
            PlayerDirection::Back | PlayerDirection::BackLeft | PlayerDirection::BackRight => (
                (self.x as u16)
                    .saturating_add(player_sprite_width / 2)
                    .saturating_sub(PLAYER_INTERACTION_BOX_WIDTH / 2),
                (self.y as u16).saturating_sub(PLAYER_INTERACTION_BOX_HEIGHT),
            ),
            PlayerDirection::Left => (
                (self.x as u16).saturating_sub(PLAYER_INTERACTION_BOX_WIDTH),
                (self.y as u16)
                    .saturating_add(player_sprite_height / 2)
                    .saturating_sub(PLAYER_INTERACTION_BOX_HEIGHT / 2),
            ),
            PlayerDirection::Right => (
                (self.x as u16).saturating_add(player_sprite_width),
                (self.y as u16)
                    .saturating_add(player_sprite_height / 2)
                    .saturating_sub(PLAYER_INTERACTION_BOX_HEIGHT / 2),
            ),
        };

        ratatui::layout::Rect::new(
            select_box_x,
            select_box_y,
            PLAYER_INTERACTION_BOX_WIDTH,
            PLAYER_INTERACTION_BOX_HEIGHT,
        )
    }

    pub fn update(
        &mut self,
        context: &mut PlayerUpdateContext,
        key_states: &HashMap<KeyCode, bool>,
        animation_frame_duration: Duration,
    ) {
        let current_map_key = (*context.current_map_row, *context.current_map_col);
        let current_map = context.loaded_maps.get(&current_map_key).cloned().unwrap();

        let mut new_player_x = self.x;
        let mut new_player_y = self.y;
        let original_player_x = self.x;
        let original_player_y = self.y;

        let up = *key_states.get(&KeyCode::Up).unwrap_or(&false);
        let down = *key_states.get(&KeyCode::Down).unwrap_or(&false);
        let left = *key_states.get(&KeyCode::Left).unwrap_or(&false);
        let right = *key_states.get(&KeyCode::Right).unwrap_or(&false);

        if up && !down && left && !right {
            self.direction = PlayerDirection::BackLeft;
        } else if up && !down && right && !left {
            self.direction = PlayerDirection::BackRight;
        } else if down && !up && left && !right {
            self.direction = PlayerDirection::FrontLeft;
        } else if down && !up && right && !left {
            self.direction = PlayerDirection::FrontRight;
        } else if up && !down {
            self.direction = PlayerDirection::Back;
        } else if down && !up {
            self.direction = PlayerDirection::Front;
        } else if left && !right {
            self.direction = PlayerDirection::Left;
        } else if right && !left {
            self.direction = PlayerDirection::Right;
        }

        let move_speed = if context.debug_mode { 1.0 } else { PLAYER_SPEED };
        let horizontal_move_speed = if context.debug_mode { 1.0 } else { PLAYER_HORIZONTAL_SPEED };

        if up && !down {
            new_player_y -= move_speed;
        } else if down && !up {
            new_player_y += move_speed;
        }

        if left && !right {
            new_player_x -= horizontal_move_speed;
        } else if right && !left {
            new_player_x += horizontal_move_speed;
        }

        if *key_states.get(&KeyCode::Char('w')).unwrap_or(&false) {
            if context.debug_mode && !context.is_drawing_select_box {
                let current_map_key = (*context.current_map_row, *context.current_map_col);
                if let Some(map_to_modify) = context.loaded_maps.get_mut(&current_map_key) {
                    map_to_modify.toggle_wall(self.x as u32, self.y as u32);
                    if let Err(e) = map_to_modify.save_data() {
                        *context.message = format!("Failed to save map data: {}", e);
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

        let (_, player_sprite_width, player_sprite_height) = self.get_sprite_content();

        let collision_box_x = (new_player_x as u16)
            .saturating_add(player_sprite_width / 2)
            .saturating_sub(PLAYER_COLLISION_WIDTH / 2);
        let collision_box_y = (new_player_y as u16)
            .saturating_add(player_sprite_height)
            .saturating_sub(PLAYER_COLLISION_HEIGHT);

        if !context.debug_mode {
            let mut collision = false;
            for y_offset in 0..PLAYER_COLLISION_HEIGHT {
                let check_y = collision_box_y.saturating_add(y_offset);
                for x_offset in 0..PLAYER_COLLISION_WIDTH {
                    let check_x = collision_box_x.saturating_add(x_offset);

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
                self.x = original_player_x;
                self.y = original_player_y;
            } else {
                self.x = new_player_x.min((current_map.width.saturating_sub(player_sprite_width)) as f32);
                self.y = new_player_y.min((current_map.height.saturating_sub(player_sprite_height)) as f32);
            }
        } else {
            self.x = new_player_x;
            self.y = new_player_y;
        }

        if self.x != original_player_x || self.y != original_player_y {
            if !self.is_walking {
                self.animation_frame = 1;
                self.animation_timer = Instant::now();
            }
            self.is_walking = true;
            self.walking_stop_timer = Instant::now();
        } else {
            if self.is_walking && self.walking_stop_timer.elapsed() >= self.walking_stop_delay {
                self.is_walking = false;
            }
        }

        self.update_animation(animation_frame_duration);
    }
}
