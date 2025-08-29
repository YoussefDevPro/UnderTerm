use crate::debug;
use std::{
    collections::HashMap,
    io,
    sync::mpsc,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};

use crate::game::config::ESC_HOLD_DURATION;
use crate::game::state::{GameState, TeleportCreationState};

pub fn input_handler(tx: mpsc::Sender<Event>) -> io::Result<()> {
    loop {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if tx.send(Event::Key(key)).is_err() {
                    break;
                }
            }
        }
    }
    Ok(())
}

pub fn process_events(
    rx: &mpsc::Receiver<Event>,
    game_state: &mut GameState,
    key_states: &mut HashMap<KeyCode, bool>,
    audio: &crate::audio::Audio,
) -> io::Result<bool> {
    // Check for Esc hold to quit
    if let Some(start_time) = game_state.esc_press_start_time {
        if start_time.elapsed() >= ESC_HOLD_DURATION {
            return Ok(true); // Quit the game
        }
    }

    while let Ok(event) = rx.try_recv() {
        if let Event::Key(key) = event {
            if game_state.is_text_input_active {
                match key.code {
                    KeyCode::Char(c) => {
                        game_state.text_input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        game_state.text_input_buffer.pop();
                    }
                    KeyCode::Enter => {
                        if game_state.is_creating_map {
                            let map_name = game_state.text_input_buffer.trim().to_string();
                            if !map_name.is_empty() {
                                if let Ok(new_map) = crate::game::map::Map::create_new(&map_name) {
                                    let map_parts: Vec<&str> = new_map.name.split('_').collect();
                                    let new_map_row: i32 = map_parts[1].parse().unwrap_or(0);
                                    let new_map_col: i32 = map_parts[2].parse().unwrap_or(0);
                                    game_state
                                        .loaded_maps
                                        .insert((new_map_row, new_map_col), new_map);
                                    game_state.message = format!("Created new map: {}", map_name);
                                } else {
                                    game_state.message =
                                        format!("Error creating map: {}", map_name);
                                }
                            }
                            game_state.is_creating_map = false;
                            game_state.is_text_input_active = false; // Text input finished here
                            game_state.text_input_buffer.clear();
                            game_state.show_message = true;
                            game_state.message_animation_start_time = Instant::now();
                            game_state.animated_message_content.clear();
                        } else if game_state.teleport_creation_state
                            == TeleportCreationState::EnteringMapName
                        {
                            // This is for map name input
                            let map_name = game_state.text_input_buffer.trim().to_string();
                            if !map_name.is_empty() {
                                // Try to load the map to validate the name
                                if let Ok(loaded_map) = crate::game::map::Map::load(&map_name) {
                                    let map_parts: Vec<&str> = loaded_map.name.split('_').collect();
                                    let _map_row: i32 = map_parts[1].parse().unwrap_or(0);
                                    let _map_col: i32 = map_parts[2].parse().unwrap_or(0);

                                    game_state.teleport_destination_map_name_buffer = map_name;
                                    

                                    game_state.is_text_input_active = false; // Text input finished for map name
                                    game_state.teleport_creation_state =
                                        TeleportCreationState::SelectingCoordinates; // Transition to visual selection

                                    game_state.message = format!(
                                        "Map '{}' loaded. Move player to select X, Y. Press Enter to confirm.",
                                        loaded_map.name
                                    );
                                    game_state.show_message = true;
                                    game_state.message_animation_start_time = Instant::now();
                                    game_state.animated_message_content.clear();
                                } else {
                                    game_state.message = format!(
                                        "Failed to load map: {}. Please enter a valid map name.",
                                        map_name
                                    );
                                    game_state.show_message = true;
                                    game_state.message_animation_start_time = Instant::now();
                                    game_state.animated_message_content.clear();
                                }
                            } else {
                                game_state.message = "Map name cannot be empty.".to_string();
                                game_state.show_message = true;
                                game_state.message_animation_start_time = Instant::now();
                                game_state.animated_message_content.clear();
                            }
                        } else if let Some(ref mut pending_box) = game_state.pending_select_box {
                            pending_box
                                .messages
                                .push(game_state.text_input_buffer.clone());
                            game_state.text_input_buffer.clear();
                            game_state.message = format!(
                                "Message added. Enter next or Esc to finish. Current: {}",
                                pending_box.messages.len()
                            );
                            game_state.show_message = true;
                            game_state.message_animation_start_time = Instant::now();
                            game_state.animated_message_content.clear();
                            game_state.is_text_input_active = false; // Text input finished here
                        }
                    }
                    KeyCode::Esc => {
                        game_state.esc_press_start_time = Some(Instant::now());
                        game_state.is_text_input_active = false;
                        game_state.text_input_buffer.clear();
                        if game_state.is_creating_map {
                            game_state.is_creating_map = false;
                            game_state.message = "Map creation cancelled.".to_string();
                        } else if game_state.teleport_creation_state != TeleportCreationState::None
                        {
                            game_state.teleport_creation_state = TeleportCreationState::None;
                            game_state.pending_select_box = None; // Clear pending teleport box
                            game_state.message = "Teleport line creation cancelled.".to_string();
                            game_state.block_player_movement_on_message = true; // Reset movement blocking
                        } else {
                            game_state.is_event_input_active = true;
                            game_state.message = "Enter events. Format: 'teleport x y map_row map_col'. Esc to finish.".to_string();
                        }
                        game_state.show_message = true;
                        game_state.message_animation_start_time = Instant::now();
                        game_state.animated_message_content.clear();
                    }
                    _ => {}
                }
                continue;
            }

            if game_state.is_event_input_active {
                match key.code {
                    KeyCode::Char(c) => game_state.text_input_buffer.push(c),
                    KeyCode::Backspace => {
                        game_state.text_input_buffer.pop();
                    }
                    KeyCode::Enter => {
                        if let Some(ref mut pending_box) = game_state.pending_select_box {
                            let input = game_state.text_input_buffer.trim();
                            let parts: Vec<&str> = input.split_whitespace().collect();
                            if parts.len() >= 5 && parts[0] == "teleport" {
                                if let (Ok(x), Ok(y), Ok(map_row), Ok(map_col)) = (
                                    parts[1].parse(),
                                    parts[2].parse(),
                                    parts[3].parse(),
                                    parts[4].parse(),
                                ) {
                                    pending_box.events.push(
                                        crate::game::map::Event::TeleportPlayer {
                                            x,
                                            y,
                                            map_row,
                                            map_col,
                                        },
                                    );
                                    game_state.message = format!(
                                        "Teleport event added. Current: {}",
                                        pending_box.events.len()
                                    );
                                } else {
                                    game_state.message = "Invalid teleport parameters.".to_string();
                                }
                            } else {
                                game_state.message = "Unknown event format.".to_string();
                            }
                            game_state.text_input_buffer.clear();
                            game_state.show_message = true;
                            game_state.message_animation_start_time = Instant::now();
                            game_state.animated_message_content.clear();
                        }
                    }
                    KeyCode::Esc => {
                        if let Some(pending_box) = game_state.pending_select_box.take() {
                            let current_map_key =
                                (game_state.current_map_row, game_state.current_map_col);
                            if let Some(map_to_modify) =
                                game_state.loaded_maps.get_mut(&current_map_key)
                            {
                                map_to_modify.add_select_object_box(pending_box);
                                if let Err(e) = map_to_modify.save_data() {
                                    game_state.message = format!("Failed to save map data: {}", e);
                                } else {
                                    game_state.message =
                                        "SelectObjectBox created and saved.".to_string();
                                }
                                game_state.show_message = true;
                                game_state.message_animation_start_time = Instant::now();
                                game_state.animated_message_content.clear();
                            }
                        }
                        game_state.is_event_input_active = false;
                        game_state.text_input_buffer.clear();
                        game_state.is_drawing_select_box = false; // Reset drawing flag
                        game_state.block_player_movement_on_message = true; // Block movement again
                    }
                    _ => {}
                }
                continue;
            }

            match key.kind {
                event::KeyEventKind::Press => {
                    key_states.insert(key.code, true);
                    if key.code == KeyCode::Esc {
                        game_state.esc_press_start_time = Some(Instant::now());
                    }
                    if game_state.is_map_kind_selection_active {
                        let current_map_key =
                            (game_state.current_map_row, game_state.current_map_col);
                        if let Some(map_to_modify) =
                            game_state.loaded_maps.get_mut(&current_map_key)
                        {
                            match key.code {
                                KeyCode::Up => map_to_modify.kind = map_to_modify.kind.previous(),
                                KeyCode::Down => map_to_modify.kind = map_to_modify.kind.next(),
                                KeyCode::Enter => {
                                    if let Err(e) = map_to_modify.save_data() {
                                        game_state.message = format!("Failed to save map: {}", e);
                                    } else {
                                        game_state.message =
                                            format!("Map kind set to {:?}", map_to_modify.kind);
                                    }
                                    game_state.show_message = true;
                                    game_state.message_animation_start_time = Instant::now();
                                    game_state.animated_message_content.clear();
                                    game_state.is_map_kind_selection_active = false;
                                }
                                KeyCode::Esc => game_state.is_map_kind_selection_active = false,
                                _ => {}
                            }
                        }
                    } else if debug::input::handle_debug_input(key, game_state) {
                        // Handled by debug module
                    } else if key.code == KeyCode::Enter {
                        if game_state.show_message {
                            if game_state.message_animation_finished {
                                if let Some(box_id) = game_state.current_interaction_box_id {
                                    let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                                    if let Some(current_map) = game_state.loaded_maps.get(&current_map_key) {
                                        if let Some(interacting_box) = current_map.select_object_boxes.iter().find(|b| b.id == box_id) {
                                            if game_state.current_message_index < interacting_box.messages.len() {
                                                game_state.message = interacting_box.messages[game_state.current_message_index].clone();
                                                game_state.show_message = true;
                                                game_state.message_animation_start_time = Instant::now();
                                                game_state.animated_message_content.clear();
                                                game_state.message_animation_finished = false;
                                                game_state.current_message_index += 1;
                                            } else {
                                                let events = interacting_box.events.clone();
                                                let box_id = interacting_box.id;
                                                game_state.dismiss_message();
                                                game_state.current_interaction_box_id = None;
                                                game_state.current_message_index = 0;
                                                let _events_to_process: Vec<crate::game::map::Event> = events;
                                                game_state.recently_teleported_from_box_id = Some(box_id);
                                            }
                                        }
                                    }
                                } else {
                                    game_state.dismiss_message();
                                }
                            } else {
                                game_state.skip_message_animation();
                            }
                        } else {
                            // No message is shown, check for new interaction
                            let (_, _, player_sprite_height) = game_state.player.get_sprite_content();
                            let collision_box_x = game_state.player.x;
                            let collision_box_y = game_state.player.y.saturating_add(player_sprite_height).saturating_sub(4);
                            let collision_box = ratatui::layout::Rect::new(collision_box_x, collision_box_y, 21, 4);
                            let interaction_boxes = [
                                ratatui::layout::Rect::new(collision_box.x, collision_box.y.saturating_sub(10), collision_box.width, 10),
                                ratatui::layout::Rect::new(collision_box.x, collision_box.y + collision_box.height, collision_box.width, 3),
                                ratatui::layout::Rect::new(collision_box.x.saturating_sub(5), collision_box.y, 5, collision_box.height),
                                ratatui::layout::Rect::new(collision_box.x + collision_box.width, collision_box.y, 5, collision_box.height),
                            ];
                            let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                            if let Some(current_map) = game_state.loaded_maps.get(&current_map_key) {
                                if let Some(found_box) = current_map.select_object_boxes.iter().find(|b| {
                                    let select_box_rect = b.to_rect();
                                    interaction_boxes.iter().any(|interaction_box| interaction_box.intersects(select_box_rect))
                                }) {
                                    if !found_box.messages.is_empty() {
                                        if game_state.recently_teleported_from_box_id != Some(found_box.id) {
                                            game_state.current_interaction_box_id = Some(found_box.id);
                                            game_state.current_message_index = 0;
                                            game_state.message = found_box.messages[game_state.current_message_index].clone();
                                            game_state.show_message = true;
                                            game_state.message_animation_start_time = Instant::now();
                                            game_state.animated_message_content.clear();
                                            game_state.message_animation_finished = false;
                                            game_state.current_message_index += 1;
                                        }
                                    }
                                }
                            }
                        }
                    } else if key.code == KeyCode::Char('q') {
                        game_state.save_game_state()?;
                        return Ok(true);
                    } else if key.code == KeyCode::Char('p') {
                        // New save key
                        game_state.save_game_state()?;
                        game_state.message = "Game saved!".to_string();
                        game_state.show_message = true;
                        game_state.message_animation_start_time = Instant::now();
                        game_state.animated_message_content.clear();
                    } else if key.code == KeyCode::F(1) {
                        game_state.show_debug_panel = !game_state.show_debug_panel;
                        if game_state.show_debug_panel {
                            audio.play_open_settings_sound();
                        }
                    } else if key.code == KeyCode::F(2) {
                        game_state.debug_mode = !game_state.debug_mode;
                    }
                }
                event::KeyEventKind::Release => {
                    key_states.insert(key.code, false);
                    if key.code == KeyCode::Esc {
                        game_state.esc_press_start_time = None;
                    }
                    if key.code == KeyCode::Char('o') && game_state.debug_mode {
                        game_state.show_collision_box = false;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(false)
}
