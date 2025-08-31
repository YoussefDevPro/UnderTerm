use super::config::{
    ANIMATION_FRAME_DURATION, DEBUG_MOVEMENT_FRAME_INTERVAL, PLAYER_HORIZONTAL_SPEED,
    PLAYER_INTERACTION_BOX_HEIGHT, PLAYER_INTERACTION_BOX_WIDTH, PLAYER_SPEED,
};
use super::map::Map;
use ansi_to_tui::IntoText;
use crossterm::event::KeyCode;
use ratatui::text::Text;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const PLAYER_COLLISION_WIDTH: u16 = 21;
const PLAYER_COLLISION_HEIGHT: u16 = 5;

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
    pub movement_counter: u8,
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
    pub is_placing_sprite: bool,
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
            movement_counter: 0,
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

    fn check_collision(&self, player_x: f32, player_y: f32, context: &PlayerUpdateContext) -> bool {
        let (_, player_sprite_width, player_sprite_height) = self.get_sprite_content();
        let collision_box_x = (player_x as u16)
            .saturating_add(player_sprite_width / 2)
            .saturating_sub(PLAYER_COLLISION_WIDTH / 2);
        let collision_box_y = (player_y as u16)
            .saturating_add(player_sprite_height)
            .saturating_sub(PLAYER_COLLISION_HEIGHT);

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
                        return true; // collision
                    }
                }
            }
        }
        false // no collision
    }

    pub fn update(
        &mut self,
        context: &mut PlayerUpdateContext,
        key_states: &HashMap<KeyCode, bool>,
        delta_time: Duration,
    ) {
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

        let mut new_player_x = self.x;
        let mut new_player_y = self.y;

        if context.debug_mode {
            self.movement_counter = self.movement_counter.wrapping_add(1);
            if self.movement_counter % DEBUG_MOVEMENT_FRAME_INTERVAL == 0 {
                if up && !down {
                    new_player_y -= 1.0;
                } else if down && !up {
                    new_player_y += 1.0;
                } else if left && !right {
                    new_player_x -= 1.0;
                } else if right && !left {
                    new_player_x += 1.0;
                }
            }
        } else {
            let y_mov = if up && !down {
                -PLAYER_SPEED
            } else if down && !up {
                PLAYER_SPEED
            } else {
                0.0
            };

            let x_mov = if left && !right {
                -PLAYER_HORIZONTAL_SPEED
            } else if right && !left {
                PLAYER_HORIZONTAL_SPEED
            } else {
                0.0
            };

            let (final_x_mov, final_y_mov) = if x_mov != 0.0 && y_mov != 0.0 {
                (
                    x_mov * (1.0 / (2.0 as f32).sqrt()),
                    y_mov * (1.0 / (2.0 as f32).sqrt()),
                )
            } else {
                (x_mov, y_mov)
            };

            new_player_x += final_x_mov * delta_time.as_secs_f32();
            new_player_y += final_y_mov * delta_time.as_secs_f32();
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

        if !context.debug_mode {
            if self.check_collision(new_player_x, new_player_y, context) {
                self.x = original_player_x;
                self.y = original_player_y;
            } else {
                self.x = new_player_x;
                self.y = new_player_y;
            }
        } else {
            self.x = new_player_x;
            self.y = new_player_y;
        }

        let current_map_key = (*context.current_map_row, *context.current_map_col);
        if let Some(current_map) = context.loaded_maps.get(&current_map_key) {
            let (_, player_sprite_width, player_sprite_height) = self.get_sprite_content();
            self.x = self
                .x
                .max(0.0)
                .min((current_map.width.saturating_sub(player_sprite_width)) as f32);
            self.y = self
                .y
                .max(0.0)
                .min((current_map.height.saturating_sub(player_sprite_height)) as f32);
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

        self.update_animation(ANIMATION_FRAME_DURATION);
    }
}