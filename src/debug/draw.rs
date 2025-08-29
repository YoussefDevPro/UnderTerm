use crate::game::config::ANIMATION_FRAME_DURATION;
use crate::game::state::{GameState, TeleportCreationState};

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

const TOP_INTERACTION_BOX_HEIGHT: u16 = 10;
const BOTTOM_INTERACTION_BOX_HEIGHT: u16 = 3;
const LEFT_INTERACTION_BOX_WIDTH: u16 = 5;
const RIGHT_INTERACTION_BOX_WIDTH: u16 = 5;
const COLLISION_BOX_WIDTH: u16 = 21;
const COLLISION_BOX_HEIGHT: u16 = 4;

pub fn draw_debug_info(frame: &mut Frame, game_state: &GameState) {
    let size = frame.area();
    let current_map_key = (game_state.current_map_row, game_state.current_map_col);

    if let Some(current_map) = game_state.loaded_maps.get(&current_map_key) {
        if let crate::game::map::MapKind::Walls = current_map.kind {
            for &(wx, wy) in &current_map.walls {
                let wall_x_on_screen = wx.saturating_sub(game_state.camera_x as u32);
                let wall_y_on_screen = wy.saturating_sub(game_state.camera_y as u32);

                if wall_x_on_screen < size.width as u32 && wall_y_on_screen < size.height as u32 {
                    let draw_rect =
                        Rect::new(wall_x_on_screen as u16, wall_y_on_screen as u16, 1, 1);
                    let clamped_rect = draw_rect.intersection(size);
                    if !clamped_rect.is_empty() {
                        let wall_paragraph = Paragraph::new("W")
                            .style(Style::default().fg(Color::Red).bg(Color::Black));
                        frame.render_widget(wall_paragraph, clamped_rect);
                    }
                }
            }
        }
    }

    // spawn point
    let spawn_x = game_state.player.x;
    let spawn_y = game_state.player.y;

    let spawn_x_on_screen = spawn_x.saturating_sub(game_state.camera_x);
    let spawn_y_on_screen = spawn_y.saturating_sub(game_state.camera_y);

    if spawn_x_on_screen < size.width && spawn_y_on_screen < size.height {
        let draw_rect = Rect::new(spawn_x_on_screen, spawn_y_on_screen, 1, 1);
        let clamped_rect = draw_rect.intersection(size);
        if !clamped_rect.is_empty() {
            let spawn_paragraph =
                Paragraph::new("S").style(Style::default().fg(Color::Green).bg(Color::Black));
            frame.render_widget(spawn_paragraph, clamped_rect);
        }
    }

    // Draw player collision box
    let (_, _, player_sprite_height) = game_state.player.get_sprite_content();
    let collision_box_start_x = game_state.player.x;
    let collision_box_start_y = game_state
        .player
        .y
        .saturating_add(player_sprite_height)
        .saturating_sub(COLLISION_BOX_HEIGHT);

    let collision_box_x_on_screen = collision_box_start_x.saturating_sub(game_state.camera_x);
    let collision_box_y_on_screen = collision_box_start_y.saturating_sub(game_state.camera_y);

    let draw_rect = Rect::new(
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
                .border_style(Style::default().fg(Color::Blue)),
        );
        frame.render_widget(collision_box_paragraph, clamped_rect);
    }

    let collision_box = Rect::new(
        collision_box_start_x,
        collision_box_start_y,
        COLLISION_BOX_WIDTH,
        COLLISION_BOX_HEIGHT,
    );

    let top_box = Rect::new(
        collision_box.x,
        collision_box.y.saturating_sub(TOP_INTERACTION_BOX_HEIGHT),
        collision_box.width,
        TOP_INTERACTION_BOX_HEIGHT,
    );
    let bottom_box = Rect::new(
        collision_box.x,
        collision_box.y + collision_box.height,
        collision_box.width,
        BOTTOM_INTERACTION_BOX_HEIGHT,
    );
    let left_box = Rect::new(
        collision_box.x.saturating_sub(LEFT_INTERACTION_BOX_WIDTH),
        collision_box.y,
        LEFT_INTERACTION_BOX_WIDTH,
        collision_box.height,
    );
    let right_box = Rect::new(
        collision_box.x + collision_box.width,
        collision_box.y,
        RIGHT_INTERACTION_BOX_WIDTH,
        collision_box.height,
    );
    let interaction_boxes = [top_box, bottom_box, left_box, right_box];

    for box_rect in &interaction_boxes {
        let box_on_screen_x = box_rect.x.saturating_sub(game_state.camera_x);
        let box_on_screen_y = box_rect.y.saturating_sub(game_state.camera_y);

        let draw_rect = Rect::new(
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

    // Render SelectObjectBoxes
    if let Some(current_map) = game_state.loaded_maps.get(&current_map_key) {
        for select_box in &current_map.select_object_boxes {
            let select_box_rect = select_box.to_rect();

            let draw_x = select_box_rect.x.saturating_sub(game_state.camera_x);
            let draw_y = select_box_rect.y.saturating_sub(game_state.camera_y);

            let draw_rect = Rect::new(
                draw_x,
                draw_y,
                select_box_rect.width,
                select_box_rect.height,
            );
            let clamped_rect = draw_rect.intersection(size);
            if !clamped_rect.is_empty() {
                let select_box_paragraph = Paragraph::new("I").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                );
                frame.render_widget(select_box_paragraph, clamped_rect);
            }
        }
    }

    // Draw pending select box or teleport line
    if (game_state.is_drawing_select_box
        || game_state.teleport_creation_state == TeleportCreationState::DrawingBox)
        && game_state.select_box_start_coords.is_some()
    {
        if let Some((start_x, start_y)) = game_state.select_box_start_coords {
            let (end_x, end_y) = (game_state.player.x, game_state.player.y);

            let rect_x = start_x.min(end_x);
            let rect_y = start_y.min(end_y);
            let rect_width = start_x.max(end_x).saturating_sub(rect_x).saturating_add(1);
            let rect_height = start_y.max(end_y).saturating_sub(rect_y).saturating_add(1);

            let draw_x = rect_x.saturating_sub(game_state.camera_x);
            let draw_y = rect_y.saturating_sub(game_state.camera_y);

            let draw_rect = Rect::new(draw_x, draw_y, rect_width, rect_height);
            let clamped_rect = draw_rect.intersection(size);

            if !clamped_rect.is_empty() {
                let color =
                    if game_state.teleport_creation_state == TeleportCreationState::DrawingBox {
                        Color::Magenta
                    } else {
                        Color::Green
                    };
                let pending_box_paragraph = Paragraph::new("").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(color)),
                );
                frame.render_widget(pending_box_paragraph, clamped_rect);
            }
        }
    }

    // Draw visual X, Y selector when selecting coordinates
    if game_state.teleport_creation_state == TeleportCreationState::SelectingCoordinates {
        let draw_x = game_state.player.x.saturating_sub(game_state.camera_x);
        let draw_y = game_state.player.y.saturating_sub(game_state.camera_y);

        let draw_rect = Rect::new(draw_x, draw_y, 1, 1); // A 1x1 box at player's position
        let clamped_rect = draw_rect.intersection(size);

        if !clamped_rect.is_empty() {
            let selector_paragraph = Paragraph::new("X").block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            );
            frame.render_widget(selector_paragraph, clamped_rect);
        }
    }

    if game_state.show_debug_panel {
        draw_debug_panel(frame, game_state);
    }
}

fn draw_debug_panel(frame: &mut Frame, game_state: &GameState) {
    let size = frame.area();
    let player_x_on_screen = game_state.player.x.saturating_sub(game_state.camera_x);
    let player_y_on_screen = game_state.player.y.saturating_sub(game_state.camera_y);
    let current_map_key = (game_state.current_map_row, game_state.current_map_col);
    let (map_width, map_height, map_kind) = game_state
        .loaded_maps
        .get(&current_map_key)
        .map(|m| (m.width, m.height, format!("{:?}", m.kind)))
        .unwrap_or((0, 0, "N/A".to_string()));

    let debug_text = vec![
        format!("Player: ({}, {})", game_state.player.x, game_state.player.y),
        format!("Direction: {:?}", game_state.player.direction),
        format!("Animation Frame: {}", game_state.player.animation_frame),
        format!("Is Walking: {}", game_state.player.is_walking),
        format!("Camera: ({}, {})", game_state.camera_x, game_state.camera_y),
        format!("Map: ({}, {})", map_width, map_height),
        format!(
            "Screen Player Pos: ({}, {})",
            player_x_on_screen, player_y_on_screen
        ),
        format!("Debug Mode: {}", game_state.debug_mode),
        format!(
            "Current Map: {} ({}, {})",
            game_state.current_map_name, game_state.current_map_row, game_state.current_map_col
        ),
        format!("Anim Frame Duration: {:?}", ANIMATION_FRAME_DURATION),
        format!("Map Kind: {}", map_kind),
    ];

    let debug_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Thick)
        .border_style(Style::default().fg(Color::White).bg(Color::White))
        .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
        .padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
        .title("Debug Panel");

    let debug_paragraph = Paragraph::new(debug_text.join("\n"))
        .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
        .block(debug_block);

    let max_line_length = debug_text.iter().map(|s| s.len() as u16).max().unwrap_or(0);
    let debug_panel_width = max_line_length + 4;
    let debug_panel_height = debug_text.len() as u16 + 4;

    let margin = 2;
    let x = size.width.saturating_sub(debug_panel_width + margin);
    let y = margin;

    let debug_panel_rect = Rect::new(x, y, debug_panel_width, debug_panel_height);
    frame.render_widget(Clear, debug_panel_rect);
    frame.render_widget(debug_paragraph, debug_panel_rect);
}
