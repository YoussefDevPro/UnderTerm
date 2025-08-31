use crate::debug;
use crate::game::state::{GameState, TeleportCreationState};
use ansi_to_tui::IntoText;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Frame,
};



pub fn draw(frame: &mut Frame, game_state: &mut GameState) {
    let size = frame.area();
    frame.render_widget(Block::default().bg(Color::Black), size);

    let player_x_on_screen = game_state.player.x.saturating_sub(game_state.camera_x);
    let player_y_on_screen = game_state.player.y.saturating_sub(game_state.camera_y);
    let combined_map_text = game_state.get_combined_map_text(size, game_state.deltarune.level);

    let map_paragraph = Paragraph::new(combined_map_text)
        .scroll((game_state.camera_y, game_state.camera_x))
        .style(Style::default().bg(Color::Black));

    let map_block = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(map_block, size);
    frame.render_widget(map_paragraph, size);

    let current_map_key = (game_state.current_map_row, game_state.current_map_col);
    if let Some(current_map) = game_state.loaded_maps.get(&current_map_key) {
        if let crate::game::map::MapKind::Objects = current_map.kind {
            let interaction_rect = game_state.player.get_interaction_rect();

            let select_box_x_on_screen = interaction_rect.x.saturating_sub(game_state.camera_x);
            let select_box_y_on_screen = interaction_rect.y.saturating_sub(game_state.camera_y);

            let draw_rect = ratatui::layout::Rect::new(
                select_box_x_on_screen,
                select_box_y_on_screen,
                interaction_rect.width,
                interaction_rect.height,
            );

            let clamped_rect = draw_rect.intersection(size);
            if !clamped_rect.is_empty() {
                let select_box_paragraph = Paragraph::new("").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green)),
                );
                frame.render_widget(select_box_paragraph, clamped_rect);
            }
        }
    }

    let (player_sprite_content, player_sprite_width, player_sprite_height) =
        if game_state.debug_mode {
            ("P".to_string().as_bytes().into_text().unwrap(), 1, 1)
        } else {
            game_state.player.get_sprite_content()
        };

    let darkened_player_sprite =
        game_state.darken_text(player_sprite_content, game_state.deltarune.level);
    let player_paragraph = Paragraph::new(darkened_player_sprite);

    let player_rect =
        ratatui::layout::Rect::new(0, 0, player_sprite_width, player_sprite_height);
    let mut player_buffer = Buffer::empty(player_rect);
    player_paragraph.render(player_rect, &mut player_buffer);

    for y in 0..player_sprite_height {
        for x in 0..player_sprite_width {
            let cell = &player_buffer[(x, y)];
            let is_space = cell.symbol() == " ";
            let has_bg = cell.bg != Color::Reset;

            if !is_space || has_bg {
                let screen_x = player_x_on_screen.saturating_add(x);
                let screen_y = player_y_on_screen.saturating_add(y);
                if screen_x < size.width && screen_y < size.height {
                    let frame_cell = &mut frame.buffer_mut()[(screen_x, screen_y)];
                    frame_cell.set_symbol(cell.symbol());
                    frame_cell.set_fg(cell.fg);
                    frame_cell.modifier = cell.modifier;
                    if has_bg {
                        frame_cell.set_bg(cell.bg);
                    }
                }
            }
        }
    }

    if game_state.debug_mode {
        debug::draw::draw_debug_info(frame, game_state);
    }

    if game_state.is_drawing_select_box {
        if let Some((start_x, start_y)) = game_state.select_box_start_coords {
            let current_x = game_state.player.x;
            let current_y = game_state.player.y;

            let min_x = start_x.min(current_x);
            let max_x = start_x.max(current_x);
            let min_y = start_y.min(current_y);
            let max_y = start_y.max(current_y);

            let width = max_x.saturating_sub(min_x).saturating_add(1);
            let height = max_y.saturating_sub(min_y).saturating_add(1);

            let draw_x = min_x.saturating_sub(game_state.camera_x);
            let draw_y = min_y.saturating_sub(game_state.camera_y);

            let draw_rect = ratatui::layout::Rect::new(draw_x, draw_y, width, height);
            let drawing_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            frame.render_widget(drawing_block, draw_rect);
        }
    }

    if game_state.debug_mode {
        let current_map_key = (game_state.current_map_row, game_state.current_map_col);
        if let Some(current_map) = game_state.loaded_maps.get(&current_map_key) {
            for &(x, y) in &current_map.walls {
                let draw_x = (x as u16).saturating_sub(game_state.camera_x);
                let draw_y = (y as u16).saturating_sub(game_state.camera_y);

                if draw_x < size.width && draw_y < size.height {
                    let wall_paragraph = Paragraph::new("W").style(Style::default().fg(Color::Red));
                    let wall_rect = ratatui::layout::Rect::new(draw_x, draw_y, 1, 1);
                    frame.render_widget(wall_paragraph, wall_rect);
                }
            }
        }
    }

    if game_state.show_message {
        let message_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .border_style(Style::default().fg(Color::White).bg(Color::White))
            .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
            .padding(ratatui::widgets::Padding::new(8, 8, 1, 1))
            .title("Message");

        let message_paragraph = Paragraph::new(game_state.animated_message_content.clone())
            .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
            .block(message_block);

        let message_height = 10;
        let bottom_margin = 5;
        let horizontal_margin = 40;

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

        frame.render_widget(Clear, message_area);
        frame.render_widget(message_paragraph, message_area);
    }

    if game_state.is_text_input_active {
        let title = if game_state.is_creating_map {
            "Enter New Map Name"
        } else if game_state.teleport_creation_state == TeleportCreationState::EnteringMapName {
            "Enter Target Map Name"
        } else {
            "Enter Message"
        };
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .border_style(Style::default().fg(Color::White).bg(Color::White))
            .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
            .padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
            .title(title);

        let input_paragraph = Paragraph::new(game_state.text_input_buffer.clone())
            .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)))
            .block(input_block);

        let input_area_height = 3;
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

        frame.render_widget(Clear, input_area);
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

        let input_area_height = 3;
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

        frame.render_widget(Clear, input_area);
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

        let input_area_height = 5;
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

        frame.render_widget(Clear, input_area);
        frame.render_widget(input_paragraph, input_area);
    }

    if game_state.esc_press_start_time.is_some() {
        let exiting_text_lines = vec![
            "╔═╗═╗ ╦╦╔╦╗╦╔╗╔╔═╗",
            "║╣ ╔╩╦╝║ ║ ║║║║║ ╦",
            "╚═╝╩ ╚═╩ ╩ ╩╝╚╝╚═╝",
        ];

        let num_dots = game_state.esc_hold_dots as usize;
        let dot_width = 4;
        let total_dots_width = num_dots * dot_width;

        let dot_line_0 = " ".repeat(total_dots_width);
        let dot_line_1 = "▄██▄ ".repeat(num_dots);
        let dot_line_2 = "▀██▀ ".repeat(num_dots);

        let combined_lines = vec![
            format!("{}\n {}", exiting_text_lines[0], dot_line_0),
            format!("{}\n {}", exiting_text_lines[1], dot_line_1),
            format!("{}\n {}", exiting_text_lines[2], dot_line_2),
        ];
        let combined_text = combined_lines.join("\n");

        let line1_width = combined_lines[0].chars().count() as u16;
        let line2_width = combined_lines[1].chars().count() as u16;
        let line3_width = combined_lines[2].chars().count() as u16;
        let text_width = line1_width.max(line2_width).max(line3_width);
        let text_height = 3;

        let paragraph = Paragraph::new(combined_text)
            .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)));

        let x = (size.width.saturating_sub(text_width)) / 2;
        let y = (size.height.saturating_sub(text_height)) / 2;
        let area = ratatui::layout::Rect::new(x, y, text_width, text_height);
        frame.render_widget(paragraph, area);
    }

    
}