use crate::debug;
use crate::game::state::{GameState, TeleportCreationState};
use ansi_to_tui::IntoText;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

const PLAYER_INTERACTION_BOX_WIDTH: u16 = 30;
const PLAYER_INTERACTION_BOX_HEIGHT: u16 = 20;

pub fn draw(frame: &mut Frame, game_state: &mut GameState) {
    let size = frame.area();
    frame.render_widget(Block::default().bg(Color::Black), size);

    let player_x_on_screen = game_state.player.x.saturating_sub(game_state.camera_x);
    let player_y_on_screen = game_state.player.y.saturating_sub(game_state.camera_y);
    let combined_map_text = game_state.get_combined_map_text(size);

    let map_paragraph = Paragraph::new(combined_map_text)
        .scroll((game_state.camera_y, game_state.camera_x))
        .style(Style::default().bg(Color::Black));

    let map_block = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(map_block, size);
    frame.render_widget(map_paragraph, size);

    let current_map_key = (game_state.current_map_row, game_state.current_map_col);
    if let Some(current_map) = game_state.loaded_maps.get(&current_map_key) {
        if let crate::game::map::MapKind::Objects = current_map.kind {
            let (_player_sprite_content, player_sprite_width, player_sprite_height) =
                game_state.player.get_sprite_content();

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

            let select_box_x_on_screen = select_box_x.saturating_sub(game_state.camera_x);
            let select_box_y_on_screen = select_box_y.saturating_sub(game_state.camera_y);

            let draw_rect = ratatui::layout::Rect::new(
                select_box_x_on_screen,
                select_box_y_on_screen,
                PLAYER_INTERACTION_BOX_WIDTH,
                PLAYER_INTERACTION_BOX_HEIGHT,
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
    let player_paragraph = Paragraph::new(player_sprite_content);

    let player_draw_rect = ratatui::layout::Rect::new(
        player_x_on_screen,
        player_y_on_screen,
        player_sprite_width,
        player_sprite_height,
    );

    let clamped_player_rect = player_draw_rect.intersection(size);
    if !clamped_player_rect.is_empty() {
        frame.render_widget(player_paragraph, clamped_player_rect);
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
            format!("{} {}", exiting_text_lines[0], dot_line_0),
            format!("{} {}", exiting_text_lines[1], dot_line_1),
            format!("{} {}", exiting_text_lines[2], dot_line_2),
        ];
        let combined_text = combined_lines.join("\n");

        let line1_width = combined_lines[0].chars().count() as u16;
        let line2_width = combined_lines[1].chars().count() as u16;
        let line3_width = combined_lines[2].chars().count() as u16;
        let text_width = line1_width.max(line2_width).max(line3_width);
        let text_height = 3;

        let paragraph = Paragraph::new(combined_text)
            .style(Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0)));

        let area = ratatui::layout::Rect::new(0, 0, text_width, text_height);
        frame.render_widget(paragraph, area);
    }
}
