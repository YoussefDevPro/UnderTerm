use crate::debug;
use std::{
    collections::HashMap,
    io,
    sync::mpsc,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};

use crate::game::state::GameState;

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
    while let Ok(event) = rx.try_recv() {
        if let Event::Key(key) = event {
            if game_state.is_text_input_active {
                // Handle text input
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
                                    game_state.loaded_maps.insert((new_map_row, new_map_col), new_map);
                                    game_state.message = format!("Created new map: {}", map_name);
                                } else {
                                    game_state.message = format!("Error creating map: {}", map_name);
                                }
                            }
                            game_state.is_creating_map = false;
                            game_state.is_text_input_active = false;
                            game_state.text_input_buffer.clear();
                            game_state.show_message = true;
                            game_state.message_animation_start_time = Instant::now();
                            game_state.animated_message_content.clear();
                        } else if game_state.is_teleport_input_active {
                            let target_map_id = game_state.text_input_buffer.trim().to_string();
                            if !target_map_id.is_empty() {
                                if let (Some((min_x, min_y)), Some((width, height))) = (
                                    game_state.teleport_zone_start_coords,
                                    game_state.select_box_start_coords,
                                ) {
                                    let teleport_zone = crate::game::state::TeleportZone {
                                        rect: ratatui::layout::Rect::new(min_x, min_y, width, height),
                                        target_map_id: target_map_id.clone(),
                                    };
                                    let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                                    if let Some(map_to_modify) = game_state.loaded_maps.get_mut(&current_map_key) {
                                        map_to_modify.add_teleport_zone(teleport_zone);
                                        if let Err(e) = map_to_modify.save_data() {
                                            game_state.message = format!("Failed to save map data: {}", e);
                                        } else {
                                            game_state.message = format!("Teleport zone created to {}.", target_map_id);
                                        }
                                        game_state.show_message = true;
                                        game_state.message_animation_start_time = Instant::now();
                                        game_state.animated_message_content.clear();
                                    }
                                }
                            }
                            game_state.is_teleport_input_active = false;
                            game_state.is_text_input_active = false;
                            game_state.text_input_buffer.clear();
                            game_state.teleport_zone_start_coords = None;
                            game_state.select_box_start_coords = None;
                            game_state.is_drawing_teleport_zone = false;
                        } else if let Some(ref mut pending_box) = game_state.pending_select_box {
                            pending_box.messages.push(game_state.text_input_buffer.clone());
                            game_state.text_input_buffer.clear();
                            game_state.message = format!("Message added. Enter next or Esc to finish. Current: {}", pending_box.messages.len());
                            game_state.show_message = true;
                            game_state.message_animation_start_time = Instant::now();
                            game_state.animated_message_content.clear();
                        }
                        game_state.is_text_input_active = false;
                    }
                    KeyCode::Esc => {
                        game_state.is_text_input_active = false;
                        game_state.text_input_buffer.clear();
                        if game_state.is_creating_map {
                            game_state.is_creating_map = false;
                            game_state.message = "Map creation cancelled.".to_string();
                        } else if game_state.is_teleport_input_active {
                            game_state.is_teleport_input_active = false;
                            game_state.teleport_zone_start_coords = None;
                            game_state.select_box_start_coords = None;
                            game_state.message = "Teleport zone creation cancelled.".to_string();
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
            } else if game_state.is_event_input_active {
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
                                    parts[1].parse(), parts[2].parse(), parts[3].parse(), parts[4].parse(),
                                ) {
                                    pending_box.events.push(crate::game::map::Event::TeleportPlayer { x, y, map_row, map_col });
                                    game_state.message = format!("Teleport event added. Current: {}", pending_box.events.len());
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
                            let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                            if let Some(map_to_modify) = game_state.loaded_maps.get_mut(&current_map_key) {
                                map_to_modify.add_select_object_box(pending_box);
                                if let Err(e) = map_to_modify.save_data() {
                                    game_state.message = format!("Failed to save map data: {}", e);
                                } else {
                                    game_state.message = "SelectObjectBox created and saved.".to_string();
                                }
                                game_state.show_message = true;
                                game_state.message_animation_start_time = Instant::now();
                                game_state.animated_message_content.clear();
                            }
                        }
                        game_state.is_event_input_active = false;
                        game_state.text_input_buffer.clear();
                    }
                    _ => {}
                }
            } else if game_state.show_message {
                if key.code == KeyCode::Enter || key.code == KeyCode::Char('o') {
                    if game_state.message_animation_finished {
                        let mut teleport_target = None;
                        if let Some(box_id) = game_state.current_interaction_box_id {
                            let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                            if let Some(current_map) = game_state.loaded_maps.get(&current_map_key) {
                                if let Some(interacting_box) = current_map.select_object_boxes.iter().find(|b| b.id == box_id) {
                                    if game_state.current_message_index + 1 < interacting_box.messages.len() {
                                        game_state.current_message_index += 1;
                                        game_state.message = interacting_box.messages[game_state.current_message_index].clone();
                                        game_state.show_message = true;
                                        game_state.message_animation_start_time = Instant::now();
                                        game_state.animated_message_content.clear();
                                        game_state.message_animation_finished = false;
                                    } else {
                                        for event in &interacting_box.events {
                                            match event {
                                                crate::game::map::Event::TeleportPlayer { x, y, map_row, map_col } => {
                                                    teleport_target = Some((*x as u16, *y as u16, *map_row, *map_col));
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        game_state.dismiss_message();
                        if let Some((x, y, map_row, map_col)) = teleport_target {
                            game_state.player.x = x;
                            game_state.player.y = y;
                            game_state.current_map_row = map_row;
                            game_state.current_map_col = map_col;
                            let new_map_key = (map_row, map_col);
                            if !game_state.loaded_maps.contains_key(&new_map_key) {
                                let new_map_name = format!("map_{}_{}", map_row, map_col);
                                if let Ok(new_map) = crate::game::map::Map::load(&new_map_name) {
                                    game_state.loaded_maps.insert(new_map_key, new_map);
                                } else {
                                    game_state.message = format!("Failed to load map: {}", new_map_name);
                                    game_state.show_message = true;
                                    game_state.message_animation_start_time = Instant::now();
                                    game_state.animated_message_content.clear();
                                }
                            }
                            game_state.current_map_name = format!("map_{}_{}", map_row, map_col);
                        }
                    } else {
                        game_state.skip_message_animation();
                    }
                } else if key.code == KeyCode::Char(' ') {
                    game_state.skip_message_animation();
                }
            } else {
                match key.kind {
                    event::KeyEventKind::Press => {
                        key_states.insert(key.code, true);
                        if game_state.is_map_kind_selection_active {
                            let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                            if let Some(map_to_modify) = game_state.loaded_maps.get_mut(&current_map_key) {
                                match key.code {
                                    KeyCode::Up => map_to_modify.kind = map_to_modify.kind.previous(),
                                    KeyCode::Down => map_to_modify.kind = map_to_modify.kind.next(),
                                    KeyCode::Enter => {
                                        if let Err(e) = map_to_modify.save_data() {
                                            game_state.message = format!("Failed to save map: {}", e);
                                        } else {
                                            game_state.message = format!("Map kind set to {:?}", map_to_modify.kind);
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
                        } else if key.code == KeyCode::Enter {
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
                                for interaction_box in &interaction_boxes {
                                    for select_box in &current_map.select_object_boxes {
                                        let select_box_rect = ratatui::layout::Rect::new(select_box.x as u16, select_box.y as u16, select_box.width as u16, select_box.height as u16);
                                        if interaction_box.intersects(select_box_rect) {
                                            if !select_box.messages.is_empty() {
                                                game_state.message = select_box.messages[0].clone();
                                                game_state.show_message = true;
                                                game_state.message_animation_start_time = Instant::now();
                                                game_state.animated_message_content.clear();
                                                game_state.current_interaction_box_id = Some(select_box.id);
                                                game_state.current_message_index = 0;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        } else if key.code == KeyCode::Char('q') {
                            game_state.save_game_state()?;
                            return Ok(true);
                        } else if key.code == KeyCode::F(1) {
                            game_state.show_debug_panel = !game_state.show_debug_panel;
                            if game_state.show_debug_panel {
                                audio.play_open_settings_sound();
                            }
                        } else if key.code == KeyCode::F(2) {
                            game_state.debug_mode = !game_state.debug_mode;
                        } else if debug::input::handle_debug_input(key, game_state) {
                            // Handled by debug module
                        }
                    }
                    event::KeyEventKind::Release => {
                        key_states.insert(key.code, false);
                        if key.code == KeyCode::Char('o') && game_state.debug_mode {
                            game_state.show_collision_box = false;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(false)
}
