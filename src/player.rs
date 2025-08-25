use ansi_to_tui::IntoText;

use ratatui::{ text::Text, };
use serde::{Deserialize, Serialize};

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
}

impl Player {
    pub fn new(x: u16, y: u16) -> Self {
        Player {
            x,
            y,
            direction: PlayerDirection::Front, // Default direction
            animation_frame: 0,
            is_walking: false,
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
                        0 => ("walk", "frisk_walk_front_1.ans"),
                        1 => ("walk", "frisk_walk_front_2.ans"),
                        _ => ("idle", "frisk_idle_front.ans"), // Fallback
                    }
                } else {
                    ("idle", "frisk_idle_front.ans")
                }
            },
            PlayerDirection::Back => {
                if self.is_walking {
                    match self.animation_frame {
                        0 => ("walk", "frisk_walk_back_1.ans"),
                        1 => ("walk", "frisk_walk_back_2.ans"),
                        _ => ("idle", "frisk_idle_back.ans"), // Fallback
                    }
                } else {
                    ("idle", "frisk_idle_back.ans")
                }
            },
            PlayerDirection::Left => {
                if self.is_walking {
                    ("walk", "frisk_walk_left.ans")
                } else {
                    ("idle", "frisk_idle_left.ans")
                }
            },
            PlayerDirection::Right => {
                if self.is_walking {
                    ("walk", "frisk_walk_right.ans")
                } else {
                    ("idle", "frisk_idle_right.ans")
                }
            },
        };
        format!("{}{}/{}", base_path, sub_dir, anim_file)
    }

    pub fn update_animation(&mut self) {
        if self.is_walking {
            match self.direction {
                PlayerDirection::Front | PlayerDirection::Back => {
                    self.animation_frame = (self.animation_frame + 1) % 3; // 0, 1, 2
                }
                PlayerDirection::Left | PlayerDirection::Right => {
                    self.animation_frame = (self.animation_frame + 1) % 2; // 0, 1
                }
            }
        } else {
            self.animation_frame = 0; // seret
        }
    }
}
