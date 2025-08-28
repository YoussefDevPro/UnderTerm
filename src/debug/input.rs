use crossterm::event::{KeyCode, KeyEvent};
use crate::game::state::GameState;
use std::time::Instant;

pub fn handle_debug_input(key: KeyEvent, game_state: &mut GameState) -> bool {
    if !game_state.debug_mode {
        return false;
    }

    match key.code {
        KeyCode::Char('r') => game_state.undo_wall_change(),
        KeyCode::Char('z') => game_state.redo_wall_change(),
        KeyCode::Char('s') => {
            game_state.set_player_spawn_to_current_position(game_state.player.x, game_state.player.y);
        }
        KeyCode::Char('k') => {
            game_state.is_map_kind_selection_active = !game_state.is_map_kind_selection_active;
            if game_state.is_map_kind_selection_active {
                game_state.message = "Map Kind Selection: Use Up/Down to cycle, Enter to confirm, Esc to cancel.".to_string();
            } else {
                game_state.message = "Map Kind Selection cancelled.".to_string();
            }
            game_state.show_message = true;
            game_state.message_animation_start_time = Instant::now();
            game_state.animated_message_content.clear();
        }
        
        KeyCode::F(3) => {
            game_state.is_creating_map = true;
            game_state.is_text_input_active = true;
            game_state.text_input_buffer.clear();
            game_state.message = "Enter new map name (e.g., map_0_1):".to_string();
            game_state.show_message = true;
            game_state.message_animation_start_time = Instant::now();
            game_state.animated_message_content.clear();
        }
        KeyCode::Enter => {
            if game_state.is_drawing_select_box {
                if let Some((start_x, start_y)) = game_state.select_box_start_coords {
                    let (end_x, end_y) = (game_state.player.x, game_state.player.y);
                    let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                    if let Some(map_to_modify) = game_state.loaded_maps.get_mut(&current_map_key) {
                        let new_id = map_to_modify.select_object_boxes.iter().map(|b| b.id).max().unwrap_or(0) + 1;
                        let new_select_box = crate::game::map::SelectObjectBox {
                            id: new_id,
                            x1: start_x as u32,
                            y1: start_y as u32,
                            x2: end_x as u32,
                            y2: end_y as u32,
                            messages: Vec::new(),
                            events: Vec::new(),
                        };
                        game_state.pending_select_box = Some(new_select_box);
                        game_state.is_text_input_active = true;
                        game_state.text_input_buffer.clear();
                        game_state.message = "Enter messages for the new object. Press Enter to add a message, Esc to finish.".to_string();
                        game_state.show_message = true;
                        game_state.message_animation_start_time = Instant::now();
                        game_state.animated_message_content.clear();
                    }
                }
                game_state.is_drawing_select_box = false;
                game_state.select_box_start_coords = None;
            }
            
        }
        _ => return false,
    }
    true
}
