use crate::debug;
use std::{
    collections::HashMap,
    io,
    sync::mpsc,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};
use serde_json;


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
    

    while let Ok(event) = rx.try_recv() {
        if let Event::Key(key) = event {
            if game_state.is_text_input_active {
                match key.code {
                    KeyCode::Char(c) => {
                        if game_state.teleport_creation_state == TeleportCreationState::EnteringMapName {
                            game_state.teleport_destination_map_name_buffer.push(c);
                        } else {
                            game_state.text_input_buffer.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if game_state.teleport_creation_state == TeleportCreationState::EnteringMapName {
                            game_state.teleport_destination_map_name_buffer.pop();
                        } else {
                            game_state.text_input_buffer.pop();
                        }
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
                            game_state.is_text_input_active = false;
                            game_state.text_input_buffer.clear();
                            game_state.show_message = true;
                            game_state.message_animation_start_time = Instant::now();
                            game_state.animated_message_content.clear();
                        } else if game_state.teleport_creation_state
                            == TeleportCreationState::EnteringMapName
                        {
                            let target_map_name = game_state
                                .teleport_destination_map_name_buffer
                                .trim()
                                .to_string();
                            if target_map_name.is_empty() {
                                game_state.message = "Target map name cannot be empty.".to_string();
                                game_state.show_message = true;
                                game_state.message_animation_start_time = Instant::now();
                                game_state.animated_message_content.clear();
                            } else {
                                let parts: Vec<&str> = target_map_name.split('_').collect();
                                if parts.len() == 3 && parts[0] == "map" {
                                    if let (Ok(map_row), Ok(map_col)) =
                                        (parts[1].parse::<i32>(), parts[2].parse::<i32>())
                                    {
                                        match crate::game::map::Map::load(&target_map_name) {
                                            Ok(target_map) => {
                                                if let Some(pending_box) =
                                                    &game_state.pending_select_box
                                                {
                                                    let current_map_key = (
                                                        game_state.current_map_row,
                                                        game_state.current_map_col,
                                                    );
                                                    if let Some(map_to_modify) = game_state
                                                        .loaded_maps
                                                        .get_mut(&current_map_key)
                                                    {
                                                        if let Some(box_to_update) = map_to_modify
                                                            .select_object_boxes
                                                            .iter_mut()
                                                            .find(|b| b.id == pending_box.id)
                                                        {
                                                            box_to_update.events.push(
                                                                crate::game::map::Event::TeleportPlayer {
                                                                    map_row,
                                                                    map_col,
                                                                    dest_x: target_map.player_spawn.0,
                                                                    dest_y: target_map.player_spawn.1,
                                                                },
                                                            );

                                                            if let Err(e) = map_to_modify.save_data()
                                                            {
                                                                game_state.message = format!(
                                                                    "Failed to save map data: {}",
                                                                    e
                                                                );
                                                            } else {
                                                                game_state.message = format!(
                                                                    "Teleport event to {} added and saved.",
                                                                    target_map_name
                                                                );
                                                            }
                                                        } else {
                                                            game_state.message = "Error: Could not find the box to update in the current map.".to_string();
                                                        }
                                                    } else {
                                                        game_state.message = "Error: Current map not found for saving.".to_string();
                                                    }
                                                } else {
                                                    game_state.message = "Error: No pending select box to add event to.".to_string();
                                                }
                                            }
                                            Err(_) => {
                                                game_state.message = format!(
                                                    "Failed to load map data for '{}'.",
                                                    target_map_name
                                                );
                                            }
                                        }
                                        game_state.teleport_creation_state =
                                            TeleportCreationState::None;
                                        game_state.is_text_input_active = false;
                                        game_state.teleport_destination_map_name_buffer.clear();
                                        game_state.pending_select_box = None;
                                        game_state.is_drawing_select_box = false;
                                        game_state.block_player_movement_on_message = true;
                                    } else {
                                        game_state.message = "Invalid map coordinates in name. Format: map_row_col".to_string();
                                    }
                                } else {
                                    game_state.message =
                                        "Invalid map name format. Expected: map_row_col"
                                            .to_string();
                                }
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
                            game_state.pending_select_box = None;
                            game_state.message = "Teleport line creation cancelled.".to_string();
                            game_state.block_player_movement_on_message = true;
                        } else {
                            game_state.is_event_input_active = true;
                            game_state.message = "Enter events. Format: 'teleport map_0_0'. Esc to finish.".to_string();
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
                            if parts.len() == 2 && parts[0] == "teleport" {
                                let target_map_name = parts[1];
                                let map_parts: Vec<&str> = target_map_name.split('_').collect();
                                if map_parts.len() == 3 && map_parts[0] == "map" {
                                    if let (Ok(map_row), Ok(map_col)) =
                                        (map_parts[1].parse(), map_parts[2].parse())
                                    {
                                        match crate::game::map::Map::load(target_map_name) {
                                            Ok(target_map) => {
                                                pending_box.events.push(
                                                    crate::game::map::Event::TeleportPlayer {
                                                        map_row,
                                                        map_col,
                                                        dest_x: target_map.player_spawn.0,
                                                        dest_y: target_map.player_spawn.1,
                                                    },
                                                );
                                                game_state.message = format!(
                                                    "Teleport event added. Current: {}",
                                                    pending_box.events.len()
                                                );
                                            }
                                            Err(_) => {
                                                game_state.message =
                                                    format!("Could not load map {}", target_map_name);
                                            }
                                        }
                                    } else {
                                        game_state.message =
                                            "Invalid map name format in teleport.".to_string();
                                    }
                                } else {
                                    game_state.message =
                                        "Invalid map name format. Expected: map_row_col"
                                            .to_string();
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
                        game_state.is_drawing_select_box = false;
                        game_state.block_player_movement_on_message = true;
                    }
                    _ => {}
                }
                continue;
            }

            match key.kind {
                event::KeyEventKind::Press => {
                    key_states.insert(key.code, true);
                    if key.code == KeyCode::Esc {
                        if game_state.esc_press_start_time.is_none() {
                            game_state.esc_dot_timer = Instant::now();
                        }
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
                    } else if key.code == KeyCode::Enter {
                        // Handle interaction with SelectObjectBox when Enter is pressed
                        if !game_state.show_message {
                            if let Some(box_id) = game_state.current_interaction_box_id {
                                let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                                if let Some(current_map) = game_state.loaded_maps.get(&current_map_key) {
                                    if let Some(interacting_box) = current_map.select_object_boxes.iter().find(|b| b.id == box_id) {
                                        if !interacting_box.messages.is_empty() {
                                            game_state.message = interacting_box.messages[0].clone();
                                            game_state.show_message = true;
                                            game_state.message_animation_start_time = Instant::now();
                                            game_state.animated_message_content.clear();
                                            game_state.message_animation_finished = false;
                                            game_state.current_message_index = 1; // Set to 1 because the first message is now displayed
                                            game_state.block_player_movement_on_message = true;
                                            return Ok(false); // Event handled
                                        }
                                    }
                                }
                            }
                        }

                        if game_state.show_message {
                            if game_state.message_animation_finished {
                                if let Some(box_id) = game_state.current_interaction_box_id {
                                    let current_map_key =
                                        (game_state.current_map_row, game_state.current_map_col);
                                    if let Some(current_map) =
                                        game_state.loaded_maps.get(&current_map_key)
                                    {
                                        if let Some(interacting_box) = current_map
                                            .select_object_boxes
                                            .iter()
                                            .find(|b| b.id == box_id)
                                        {
                                            if game_state.current_message_index
                                                < interacting_box.messages.len()
                                            {
                                                game_state.message = interacting_box.messages
                                                    [game_state.current_message_index]
                                                    .clone();
                                                game_state.show_message = true;
                                                game_state.message_animation_start_time =
                                                    Instant::now();
                                                game_state.animated_message_content.clear();
                                                game_state.message_animation_finished = false;
                                                game_state.current_message_index += 1;
                                            } else {
                                                let events = interacting_box.events.clone();
                                                let box_id = interacting_box.id;
                                                game_state.dismiss_message();
                                                game_state.current_interaction_box_id = None;
                                                game_state.current_message_index = 0;
                                                let _events_to_process: Vec<
                                                    crate::game::map::Event,
                                                > = events;
                                                game_state.recently_teleported_from_box_id =
                                                    Some(box_id);
                                            }
                                        }
                                    }
                                } else {
                                    game_state.dismiss_message();
                                }
                            } else {
                                game_state.skip_message_animation();
                            }
                        }
                    } else if key.code == KeyCode::Char('+') {
                        game_state.deltarune.increase();
                    } else if key.code == KeyCode::Char('-') {
                        game_state.deltarune.decrease();
                    } else if key.code == KeyCode::Char('q') {
                        game_state.save_game_state()?;
                        return Ok(true);
                    } else if key.code == KeyCode::Char('p') {
                        game_state.save_game_state()?;
                        game_state.message = "Game saved!".to_string();
                        game_state.show_message = true;
                        game_state.message_animation_start_time = Instant::now();
                        game_state.animated_message_content.clear();
                    } else if key.code == KeyCode::F(2) {
                        game_state.debug_mode = !game_state.debug_mode;
                        if game_state.debug_mode {
                            audio.play_open_settings_sound();
                        }
                    } else if key.code == KeyCode::Char('m') {
                        let state_json = serde_json::to_string_pretty(game_state).unwrap();
                        std::fs::write("debug.txt", state_json).unwrap();
                        game_state.message = "Game state saved to debug.txt".to_string();
                        game_state.show_message = true;
                        game_state.message_animation_start_time = Instant::now();
                        game_state.animated_message_content.clear();
                    }
                }
                event::KeyEventKind::Release => {
                    key_states.insert(key.code, false);
                    if key.code == KeyCode::Esc {
                        if game_state.esc_press_start_time.is_some() {
                            game_state.esc_dot_timer = Instant::now();
                        }
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