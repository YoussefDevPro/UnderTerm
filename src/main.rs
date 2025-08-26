use std::{
    collections::HashMap, // Added
    io::{self, stdout},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use ansi_to_tui::IntoText;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::Stylize; // Add this line
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};

// Constants for player interaction box (selection box)
const PLAYER_INTERACTION_BOX_WIDTH: u16 = 30;
const PLAYER_INTERACTION_BOX_HEIGHT: u16 = 20;

const TOP_INTERACTION_BOX_HEIGHT: u16 = 10;
const BOTTOM_INTERACTION_BOX_HEIGHT: u16 = 1;
const LEFT_INTERACTION_BOX_WIDTH: u16 = 5;
const RIGHT_INTERACTION_BOX_WIDTH: u16 = 5;

const COLLISION_BOX_WIDTH: u16 = 21;
const COLLISION_BOX_HEIGHT: u16 = 4;

mod audio;
mod game;

use crate::game::config::{ANIMATION_FRAME_DURATION, FRAME_RATE};
use crate::game::state::GameState;

fn run_app() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        if let Err(e) = input_handler(tx) {
            eprintln!("Input handler error: {:?}", e);
        }
    });

    let mut game_state = GameState::load_game_state()?;
    game_state.player.is_walking = false;
    game_state.player.animation_frame = 0;
    let mut last_frame_time = Instant::now();

    let audio = crate::audio::Audio::new().unwrap();

    loop {
        let elapsed_time = last_frame_time.elapsed();
        if elapsed_time >= Duration::from_millis(1000 / FRAME_RATE) {
            last_frame_time = Instant::now();

            let mut key_states: HashMap<KeyCode, bool> = HashMap::new(); // Initialize key states
            while let Ok(event) = rx.try_recv() {
                // Receive generic events
                match event {
                    Event::Key(key) => {
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
                                    // When Enter is pressed, the current text in the buffer is added as a message.
                                    // It is not immediately displayed on the map, but stored for the SelectObjectBox.
                                    if let Some(ref mut pending_box) = game_state.pending_select_box
                                    {
                                        pending_box
                                            .messages
                                            .push(game_state.text_input_buffer.clone());
                                        game_state.text_input_buffer.clear();
                                        game_state.message = format!(
                                            "Message added. Enter next or Esc to finish. Current messages: {}",
                                            pending_box.messages.len()
                                        );
                                        game_state.show_message = true;
                                        game_state.message_animation_start_time = Instant::now();
                                        game_state.animated_message_content.clear();
                                    }
                                }
                                KeyCode::Esc => {
                                    // Finish text input and transition to event input
                                    if let Some(ref mut pending_box) = game_state.pending_select_box
                                    {
                                        // If there's text in the buffer, add it as the last message
                                        if !game_state.text_input_buffer.is_empty() {
                                            pending_box
                                                .messages
                                                .push(game_state.text_input_buffer.clone());
                                            game_state.text_input_buffer.clear();
                                        }
                                    }
                                    game_state.is_text_input_active = false;
                                    game_state.text_input_buffer.clear();
                                    game_state.is_event_input_active = true; // Transition to event input
                                    game_state.message = "Enter events for the new object. Format: 'teleport x y map_row map_col'. Press Enter to add an event, Esc to finish.".to_string();
                                    game_state.show_message = true;
                                    game_state.message_animation_start_time = Instant::now();
                                    game_state.animated_message_content.clear();
                                }
                                _ => {}
                            }
                        } else if game_state.is_event_input_active {
                            // Handle event input
                            match key.code {
                                KeyCode::Char(c) => {
                                    game_state.text_input_buffer.push(c);
                                }
                                KeyCode::Backspace => {
                                    game_state.text_input_buffer.pop();
                                }
                                KeyCode::Enter => {
                                    // When Enter is pressed, the current text in the buffer is parsed as an event
                                    // and added to the pending SelectObjectBox. It is not immediately displayed.
                                    if let Some(ref mut pending_box) = game_state.pending_select_box
                                    {
                                        // Parse event from text_input_buffer
                                        let input = game_state.text_input_buffer.trim();
                                        let parts: Vec<&str> = input.split_whitespace().collect();

                                        if parts.len() >= 5 && parts[0] == "teleport" {
                                            if let (Ok(x), Ok(y), Ok(map_row), Ok(map_col)) = (
                                                parts[1].parse::<u32>(),
                                                parts[2].parse::<u32>(),
                                                parts[3].parse::<i32>(),
                                                parts[4].parse::<i32>(),
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
                                                    "Teleport event added. Enter next or Esc to finish. Current events: {}",
                                                    pending_box.events.len()
                                                );
                                            } else {
                                                game_state.message = "Invalid teleport parameters. Format: 'teleport x y map_row map_col'".to_string();
                                            }
                                        } else {
                                            game_state.message = "Unknown event format. Try 'teleport x y map_row map_col'".to_string();
                                        }
                                        game_state.text_input_buffer.clear();
                                        game_state.show_message = true;
                                        game_state.message_animation_start_time = Instant::now();
                                        game_state.animated_message_content.clear();
                                    }
                                }
                                KeyCode::Esc => {
                                    // Finish event input
                                    if let Some(pending_box) = game_state.pending_select_box.take()
                                    {
                                        let current_map_key = (
                                            game_state.current_map_row,
                                            game_state.current_map_col,
                                        );
                                        if let Some(map_to_modify) =
                                            game_state.loaded_maps.get_mut(&current_map_key)
                                        {
                                            map_to_modify.add_select_object_box(pending_box);
                                            if let Err(e) = map_to_modify.save_data() {
                                                game_state.message = 
                                                    format!("Failed to save map data: {}", e);
                                                game_state.show_message = true;
                                                game_state.message_animation_start_time = 
                                                    Instant::now();
                                                game_state.animated_message_content.clear();
                                            } else {
                                                game_state.message = "SelectObjectBox created and saved with messages and events.".to_string();
                                                game_state.show_message = true;
                                                game_state.message_animation_start_time = 
                                                    Instant::now();
                                                game_state.animated_message_content.clear();
                                            }
                                        }
                                    }
                                    game_state.is_event_input_active = false;
                                    game_state.text_input_buffer.clear();
                                    game_state.pending_select_box = None;
                                }
                                _ => {}
                            }
                        } else if game_state.show_message {
                            // If message is active, handle message-related input
                            if key.code == KeyCode::Enter || key.code == KeyCode::Char('o') {
                                // Handle Enter or 'o'
                                if game_state.message_animation_finished {
                                    // Only advance/dismiss if animation is finished
                                    let mut teleport_target: Option<(u16, u16, i32, i32)> = None; // Declare here

                                    // Extract necessary data before current_map borrow is released
                                    let current_map_key_for_events =
                                        (game_state.current_map_row, game_state.current_map_col);
                                    let box_id_for_events = game_state.current_interaction_box_id;

                                    // Dismiss message and clear interaction state
                                    game_state.dismiss_message();
                                    game_state.current_interaction_box_id = None; // Clear interaction
                                    game_state.current_message_index = 0; // Reset index

                                    // Now, safely perform mutable operations on game_state
                                    if let Some(box_id) = box_id_for_events {
                                        if let Some(current_map) =
                                            game_state.loaded_maps.get(&current_map_key_for_events)
                                        {
                                            if let Some(interacting_box) = current_map
                                                .select_object_boxes
                                                .iter()
                                                .find(|b| b.id == box_id)
                                            {
                                                if game_state.current_message_index + 1
                                                    < interacting_box.messages.len()
                                                {
                                                    // Advance to next message
                                                    game_state.current_message_index += 1;
                                                    game_state.message = interacting_box.messages
                                                        [game_state.current_message_index]
                                                        .clone();
                                                    game_state.show_message = true;
                                                    game_state.message_animation_start_time =
                                                        Instant::now();
                                                    game_state.animated_message_content.clear();
                                                    game_state.message_animation_finished = false;
                                                // Reset for new message
                                                } else {
                                                    // Collect teleport target before current_map is dropped
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

                                    // Execute teleport after all borrows are released
                                    if let Some((x, y, map_row, map_col)) = teleport_target {
                                        game_state.player.x = x;
                                        game_state.player.y = y;
                                        game_state.current_map_row = map_row;
                                        game_state.current_map_col = map_col;

                                        // Load the new map if not already loaded
                                        let new_map_key = (map_row, map_col);
                                        if !game_state.loaded_maps.contains_key(&new_map_key) {
                                            let new_map_name =
                                                format!("map_{}_{}", map_row, map_col);
                                            match crate::game::map::Map::load(&new_map_name) {
                                                Ok(new_map) => {
                                                    game_state
                                                        .loaded_maps
                                                        .insert(new_map_key, new_map);
                                                }
                                                Err(e) => {
                                                    game_state.message = format!(
                                                        "Failed to load map for teleport: {}",
                                                        e
                                                    );
                                                    game_state.show_message = true;
                                                    game_state.message_animation_start_time =
                                                        Instant::now();
                                                    game_state.animated_message_content.clear();
                                                }
                                            }
                                        }
                                        game_state.current_map_name =
                                            format!("map_{}_{}", map_row, map_col);
                                    }
                                } else {
                                    // Animation not finished, skip it
                                    game_state.skip_message_animation();
                                }
                            } else if key.code == KeyCode::Char(' ') {
                                // Spacebar to skip animation
                                game_state.skip_message_animation();
                            }
                        } else {
                            // Otherwise, handle normal game input
                            match key.kind {
                                event::KeyEventKind::Press => {
                                    key_states.insert(key.code, true);
                                    if game_state.is_map_kind_selection_active {
                                        // Handle MapKind selection input
                                        let current_map_key = (
                                            game_state.current_map_row,
                                            game_state.current_map_col,
                                        );
                                        if let Some(map_to_modify) =
                                            game_state.loaded_maps.get_mut(&current_map_key)
                                        {
                                            match key.code {
                                                KeyCode::Up => {
                                                    map_to_modify.kind = match map_to_modify.kind {
                                                        crate::game::map::MapKind::Walls => {
                                                            crate::game::map::MapKind::Empty
                                                        }
                                                        crate::game::map::MapKind::Objects => {
                                                            crate::game::map::MapKind::Walls
                                                        }
                                                        crate::game::map::MapKind::Empty => {
                                                            crate::game::map::MapKind::Objects
                                                        }
                                                    };
                                                    game_state.message = format!(
                                                        "Map Kind: {:?}",
                                                        map_to_modify.kind
                                                    );
                                                    game_state.show_message = true;
                                                    game_state.message_animation_start_time =
                                                        Instant::now();
                                                    game_state.animated_message_content.clear();
                                                }
                                                KeyCode::Down => {
                                                    map_to_modify.kind = match map_to_modify.kind {
                                                        crate::game::map::MapKind::Walls => {
                                                            crate::game::map::MapKind::Objects
                                                        }
                                                        crate::game::map::MapKind::Objects => {
                                                            crate::game::map::MapKind::Empty
                                                        }
                                                        crate::game::map::MapKind::Empty => {
                                                            crate::game::map::MapKind::Walls
                                                        }
                                                    };
                                                    game_state.message = format!(
                                                        "Map Kind: {:?}",
                                                        map_to_modify.kind
                                                    );
                                                    game_state.show_message = true;
                                                    game_state.message_animation_start_time =
                                                        Instant::now();
                                                    game_state.animated_message_content.clear();
                                                }
                                                KeyCode::Enter => {
                                                    if let Err(e) = map_to_modify.save_data() {
                                                        game_state.message = format!(
                                                            "Failed to save map data: {}",
                                                            e
                                                        );
                                                        game_state.show_message = true;
                                                        game_state.message_animation_start_time =
                                                            Instant::now();
                                                        game_state.animated_message_content.clear();
                                                    } else {
                                                        game_state.message = format!(
                                                            "Map Kind set to {:?} and saved.",
                                                            map_to_modify.kind
                                                        );
                                                        game_state.show_message = true;
                                                        game_state.message_animation_start_time =
                                                            Instant::now();
                                                        game_state.animated_message_content.clear();
                                                    }
                                                    game_state.is_map_kind_selection_active = false;
                                                }
                                                KeyCode::Esc => {
                                                    game_state.is_map_kind_selection_active = false;
                                                    game_state.message =
                                                        "Map Kind Selection cancelled.".to_string();
                                                    game_state.show_message = true;
                                                    game_state.message_animation_start_time =
                                                        Instant::now();
                                                    game_state.animated_message_content.clear();
                                                }
                                                _ => {}
                                            }
                                        }
                                    } else {
                                        // Normal game input
                                        if key.code == KeyCode::Enter {
                                            // Handle Enter key for interaction in normal mode
                                            let (_, _, player_sprite_height) =
                                                game_state.player.get_sprite_content();
                                            let collision_box_x = game_state.player.x;
                                            let collision_box_y = game_state
                                                .player
                                                .y
                                                .saturating_add(player_sprite_height)
                                                .saturating_sub(COLLISION_BOX_HEIGHT);
                                            let collision_box = ratatui::layout::Rect::new(
                                                collision_box_x,
                                                collision_box_y,
                                                COLLISION_BOX_WIDTH,
                                                COLLISION_BOX_HEIGHT,
                                            );

                                            let top_box = ratatui::layout::Rect::new(
                                                collision_box.x,
                                                collision_box.y.saturating_sub(TOP_INTERACTION_BOX_HEIGHT),
                                                collision_box.width,
                                                TOP_INTERACTION_BOX_HEIGHT,
                                            );
                                            let bottom_box = ratatui::layout::Rect::new(
                                                collision_box.x,
                                                collision_box.y + collision_box.height,
                                                collision_box.width,
                                                BOTTOM_INTERACTION_BOX_HEIGHT,
                                            );
                                            let left_box = ratatui::layout::Rect::new(
                                                collision_box.x.saturating_sub(LEFT_INTERACTION_BOX_WIDTH),
                                                collision_box.y,
                                                LEFT_INTERACTION_BOX_WIDTH,
                                                collision_box.height,
                                            );
                                            let right_box = ratatui::layout::Rect::new(
                                                collision_box.x + collision_box.width,
                                                collision_box.y,
                                                RIGHT_INTERACTION_BOX_WIDTH,
                                                collision_box.height,
                                            );
                                            let interaction_boxes = [top_box, bottom_box, left_box, right_box];

                                            let current_map_key = (
                                                game_state.current_map_row,
                                                game_state.current_map_col,
                                            );
                                            if let Some(current_map) =
                                                game_state.loaded_maps.get(&current_map_key)
                                            {
                                                'outer: for interaction_box in &interaction_boxes {
                                                    for select_box in &current_map.select_object_boxes {
                                                        let select_box_rect =
                                                            ratatui::layout::Rect::new(
                                                                select_box.x as u16,
                                                                select_box.y as u16,
                                                                select_box.width as u16,
                                                                select_box.height as u16,
                                                            );

                                                        if interaction_box.intersects(select_box_rect) {
                                                            if !select_box.messages.is_empty() {
                                                                game_state.message =
                                                                    select_box.messages[0].clone();
                                                                game_state.show_message = true;
                                                                game_state
                                                                    .message_animation_start_time =
                                                                    Instant::now();
                                                                game_state
                                                                    .animated_message_content
                                                                    .clear();
                                                                game_state.current_interaction_box_id =
                                                                    Some(select_box.id);
                                                                game_state.current_message_index = 0;
                                                            }
                                                            break 'outer;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if key.code == KeyCode::Char('q') {
                                            game_state.save_game_state()?;
                                            return Ok(());
                                        } else if key.code == KeyCode::F(1) {
                                            game_state.show_debug_panel =
                                                !game_state.show_debug_panel;
                                            if game_state.show_debug_panel {
                                                audio.play_open_settings_sound();
                                            }
                                        } else if key.code == KeyCode::F(2) {
                                            game_state.debug_mode = !game_state.debug_mode;
                                        } else if key.code == KeyCode::Enter {
                                            // Handle Enter key for confirming object creation
                                            if game_state.is_drawing_select_box {
                                                // Confirm object creation
                                                if let Some((start_x, start_y)) = 
                                                    game_state.select_box_start_coords
                                                {
                                                    let end_x = game_state.player.x;
                                                    let end_y = game_state.player.y;

                                                    let min_x = start_x.min(end_x);
                                                    let max_x = start_x.max(end_x);
                                                    let min_y = start_y.min(end_y);
                                                    let max_y = start_y.max(end_y);

                                                    let width = max_x
                                                        .saturating_sub(min_x)
                                                        .saturating_add(1);
                                                    let height = max_y
                                                        .saturating_sub(min_y)
                                                        .saturating_add(1);

                                                    if width > 0 && height > 0 {
                                                        let current_map_key = (
                                                            game_state.current_map_row,
                                                            game_state.current_map_col,
                                                        );
                                                        if let Some(map_to_modify) = game_state
                                                            .loaded_maps
                                                            .get_mut(&current_map_key)
                                                        {
                                                            // Generate a simple ID (e.g., max existing ID + 1)
                                                            let new_id = map_to_modify
                                                                .select_object_boxes
                                                                .iter()
                                                                .map(|b| b.id)
                                                                .max()
                                                                .unwrap_or(0)
                                                                + 1;

                                                            let new_select_box =
                                                                crate::game::map::SelectObjectBox {
                                                                    id: new_id,
                                                                    x: min_x as u32,
                                                                    y: min_y as u32,
                                                                    width: width as u32,
                                                                    height: height as u32,
                                                                    messages: Vec::new(), // Start with empty messages
                                                                    events: Vec::new(), // Initialize with empty events
                                                                };
                                                            game_state.pending_select_box =
                                                                Some(new_select_box); // Store for text input
                                                            game_state.is_text_input_active = true;
                                                            game_state.text_input_buffer.clear();
                                                            game_state.message = "Enter messages for the new object. Press Enter to add a message, Esc to finish.".to_string();
                                                            game_state.show_message = true;
                                                            game_state
                                                                .message_animation_start_time =
                                                                Instant::now();
                                                            game_state
                                                                .animated_message_content
                                                                .clear();
                                                        }
                                                    }
                                                }
                                                game_state.is_drawing_select_box = false;
                                                game_state.select_box_start_coords = None;
                                            }
                                        } else if key.code == KeyCode::Char('o') {
                                            // Handle 'o' key for debug drawing
                                            if game_state.is_drawing_select_box {
                                                // If already drawing, cancel
                                                game_state.is_drawing_select_box = false;
                                                game_state.select_box_start_coords = None;
                                                game_state.show_collision_box = false;
                                            // Also turn off collision box visualization
                                            } else {
                                                // Start drawing
                                                game_state.is_drawing_select_box = true;
                                                game_state.select_box_start_coords = Some((
                                                    game_state.player.x,
                                                    game_state.player.y,
                                                ));
                                                game_state.show_collision_box = false;
                                                // Turn off collision box visualization
                                            }
                                        } else if game_state.debug_mode {
                                            // Debug-specific keys
                                            // Handle debug-specific keys
                                            if key.code == KeyCode::Char('r') {
                                                game_state.undo_wall_change();
                                            } else if key.code == KeyCode::Char('z') {
                                                game_state.redo_wall_change();
                                            } else if key.code == KeyCode::Char('s') {
                                                game_state.set_player_spawn_to_current_position(
                                                    game_state.player.x,
                                                    game_state.player.y,
                                                );
                                            } else if key.code == KeyCode::Char('k') {
                                                // Handle 'k' key for MapKind selection
                                                game_state.is_map_kind_selection_active =
                                                    !game_state.is_map_kind_selection_active;
                                                if game_state.is_map_kind_selection_active {
                                                    game_state.message = "Map Kind Selection: Use Up/Down to cycle, Enter to confirm, Esc to cancel.".to_string();
                                                    game_state.show_message = true;
                                                    game_state.message_animation_start_time =
                                                        Instant::now();
                                                    game_state.animated_message_content.clear();
                                                } else {
                                                    game_state.message =
                                                        "Map Kind Selection cancelled.".to_string();
                                                    game_state.show_message = true;
                                                    game_state.message_animation_start_time =
                                                        Instant::now();
                                                    game_state.animated_message_content.clear();
                                                }
                                            }
                                        }
                                    }
                                }
                                event::KeyEventKind::Release => {
                                    key_states.insert(key.code, false);
                                    if key.code == KeyCode::Char('o') {
                                        // Handle 'o' key release
                                        if game_state.debug_mode {
                                            game_state.show_collision_box = false;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {} // Ignore other events
                }
            }

            let current_frame_size = terminal.size()?;

            game_state.update(
                &key_states, // Pass reference to key_states
                ratatui::layout::Rect::new(
                    0,
                    0,
                    current_frame_size.width,
                    current_frame_size.height,
                ),
                ANIMATION_FRAME_DURATION,
            );

            terminal.draw(|frame| {
                let size = frame.area();
                frame.render_widget(Block::default().bg(Color::Black), size); // Draw a solid black background block

                let player_x_on_screen = game_state.player.x.saturating_sub(game_state.camera_x);
                let player_y_on_screen = game_state.player.y.saturating_sub(game_state.camera_y);

                // Get the combined map text from GameState
                let combined_map_text = game_state.get_combined_map_text(size);

                let map_paragraph = Paragraph::new(combined_map_text)
                    .scroll((game_state.camera_y, game_state.camera_x)) // Use camera_y and camera_x
                    .style(Style::default().bg(Color::Black)); // Ensure map background is black

                // Create a block to ensure the entire area is covered by the map background
                let map_block = Block::default().style(Style::default().bg(Color::Black));
                frame.render_widget(map_block, size); // Draw the background block first
                frame.render_widget(map_paragraph, size); // Then draw the map content on top

                let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                let current_map = game_state.loaded_maps.get(&current_map_key).unwrap();

                match current_map.kind {
                    crate::game::map::MapKind::Walls => {
                        // Render walls only if not in debug mode
                        if game_state.debug_mode {
                            for &(wx, wy) in &current_map.walls {
                                let wall_x_on_screen =
                                    wx.saturating_sub(game_state.camera_x as u32);
                                let wall_y_on_screen =
                                    wy.saturating_sub(game_state.camera_y as u32);

                                if wall_x_on_screen < size.width as u32
                                    && wall_y_on_screen < size.height as u32
                                {
                                    let draw_rect = ratatui::layout::Rect::new(
                                        wall_x_on_screen as u16,
                                        wall_y_on_screen as u16,
                                        1,
                                        1,
                                    );
                                    let clamped_rect = draw_rect.intersection(size);
                                    if !clamped_rect.is_empty() {
                                        let wall_paragraph = Paragraph::new("W").style(
                                            Style::default().fg(Color::Red).bg(Color::Black),
                                        );
                                        frame.render_widget(wall_paragraph, clamped_rect);
                                    }
                                }
                            }
                        }
                    }
                    crate::game::map::MapKind::Objects => {
                        // Render SelectObjectBoxes only
                        // Render the dynamically created "select object box" around the player
                        let (_player_sprite_content, player_sprite_width, player_sprite_height) =
                            game_state.player.get_sprite_content();

                        let (select_box_width, select_box_height) =
                            (PLAYER_INTERACTION_BOX_WIDTH, PLAYER_INTERACTION_BOX_HEIGHT);

                        let (select_box_x, select_box_y) = match game_state.player.direction {
                            crate::game::player::PlayerDirection::Front => (
                                game_state
                                    .player
                                    .x
                                    .saturating_add(player_sprite_width / 2)
                                    .saturating_sub(PLAYER_INTERACTION_BOX_WIDTH / 2),
                                game_state.player.y.saturating_add(player_sprite_height),
                            ),
                            crate::game::player::PlayerDirection::Back => (
                                game_state
                                    .player
                                    .x
                                    .saturating_add(player_sprite_width / 2)
                                    .saturating_sub(PLAYER_INTERACTION_BOX_WIDTH / 2),
                                game_state
                                    .player
                                    .y
                                    .saturating_sub(PLAYER_INTERACTION_BOX_HEIGHT),
                            ),
                            crate::game::player::PlayerDirection::Left => (
                                game_state
                                    .player
                                    .x
                                    .saturating_sub(PLAYER_INTERACTION_BOX_WIDTH),
                                game_state
                                    .player
                                    .y
                                    .saturating_add(player_sprite_height / 2)
                                    .saturating_sub(PLAYER_INTERACTION_BOX_HEIGHT / 2),
                            ),
                            crate::game::player::PlayerDirection::Right => (
                                game_state.player.x.saturating_add(player_sprite_width),
                                game_state
                                    .player
                                    .y
                                    .saturating_add(player_sprite_height / 2)
                                    .saturating_sub(PLAYER_INTERACTION_BOX_HEIGHT / 2),
                            ),
                        };

                        let select_box_x_on_screen =
                            select_box_x.saturating_sub(game_state.camera_x);
                        let select_box_y_on_screen =
                            select_box_y.saturating_sub(game_state.camera_y);

                        let draw_rect = ratatui::layout::Rect::new(
                            select_box_x_on_screen,
                            select_box_y_on_screen,
                            select_box_width,
                            select_box_height,
                        );

                        let clamped_rect = draw_rect.intersection(size);
                        if !clamped_rect.is_empty() {
                            let select_box_paragraph = Paragraph::new("").block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .border_style(Style::default().fg(Color::Green)), // Use Green for player's interaction box
                            );
                            frame.render_widget(select_box_paragraph, clamped_rect);
                        }

                        // Existing rendering of map's SelectObjectBoxes
                        for select_box in &current_map.select_object_boxes {
                            let select_box_x_on_screen =
                                select_box.x.saturating_sub(game_state.camera_x as u32);
                            let select_box_y_on_screen =
                                select_box.y.saturating_sub(game_state.camera_y as u32);

                            let draw_rect = ratatui::layout::Rect::new(
                                select_box_x_on_screen as u16,
                                select_box_y_on_screen as u16,
                                select_box.width as u16,
                                select_box.height as u16,
                            );

                            let clamped_rect = draw_rect.intersection(size);
                            if !clamped_rect.is_empty() {
                                let select_box_paragraph = Paragraph::new("").block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .border_style(Style::default().fg(Color::Cyan)),
                                );
                                frame.render_widget(select_box_paragraph, clamped_rect);
                            }
                        }
                    }
                    crate::game::map::MapKind::Empty => {
                        // Render nothing specific for Empty map kind
                    }
                }

                let (player_sprite_content, player_sprite_width, player_sprite_height) =
                    if game_state.debug_mode {
                        ("P".to_string().as_bytes().into_text().unwrap(), 1u16, 1u16)
                    // 'P' is 1x1
                    } else {
                        game_state.player.get_sprite_content()
                    };
                let player_paragraph = Paragraph::new(player_sprite_content);
                // The player sprite itself will not have a special background in debug mode.
                // Debug boxes will be drawn on top of it.

                let player_draw_rect = ratatui::layout::Rect::new(
                    player_x_on_screen,
                    player_y_on_screen,
                    player_sprite_width,
                    player_sprite_height,
                );
                // Ensure player_draw_rect is within size
                let clamped_player_rect = player_draw_rect.intersection(size);
                if game_state.debug_mode || !clamped_player_rect.is_empty() {
                    frame.render_widget(player_paragraph, clamped_player_rect);
                }

                // Debug mode specific rendering (always render these if debug_mode is true)
                if game_state.debug_mode {
                    // Draw spawn point 'S'
                    let spawn_x = game_state.player.x; // Use player's current x
                    let spawn_y = game_state.player.y; // Use player's current y

                    let spawn_x_on_screen = spawn_x.saturating_sub(game_state.camera_x);
                    let spawn_y_on_screen = spawn_y.saturating_sub(game_state.camera_y);

                    // Only draw if within screen bounds (or always in debug mode)
                    if game_state.debug_mode
                        || (spawn_x_on_screen < size.width && spawn_y_on_screen < size.height)
                    {
                        let draw_rect =
                            ratatui::layout::Rect::new(spawn_x_on_screen, spawn_y_on_screen, 1, 1);
                        let clamped_rect = draw_rect.intersection(size);
                        if game_state.debug_mode || !clamped_rect.is_empty() {
                            let spawn_paragraph = Paragraph::new("S")
                                .style(Style::default().fg(Color::Green).bg(Color::Black));
                            frame.render_widget(spawn_paragraph, clamped_rect);
                        }
                    }

                    // Draw player collision box (always in debug mode)
                    let (_, _, player_sprite_height) =
                        game_state.player.get_sprite_content();
                    let collision_box_start_x = game_state.player.x;
                    let collision_box_start_y = game_state
                        .player
                        .y
                        .saturating_add(player_sprite_height)
                        .saturating_sub(COLLISION_BOX_HEIGHT);

                    let collision_box_x_on_screen =
                        collision_box_start_x.saturating_sub(game_state.camera_x);
                    let collision_box_y_on_screen =
                        collision_box_start_y.saturating_sub(game_state.camera_y);

                    let draw_rect = ratatui::layout::Rect::new(
                        collision_box_x_on_screen,
                        collision_box_y_on_screen,
                        COLLISION_BOX_WIDTH,
                        COLLISION_BOX_HEIGHT,
                    );

                    let clamped_rect = draw_rect.intersection(size);
                    if !clamped_rect.is_empty() {
                        let collision_box_paragraph = Paragraph::new("").block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(Color::Blue)), // Changed to Blue
                        );
                        frame.render_widget(collision_box_paragraph, clamped_rect);
                    }

                    let collision_box = ratatui::layout::Rect::new(
                        collision_box_start_x,
                        collision_box_start_y,
                        COLLISION_BOX_WIDTH,
                        COLLISION_BOX_HEIGHT,
                    );

                    let top_box = ratatui::layout::Rect::new(
                        collision_box.x,
                        collision_box.y.saturating_sub(TOP_INTERACTION_BOX_HEIGHT),
                        collision_box.width,
                        TOP_INTERACTION_BOX_HEIGHT,
                    );
                    let bottom_box = ratatui::layout::Rect::new(
                        collision_box.x,
                        collision_box.y + collision_box.height,
                        collision_box.width,
                        BOTTOM_INTERACTION_BOX_HEIGHT,
                    );
                    let left_box = ratatui::layout::Rect::new(
                        collision_box.x.saturating_sub(LEFT_INTERACTION_BOX_WIDTH),
                        collision_box.y,
                        LEFT_INTERACTION_BOX_WIDTH,
                        collision_box.height,
                    );
                    let right_box = ratatui::layout::Rect::new(
                        collision_box.x + collision_box.width,
                        collision_box.y,
                        RIGHT_INTERACTION_BOX_WIDTH,
                        collision_box.height,
                    );
                    let interaction_boxes = [top_box, bottom_box, left_box, right_box];

                    for box_rect in &interaction_boxes {
                        let box_on_screen_x = box_rect.x.saturating_sub(game_state.camera_x);
                        let box_on_screen_y = box_rect.y.saturating_sub(game_state.camera_y);

                        let draw_rect = ratatui::layout::Rect::new(
                            box_on_screen_x,
                            box_on_screen_y,
                            box_rect.width,
                            box_rect.height,
                        );

                        let clamped_rect = draw_rect.intersection(size);
                        if !clamped_rect.is_empty() {
                            let box_paragraph = Paragraph::new("").block(
                                Block::default()
                                    .borders(Borders::ALL)
                                    .border_style(Style::default().fg(Color::Yellow)),
                            );
                            frame.render_widget(box_paragraph, clamped_rect);
                        }
                    }
                }

                if game_state.is_drawing_select_box {
                    // Draw the dynamic select box being created
                    if let Some((start_x, start_y)) = game_state.select_box_start_coords {
                        let current_x = game_state.player.x;
                        let current_y = game_state.player.y;

                        let min_x = start_x.min(current_x);
                        let max_x = start_x.max(current_x);
                        let min_y = start_y.min(current_y);
                        let max_y = start_y.max(current_y);

                        let width = max_x.saturating_sub(min_x).saturating_add(1);
                        let height = max_y.saturating_sub(min_y).saturating_add(1);

                        if width > 0 && height > 0 {
                            let draw_rect = ratatui::layout::Rect::new(
                                min_x.saturating_sub(game_state.camera_x),
                                min_y.saturating_sub(game_state.camera_y),
                                width,
                                height,
                            );

                            let clamped_rect = draw_rect.intersection(size);
                            if !clamped_rect.is_empty() {
                                let drawing_box_paragraph = Paragraph::new("").block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .border_style(Style::default().fg(Color::Yellow)), // Changed to Yellow, removed background
                                );
                                frame.render_widget(drawing_box_paragraph, clamped_rect);
                            }
                        }
                    }
                }

                if game_state.show_debug_panel {
                    let debug_text = vec![
                        format!("Player: ({}, {})", game_state.player.x, game_state.player.y),
                        format!("Direction: {:?}", game_state.player.direction),
                        format!("Animation Frame: {}", game_state.player.animation_frame),
                        format!("Is Walking: {}", game_state.player.is_walking),
                        format!("Camera: ({}, {})", game_state.camera_x, game_state.camera_y),
                        format!("Map: ({}, {})", current_map.width, current_map.height),
                        format!(
                            "Screen Player Pos: ({}, {})",
                            player_x_on_screen, player_y_on_screen
                        ),
                        format!("Debug Mode: {}", game_state.debug_mode),
                        format!(
                            "Current Map: {} ({}, {})",
                            game_state.current_map_name,
                            game_state.current_map_row,
                            game_state.current_map_col
                        ),
                        format!("Anim Frame Duration: {:?}", ANIMATION_FRAME_DURATION),
                        format!("Map Kind: {:?}", current_map.kind), // Added Map Kind
                    ];

                    let debug_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Thick)
                        .border_style(Style::default().fg(Color::White).bg(Color::White)) // Border color and background
                        .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0))) // Content color and background
                        .padding(ratatui::widgets::Padding::new(1, 1, 1, 1)) // Add padding (left, right, top, bottom)
                        .title("Debug Panel");

                    let debug_paragraph = Paragraph::new(debug_text.join("\n"))
                        .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0))) // Explicitly set style for Paragraph content
                        .block(debug_block);

                    let max_line_length =
                        debug_text.iter().map(|s| s.len() as u16).max().unwrap_or(0);
                    let area = size; // Reintroduce area
                    let debug_panel_width = max_line_length + 2 + 2; // +2 for borders, +2 for horizontal padding (1 left, 1 right)
                    let debug_panel_height = debug_text.len() as u16 + 4; // +2 for borders, +2 for vertical padding

                    let margin = 2; // Define margin value
                    let x = area.width.saturating_sub(debug_panel_width + margin); // Right margin
                    let y = margin; // Top margin

                    let debug_panel_rect =
                        ratatui::layout::Rect::new(x, y, debug_panel_width, debug_panel_height);
                    frame.render_widget(Clear, debug_panel_rect); // Clear the area before drawing the panel

                    frame.render_widget(debug_paragraph, debug_panel_rect);
                }

                if game_state.show_message {
                    let message_block = Block::default() 
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Thick)
                        .border_style(Style::default().fg(Color::White).bg(Color::White))
                        .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
                        .padding(ratatui::widgets::Padding::new(8, 8, 1, 1)) // 8 left/right padding
                        .title("Message");

                    let message_paragraph =
                        Paragraph::new(game_state.animated_message_content.clone())
                            .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
                            .block(message_block);

                    let message_height = 10; // Set message box height to 10 lines
                    let bottom_margin = 5; // 5 lines bottom margin
                    let horizontal_margin = 40; // 15 lines horizontal margin (increased for smaller width)

                    let message_area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(message_height),
                            Constraint::Length(bottom_margin),
                        ])
                        .split(size)[1];

                    let message_area = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(horizontal_margin),
                            Constraint::Min(0),
                            Constraint::Length(horizontal_margin),
                        ])
                        .split(message_area)[1];

                    frame.render_widget(Clear, message_area); // Clear the area before drawing the panel
                    frame.render_widget(message_paragraph, message_area);
                }

                if game_state.is_text_input_active {
                    let input_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Thick)
                        .border_style(Style::default().fg(Color::White).bg(Color::White))
                        .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
                        .padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
                        .title("Enter Message");

                    let input_paragraph = Paragraph::new(game_state.text_input_buffer.clone())
                        .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
                        .block(input_block);

                    let input_area_height = 3; // 1 line for text, 2 for borders
                    let input_area_width = 60;

                    let input_area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(input_area_height),
                            Constraint::Length(1),
                        ])
                        .split(size)[1];

                    let input_area = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(input_area_width),
                            Constraint::Min(0),
                        ])
                        .split(input_area)[1];

                    frame.render_widget(Clear, input_area); // Clear the area before drawing
                    frame.render_widget(input_paragraph, input_area);
                }

                if game_state.is_event_input_active {
                    let input_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Thick)
                        .border_style(Style::default().fg(Color::White).bg(Color::White))
                        .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
                        .padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
                        .title("Enter Event");

                    let input_paragraph = Paragraph::new(game_state.text_input_buffer.clone())
                        .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
                        .block(input_block);

                    let input_area_height = 3; // 1 line for text, 2 for borders
                    let input_area_width = 60;

                    let input_area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(input_area_height),
                            Constraint::Length(1),
                        ])
                        .split(size)[1];

                    let input_area = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(input_area_width),
                            Constraint::Min(0),
                        ])
                        .split(input_area)[1];

                    frame.render_widget(Clear, input_area); // Clear the area before drawing
                    frame.render_widget(input_paragraph, input_area);
                }

                if game_state.is_map_kind_selection_active {
                    let input_block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(ratatui::widgets::BorderType::Thick)
                        .border_style(Style::default().fg(Color::White).bg(Color::White))
                        .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
                        .padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
                        .title("Select Map Kind");

                    let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                    let current_map_kind = game_state
                        .loaded_maps
                        .get(&current_map_key)
                        .map(|m| format!("{:?}", m.kind))
                        .unwrap_or_else(|| "Unknown".to_string());

                    let input_paragraph = Paragraph::new(format!("Current: {}", current_map_kind))
                        .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
                        .block(input_block);

                    let input_area_height = 5; // A bit more height for context
                    let input_area_width = 40;

                    let input_area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(input_area_height),
                            Constraint::Length(1),
                        ])
                        .split(size)[1];

                    let input_area = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(input_area_width),
                            Constraint::Min(0),
                        ])
                        .split(input_area)[1];

                    frame.render_widget(Clear, input_area); // Clear the area before drawing
                    frame.render_widget(input_paragraph, input_area);
                }
            })?;
        }
    }
}

fn input_handler(tx: mpsc::Sender<Event>) -> io::Result<()> {
    loop {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                tx.send(Event::Key(key))
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?; // Send KeyEvent directly
            }
        }
    }
}

fn main() -> io::Result<()> {
    let result = run_app();

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}