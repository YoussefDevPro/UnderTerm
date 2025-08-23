use crate::player::{Player, PlayerDirection};
use crate::utils::text_to_ansi_string;
use crate::PLAYER_SPEED;
use ansi_to_tui::IntoText;
use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Style},
    text::Span,
};
use serde::{Deserialize, Serialize};
use std::io::{self};

use unicode_width::UnicodeWidthStr;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub map_raw_data: String, // Store map as a Text object
    pub camera_x: u16,
    pub camera_y: u16,
    pub map_width: u16,
    pub map_height: u16,
    pub message: String,
    pub show_message: bool,
    pub spawn_x: u16,
    pub spawn_y: u16,
    pub auto_draw_enabled: bool,
    pub previous_player_x: u16,
    pub previous_player_y: u16,
    pub wall_grid: Vec<Vec<bool>>,
}

impl GameState {
    pub fn new() -> io::Result<Self> {
        let map_data_raw = std::fs::read_to_string("assets/sprites/map/map_1/sprite.ans")?;
        let map_text_for_dimensions = map_data_raw.as_bytes().into_text().unwrap();

        let map_height = {
            let mut actual_height = 0;
            for (i, line) in map_text_for_dimensions.lines.iter().enumerate() {
                if !line.spans.iter().all(|span| span.content.trim().is_empty()) {
                    actual_height = i + 1;
                }
            }
            actual_height as u16
        };

        let mut max_width = 0;
        for line in map_text_for_dimensions.lines.iter() {
            let line_width = line.width() as u16;
            if line_width > max_width {
                max_width = line_width;
            }
        }
        let map_width = max_width;

        let wall_grid = vec![vec![false; map_width as usize]; map_height as usize];

        Ok(GameState {
            player: Player::new(map_width / 2, map_height / 2),
            map_raw_data: map_data_raw,
            camera_x: 0,
            camera_y: 0,
            map_width,
            map_height,
            message: String::new(),
            show_message: false,
            spawn_x: map_width / 2,
            spawn_y: map_height / 2,
            auto_draw_enabled: false,
            previous_player_x: map_width / 2,
            previous_player_y: map_height / 2,
            wall_grid,
        })
    }

    pub fn reset_map(&mut self) -> io::Result<()> {
        let map_data_raw = std::fs::read_to_string("assets/sprites/map/map_1/sprite.ans")?;
        self.map_raw_data = map_data_raw;
        let updated_map_content = self.map_raw_data.clone();
        std::fs::write("assets/sprites/map/map_1/sprite.ans", updated_map_content)?;
        Ok(())
    }

    pub fn save_game_state(&self) -> io::Result<()> {
        let serialized = serde_json::to_string(&self)?;
        std::fs::write("game_data.json", serialized)?;
        Ok(())
    }

    pub fn load_game_state() -> io::Result<Self> {
        let default_state = GameState::new()?;
        match std::fs::read_to_string("game_data.json") {
            Ok(data) => {
                let deserialized: GameState = serde_json::from_str(&data)?;
                Ok(deserialized)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                default_state.save_game_state()?;
                Ok(default_state)
            }
            Err(e) => Err(e),
        }
    }

    pub fn update(&mut self, key_code: Option<KeyCode>, frame_size: ratatui::layout::Rect) {
        self.player.is_walking = false;
        self.show_message = false;

        if let Some(key) = key_code {
            let mut new_x = self.player.x;
            let mut new_y = self.player.y;

            match key {
                KeyCode::Up => {
                    new_y = new_y.saturating_sub(PLAYER_SPEED);
                    self.player.direction = PlayerDirection::Back;
                    self.player.is_walking = true;
                }
                KeyCode::Down => {
                    new_y = new_y.saturating_add(PLAYER_SPEED);
                    self.player.direction = PlayerDirection::Front;
                    self.player.is_walking = true;
                }
                KeyCode::Left => {
                    new_x = new_x.saturating_sub(PLAYER_SPEED);
                    self.player.direction = PlayerDirection::Left;
                    self.player.is_walking = true;
                }
                KeyCode::Right => {
                    new_x = new_x.saturating_add(PLAYER_SPEED);
                    self.player.direction = PlayerDirection::Right;
                    self.player.is_walking = true;
                }
                KeyCode::Enter => {
                    self.auto_draw_enabled = !self.auto_draw_enabled;
                    if self.auto_draw_enabled {
                        self.message = "Auto-draw enabled!".to_string();
                    } else {
                        self.message = "Auto-draw disabled!".to_string();
                    }
                    self.show_message = true;
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    if let Err(e) = self.reset_map() {
                        eprintln!("Failed to reset map: {:?}", e);
                    }
                    self.message = "Map reset!".to_string();
                    self.show_message = true;
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.spawn_x = self.player.x;
                    self.spawn_y = self.player.y;
                    self.message =
                        format!("Spawn point set to ({}, {})", self.spawn_x, self.spawn_y);
                    self.show_message = true;
                }
                _ => {}
            }

            // Collision detection
            let (_, player_sprite_width, player_sprite_height) = self.player.get_sprite_content();
            let collision_box_height = 3;
            let collision_box_start_y = new_y.saturating_add(player_sprite_height).saturating_sub(collision_box_height);

            let mut collision = false;
            for y_offset in 0..collision_box_height {
                let check_y = collision_box_start_y.saturating_add(y_offset);
                for x_offset in 0..player_sprite_width {
                    let check_x = new_x.saturating_add(x_offset);

                    let map_text = self.map_raw_data.as_bytes().into_text().unwrap();
                    if let Some(line) = map_text.lines.get(check_y as usize) {
                        let mut current_char_idx = 0;
                        for span in line.spans.iter() {
                            let span_len_chars = span.content.width() as u16;
                            if check_x >= current_char_idx && check_x < current_char_idx + span_len_chars {
                                if let Some(char) = span.content.chars().nth((check_x - current_char_idx) as usize) {
                                    if char == '█' {
                                        collision = true;
                                        break;
                                    }
                                }
                            }
                            current_char_idx += span_len_chars;
                        }
                    }
                    if collision { break; }
                }
                if collision { break; }
            }

            if collision {
                self.message = "You hit a wall!".to_string();
                self.show_message = true;
                return; // Exit update early, or just don't update x/y
            }

            self.player.x = new_x.min(self.map_width - 1); // Ensure player stays within map bounds
            self.player.y = new_y.min(self.map_height - 1); // Ensure player stays within map bounds

            // Auto-draw logic
            if self.auto_draw_enabled
                && (self.player.x != self.previous_player_x
                    || self.player.y != self.previous_player_y)
            {
                let player_x = self.previous_player_x as usize;
                let player_y = self.previous_player_y as usize;

                // Convert raw ANSI map data to Text for modification
                let mut map_text_editable = self.map_raw_data.as_bytes().into_text().unwrap();

                if let Some(line) = map_text_editable.lines.get_mut(player_y) {
                    let mut new_spans: Vec<Span> = Vec::new();
                    let mut current_char_idx = 0;

                    for span in line.spans.drain(..) {
                        let span_len_chars = span.content.width() as usize;

                        // Case 1: Wall is before this span
                        if player_x < current_char_idx {
                            new_spans.push(span);
                        }
                        // Case 2: Wall is within this span
                        else if player_x >= current_char_idx
                            && player_x < current_char_idx + span_len_chars
                        {
                            // Split the span into three parts: before wall, wall, after wall
                            let wall_char = '█';
                            let wall_style = Style::default().fg(Color::Rgb(255, 0, 255));

                            let char_offset_in_span = player_x - current_char_idx;

                            // Part before the wall
                            if char_offset_in_span > 0 {
                                let before_wall_content: String =
                                    span.content.chars().take(char_offset_in_span).collect();
                                new_spans.push(Span::styled(before_wall_content, span.style));
                            }

                            // The wall itself
                            new_spans.push(Span::styled(wall_char.to_string(), wall_style));

                            // Part after the wall
                            if char_offset_in_span + 1 < span_len_chars {
                                let after_wall_content: String =
                                    span.content.chars().skip(char_offset_in_span + 1).collect();
                                new_spans.push(Span::styled(after_wall_content, span.style));
                            }
                        }
                        // Case 3: Wall is after this span
                        else {
                            new_spans.push(span);
                        }
                        current_char_idx += span_len_chars;
                    }
                    line.spans = new_spans; // Replace with new spans
                }

                // Convert modified Text back to ANSI string
                self.map_raw_data = text_to_ansi_string(&map_text_editable);

                // Save the updated map to file
                if let Err(e) =
                    std::fs::write("assets/sprites/map/map_1/sprite.ans", &self.map_raw_data)
                {
                    eprintln!("Failed to save map: {:?}", e);
                }
            }

            // Update previous player position for next frame
            self.previous_player_x = self.player.x;
            self.previous_player_y = self.player.y;
        }

        self.player.update_animation();

        // Update camera to follow player
        let mut new_camera_x = self.player.x.saturating_sub(frame_size.width / 2);
        let mut new_camera_y = self.player.y.saturating_sub(frame_size.height / 2);

        // Clamp camera position to map boundaries
        new_camera_x = new_camera_x.min(self.map_width.saturating_sub(frame_size.width));
        new_camera_y = new_camera_y.min(self.map_height.saturating_sub(frame_size.height));

        // Ensure camera coordinates are not negative
        self.camera_x = new_camera_x.max(0);
        self.camera_y = new_camera_y.max(0);
    }
}
