use crate::game::state::{GameState, TeleportCreationState};
use ansi_to_tui::IntoText;
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Instant;

pub fn handle_debug_input(key: KeyEvent, game_state: &mut GameState) -> bool {
    if !game_state.debug_mode {
        return false;
    }

    match key.code {
        KeyCode::Char('r') => {
            game_state.undo_wall_change();
            true
        }
        KeyCode::Char('z') => {
            game_state.redo_wall_change();
            true
        }
        KeyCode::Char('s') => {
            game_state
                .set_player_spawn_to_current_position(game_state.player.x, game_state.player.y);
            true
        }
        KeyCode::Char('k') => {
            game_state.is_map_kind_selection_active = !game_state.is_map_kind_selection_active;
            if game_state.is_map_kind_selection_active {
                game_state.message =
                    "Map Kind Selection: Use Up/Down to cycle, Enter to confirm, Esc to cancel."
                        .to_string();
            } else {
                game_state.message = "Map Kind Selection cancelled.".to_string();
            }
            game_state.show_message = true;
            game_state.message_animation_start_time = Instant::now();
            game_state.animated_message_content.clear();
            true
        }
        KeyCode::Char('o') => {
            if !game_state.is_drawing_select_box {
                game_state.is_drawing_select_box = true;
                game_state.select_box_start_coords =
                    Some((game_state.player.x as u16, game_state.player.y as u16));
                game_state.message =
                    "Drawing select box: Move player to set end point, then press Enter."
                        .to_string();
                game_state.block_player_movement_on_message = false;
            } else {
                game_state.message = "Finish drawing select box by pressing Enter.".to_string();
            }
            game_state.show_message = true;
            game_state.message_animation_start_time = Instant::now();
            game_state.animated_message_content.clear();
            true
        }

        KeyCode::F(3) => {
            game_state.is_creating_map = true;
            game_state.is_text_input_active = true;
            game_state.text_input_buffer.clear();
            game_state.message = "Enter new map name (e.g., map_0_1):".to_string();
            game_state.show_message = true;
            game_state.message_animation_start_time = Instant::now();
            game_state.animated_message_content.clear();
            true
        }
        KeyCode::Char('t') => {
            if game_state.teleport_creation_state == TeleportCreationState::None {
                game_state.teleport_creation_state = TeleportCreationState::DrawingBox;
                game_state.select_box_start_coords =
                    Some((game_state.player.x as u16, game_state.player.y as u16));
                game_state.message =
                    "Drawing teleport line: Move player to set end point, then press Enter."
                        .to_string();
                game_state.block_player_movement_on_message = false;
            } else if game_state.teleport_creation_state == TeleportCreationState::DrawingBox {
                game_state.message = "Finish drawing teleport line by pressing Enter.".to_string();
            }
            game_state.show_message = true;
            game_state.message_animation_start_time = Instant::now();
            game_state.animated_message_content.clear();
            true
        }
        KeyCode::Char('x') => {
            if !game_state.is_placing_sprite {
                game_state.is_placing_sprite = true;
                                let sprite_content = std::fs::read_to_string(
                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sprites/ME/idle/default.ans"),
                ).unwrap_or_else(|_| "X".to_string());
                let text = sprite_content.as_bytes().into_text().unwrap();
                let height = text.lines.len() as u32;
                let mut width = 0;
                for line in text.lines.iter() {
                    let line_width = line.width() as u32;
                    if line_width > width {
                        width = line_width;
                    }
                }

                let new_placed_sprite = crate::game::map::PlacedSprite {
                    id: 0,
                    x: game_state.player.x as u32,
                    y: game_state.player.y as u32,
                    width,
                    height,
                    ansi_content: sprite_content,
                };
                game_state.pending_placed_sprite = Some(new_placed_sprite);
                game_state.message =
                    "Placing sprite: Move player to position, press Enter to place.".to_string();
                game_state.block_player_movement_on_message = false;
            } else {
                game_state.message = "Finish placing sprite by pressing Enter.".to_string();
            }
            game_state.show_message = true;
            game_state.message_animation_start_time = Instant::now();
            game_state.animated_message_content.clear();
            true
        }
        KeyCode::Enter => {
            if game_state.is_drawing_select_box {
                if game_state.is_confirming_select_box {
                    game_state.is_text_input_active = true;
                    game_state.text_input_buffer.clear();
                    game_state.message =
                        "Enter message. Press Enter to add, or Esc to finish.".to_string();
                    game_state.show_message = true;
                    game_state.message_animation_start_time = Instant::now();
                    game_state.animated_message_content.clear();
                    game_state.is_confirming_select_box = false;
                } else {
                    if let Some((start_x, start_y)) = game_state.select_box_start_coords {
                        let (end_x, end_y) =
                            (game_state.player.x as u16, game_state.player.y as u16);
                        let current_map_key =
                            (game_state.current_map_row, game_state.current_map_col);
                        if let Some(map_to_modify) =
                            game_state.loaded_maps.get_mut(&current_map_key)
                        {
                            let new_id = map_to_modify
                                .select_object_boxes
                                .iter()
                                .map(|b| b.id)
                                .max()
                                .unwrap_or(0)
                                + 1;
                            let new_select_box = crate::game::map::SelectObjectBox {
                                id: new_id,
                                x: start_x.min(end_x) as u32,
                                y: start_y.min(end_y) as u32,
                                width: start_x
                                    .max(end_x)
                                    .saturating_sub(start_x.min(end_x))
                                    .saturating_add(1)
                                    as u32,
                                height: start_y
                                    .max(end_y)
                                    .saturating_sub(start_y.min(end_y))
                                    .saturating_add(1)
                                    as u32,
                                messages: Vec::new(),
                                events: Vec::new(),
                            };
                            game_state.pending_select_box = Some(new_select_box);
                            game_state.message = "Select box confirmed. Press Enter again to add messages, or Esc to cancel.".to_string();
                            game_state.show_message = true;
                            game_state.message_animation_start_time = Instant::now();
                            game_state.animated_message_content.clear();
                        }
                    }
                    game_state.select_box_start_coords = None;
                    game_state.is_confirming_select_box = true;
                }
                true
            } else if game_state.teleport_creation_state == TeleportCreationState::DrawingBox {
                if let Some((start_x, start_y)) = game_state.select_box_start_coords {
                    let (end_x, end_y) = (game_state.player.x as u16, game_state.player.y as u16);

                    let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                    if let Some(map_to_modify) = game_state.loaded_maps.get_mut(&current_map_key) {
                        let new_id = map_to_modify
                            .select_object_boxes
                            .iter()
                            .map(|b| b.id)
                            .max()
                            .unwrap_or(0)
                            + 1;
                        let new_teleport_box = crate::game::map::SelectObjectBox {
                            id: new_id,
                            x: start_x.min(end_x) as u32,
                            y: start_y.min(end_y) as u32,
                            width: start_x
                                .max(end_x)
                                .saturating_sub(start_x.min(end_x))
                                .saturating_add(1) as u32,
                            height: start_y
                                .max(end_y)
                                .saturating_sub(start_y.min(end_y))
                                .saturating_add(1) as u32,
                            messages: Vec::new(),
                            events: Vec::new(),
                        };
                        map_to_modify.add_select_object_box(new_teleport_box.clone());
                        if let Err(e) = map_to_modify.save_data() {
                            game_state.message = format!("Failed to save map data: {}", e);
                        } else {
                            game_state.message = "Teleport box created and saved. Enter target map name (e.g., map_0_0):".to_string();
                        }
                        game_state.pending_select_box = Some(new_teleport_box);
                        game_state.teleport_creation_state = TeleportCreationState::EnteringMapName;
                        game_state.is_text_input_active = true;
                        game_state.teleport_destination_map_name_buffer.clear();
                        game_state.show_message = true;
                        game_state.message_animation_start_time = Instant::now();
                        game_state.animated_message_content.clear();
                    }
                }
                true
            } else if game_state.is_placing_sprite {
                if let Some(mut placed_sprite) = game_state.pending_placed_sprite.take() {
                    let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                    if let Some(map_to_modify) = game_state.loaded_maps.get_mut(&current_map_key) {
                        placed_sprite.x = game_state.player.x as u32;
                        placed_sprite.y = game_state.player.y as u32;
                        let new_id = map_to_modify
                            .placed_sprites
                            .iter()
                            .map(|s| s.id)
                            .max()
                            .unwrap_or(0)
                            + 1;
                        placed_sprite.id = new_id;
                        map_to_modify.add_placed_sprite(placed_sprite);
                        if let Err(e) = map_to_modify.save_data() {
                            game_state.message = format!("Failed to save map data: {}", e);
                        } else {
                            game_state.message = "Sprite placed and saved.".to_string();
                        }
                    } else {
                        game_state.message =
                            "Error: Current map not found for saving sprite.".to_string();
                    }
                } else {
                    game_state.message = "Error: No pending sprite to place.".to_string();
                }
                game_state.is_placing_sprite = false;
                game_state.block_player_movement_on_message = true;
                game_state.show_message = true;
                game_state.message_animation_start_time = Instant::now();
                game_state.animated_message_content.clear();
                true
            } else {
                false
            }
        }
        _ => false,
    }
}
