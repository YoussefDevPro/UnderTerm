use crate::debug;
use crate::game::state::{GameState, TeleportCreationState};
use crate::game::utils::wrap_text_to_width;
use crate::load_sprite_asset_str;
use ansi_to_tui::IntoText;
use figlet_rs::FIGfont;
use ratatui::prelude::Text;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Frame,
};

fn convert_and_fix_t(font: &FIGfont, text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    if let Some(fig_text) = font.convert(text) {
        fig_text.to_string()
    } else {
        "FIGLET CONVERSION FAILED".to_string()
    }
}

fn draw_enemy_ansi(frame: &mut Frame) {
    let size = frame.area();
    let background = Block::default().bg(Color::Rgb(0, 0, 0));
    frame.render_widget(background, size);

    let ansi_content = include_str!("../assets/sprites/enemy/not_a_placeholder/battle_3.ans");
    let enemy_text = ansi_content.as_bytes().into_text().unwrap();
    let enemy_height = enemy_text.lines.len() as u16;
    let mut enemy_width = 0;
    for line in enemy_text.lines.iter() {
        let line_width = line.width() as u16;
        if line_width > enemy_width {
            enemy_width = line_width;
        }
    }

    let enemy_draw_width = enemy_width.min(size.width);
    let enemy_draw_height = enemy_height.min(size.height);

    let enemy_x = (size.x + (size.width.saturating_sub(enemy_draw_width)) / 2) as i32;
    let enemy_y = size.y + (size.height.saturating_sub(enemy_draw_height)) / 2;

    let enemy_area =
        ratatui::layout::Rect::new(enemy_x as u16, enemy_y, enemy_draw_width, enemy_draw_height);
    let enemy_paragraph = Paragraph::new(enemy_text);
    frame.render_widget(enemy_paragraph, enemy_area);
}

fn draw_dialogue(frame: &mut Frame, game_state: &mut GameState) {
    if let Some(dialogue) = game_state.dialogue_manager.current_dialogue() {
        let size = frame.area();
        frame.render_widget(Block::default().bg(Color::Rgb(0, 0, 0)), size);

        let enemy_ansi = load_sprite_asset_str!(dialogue.enemy_ansi_path.as_str());
        let enemy_text = enemy_ansi.as_bytes().into_text().unwrap();
        let enemy_height = enemy_text.lines.len() as u16;
        let mut enemy_width = 0;
        for line in enemy_text.lines.iter() {
            let line_width = line.width() as u16;
            if line_width > enemy_width {
                enemy_width = line_width;
            }
        }

        let enemy_draw_width = enemy_width.min(size.width);
        let enemy_draw_height = enemy_height.min(size.height);

        const MIN_DIALOGUE_BOX_HEIGHT: u16 = 8;
        let dialogue_box_height = (size.height / 3).max(MIN_DIALOGUE_BOX_HEIGHT);

        let enemy_x = (size.width as i32 - enemy_draw_width as i32) / 2;
        let enemy_y = (size.height.saturating_sub(dialogue_box_height + 5))
            .saturating_sub(enemy_draw_height + 1);

        let enemy_area = ratatui::layout::Rect::new(
            enemy_x.max(0) as u16,
            enemy_y.max(0),
            enemy_draw_width,
            enemy_draw_height,
        );
        frame.render_widget(Paragraph::new(enemy_text.clone()), enemy_area);

        let dialogue_box_area = ratatui::layout::Rect::new(
            size.x + 30,
            size.height.saturating_sub(dialogue_box_height + 4),
            size.width - 60,
            dialogue_box_height,
        );
        let dialogue_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .border_style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            )
            .title("Dialogue");
        frame.render_widget(dialogue_block.clone(), dialogue_box_area);

        let face_width = (dialogue_box_area.width as f32 * 0.20) as u16;
        let face_height = dialogue_box_area.height.saturating_sub(2);
        let face_area = ratatui::layout::Rect::new(
            dialogue_box_area.x + 1,
            dialogue_box_area.y + 1,
            face_width,
            face_height,
        );

        let text_x = dialogue_box_area.x + face_width + 2;
        let text_width = dialogue_box_area
            .width
            .saturating_sub(face_width)
            .saturating_sub(3);
        let text_y = dialogue_box_area.y + 3;
        let text_height = dialogue_box_area.height.saturating_sub(4);
        let text_area = ratatui::layout::Rect::new(text_x, text_y, text_width, text_height);

        let face_ansi = load_sprite_asset_str!(dialogue.face_ansi_path.as_str());
        let face_text = face_ansi.as_bytes().into_text().unwrap();
        frame.render_widget(
            Paragraph::new(face_text).style(Style::default().add_modifier(Modifier::BOLD)),
            face_area,
        );

        let font = FIGfont::from_file("assets/fonts/Calvin S.flf").unwrap();

        let mut chunks = Vec::new();
        let visible_text = dialogue
            .text
            .chars()
            .take(game_state.dialogue_manager.visible_text_len)
            .collect::<String>();
        let mut remaining_text = visible_text.as_str();

        while !remaining_text.is_empty() {
            let mut best_split = remaining_text.len();
            let mut current_width = 0;
            let mut last_space = 0;

            for (i, c) in remaining_text.char_indices() {
                if c.is_whitespace() {
                    last_space = i;
                }
                current_width += 3; // estimated width
                if current_width > 130 {
                    if last_space > 0 {
                        best_split = last_space;
                    } else {
                        best_split = i;
                    }
                    break;
                }
            }
            let (chunk, rest) = remaining_text.split_at(best_split);
            chunks.push(chunk.to_string());
            remaining_text = rest;
        }

        let num_chunks = chunks.len();
        let constraints: Vec<Constraint> = (0..num_chunks).map(|_| Constraint::Length(3)).collect();
        let text_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .spacing(1)
            .split(text_area);

        for (i, chunk) in chunks.iter().enumerate() {
            if i < text_chunks.len() {
                let wrapped_text = wrap_text_to_width(chunk, text_chunks[i].width);
                let fig_text_str = convert_and_fix_t(&font, &wrapped_text);
                let text_paragraph = Paragraph::new(fig_text_str)
                    .wrap(ratatui::widgets::Wrap { trim: false })
                    .style(Style::default().add_modifier(Modifier::BOLD));
                frame.render_widget(text_paragraph, text_chunks[i]);
            }
        }
    }
}

fn draw_thank_you_screen(frame: &mut Frame, game_state: &mut GameState) {
    let size = frame.area();
    frame.render_widget(Block::default().bg(Color::Rgb(0, 0, 0)), size);

    let ansi_content = load_sprite_asset_str!("assets/sprites/ME/idle/insanly_dead.ans");
    let ansi_text = game_state.darken_text(
        ansi_content.as_bytes().into_text().unwrap(),
        game_state.deltarune.level,
    );
    let ansi_height = ansi_text.lines.len() as u16;
    let mut ansi_width = 0;
    for line in ansi_text.lines.iter() {
        let line_width = line.width() as u16;
        if line_width > ansi_width {
            ansi_width = line_width;
        }
    }

    let ansi_draw_width = ansi_width.min(size.width);
    let ansi_draw_height = ansi_height.min(size.height);

    let ansi_x = (size.width.saturating_sub(ansi_draw_width)) / 2 + 20;
    let ansi_y = (size.height.saturating_sub(ansi_draw_height)) / 2 + 10;

    let ansi_area = ratatui::layout::Rect::new(ansi_x, ansi_y, ansi_draw_width, ansi_draw_height);
    frame.render_widget(Paragraph::new(ansi_text), ansi_area);

    let font = FIGfont::from_file("assets/fonts/3d.flf").unwrap();
    let thank_you_text = "Thanks for playing ?";
    let fig_text = game_state.darken_text(
        Text::raw(convert_and_fix_t(&font, thank_you_text)),
        game_state.deltarune.level,
    );

    let fig_text_lines: Vec<String> = fig_text.lines.iter().map(|l| l.to_string()).collect();
    let fig_text_height = fig_text_lines.len() as u16;
    let mut fig_text_width = 0;
    for line in fig_text_lines {
        let line_width = line.len() as u16;
        if line_width > fig_text_width {
            fig_text_width = line_width;
        }
    }

    let text_x = (size.width.saturating_sub(fig_text_width)) / 2 + 10;
    let text_y = size.height / 4;

    let text_area = ratatui::layout::Rect::new(text_x, text_y, fig_text_width, fig_text_height);
    frame.render_widget(Paragraph::new(fig_text), text_area);
}

pub fn draw(frame: &mut Frame, game_state: &mut GameState) {
    let size = frame.area();

    if game_state.dialogue_active {
        draw_dialogue(frame, game_state);
        return;
    }

    if game_state.teleport_state == crate::game::state::TeleportState::ThankYouScreen {
        draw_thank_you_screen(frame, game_state);
        return;
    }

    // If dialogue is not active and screen is black, draw the thank you screen
    if !game_state.dialogue_active && game_state.deltarune.level >= 99 {
        draw_thank_you_screen(frame, game_state);
        return;
    }

    if game_state.show_enemy_ansi {
        draw_enemy_ansi(frame);
        return;
    }

    frame.render_widget(Block::default().bg(Color::Rgb(0, 0, 0)), size);

    if game_state.is_flickering && game_state.show_flicker_black_screen {
        frame.render_widget(Block::default().bg(Color::Rgb(0, 0, 0)), size);

        let ascii_art = "  ▄ ▄  \n■██▄██■\n ▀███▀ \n   ▀   ";
        let ascii_text = Text::styled(ascii_art, Style::default().fg(Color::Rgb(255, 0, 0)));

        let ascii_width = 7;
        let ascii_height = 4;

        let (_, player_sprite_width, player_sprite_height) = game_state.player.get_sprite_content();

        let player_x_on_screen = (game_state.player.x as u16).saturating_sub(game_state.camera_x);
        let player_y_on_screen = (game_state.player.y as u16).saturating_sub(game_state.camera_y);

        let art_x = (player_x_on_screen + player_sprite_width / 2).saturating_sub(ascii_width / 2);
        let art_y =
            (player_y_on_screen + player_sprite_height / 2).saturating_sub(ascii_height / 2);

        let art_rect = ratatui::layout::Rect::new(art_x, art_y, ascii_width, ascii_height);

        frame.render_widget(Paragraph::new(ascii_text), art_rect);
        return;
    }

    let combined_map_text = game_state.get_combined_map_text(size, game_state.deltarune.level);

    let map_paragraph = Paragraph::new(combined_map_text)
        .scroll((game_state.camera_y, game_state.camera_x))
        .style(Style::default().bg(Color::Rgb(0, 0, 0)));

    let map_block = Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0)));
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
                        .border_style(Style::default().fg(Color::Rgb(0, 255, 0))),
                );
                frame.render_widget(select_box_paragraph, clamped_rect);
            }
        }
    }

    let mut drawable_elements: Vec<(i32, u8, Text<'static>, i32, i32, u16, u16)> = Vec::new(); // (y_sort_key, z_index, ansi_content, x, y, width, height) ദ്ദി/ᐠ｡‸｡ᐟ\

    let (player_sprite_content, player_sprite_width, player_sprite_height) =
        game_state.player.get_sprite_content();
    let player_x_on_screen =
        (game_state.player.x as i32).saturating_sub(game_state.camera_x as i32);
    let player_y_on_screen =
        (game_state.player.y as i32).saturating_sub(game_state.camera_y as i32);
    drawable_elements.push((
        player_y_on_screen + player_sprite_height as i32,
        1,
        player_sprite_content,
        player_x_on_screen,
        player_y_on_screen,
        player_sprite_width,
        player_sprite_height,
    ));

    if game_state.is_placing_sprite {
        if let Some(pending_sprite) = &game_state.pending_placed_sprite {
            let sprite_x_on_screen = pending_sprite.x as i32 - game_state.camera_x as i32;
            let sprite_y_on_screen = pending_sprite.y as i32 - game_state.camera_y as i32;
            drawable_elements.push((
                sprite_y_on_screen + pending_sprite.height as i32,
                0,
                pending_sprite.ansi_content.as_bytes().into_text().unwrap(),
                sprite_x_on_screen,
                sprite_y_on_screen,
                pending_sprite.width as u16,
                pending_sprite.height as u16,
            ));
        }
    }

    if let Some(current_map) = game_state.loaded_maps.get(&current_map_key) {
        for placed_sprite in &current_map.placed_sprites {
            let sprite_x_on_screen = placed_sprite.x as i32 - game_state.camera_x as i32;
            let sprite_y_on_screen = placed_sprite.y as i32 - game_state.camera_y as i32;
            drawable_elements.push((
                sprite_y_on_screen + placed_sprite.height as i32,
                0,
                placed_sprite.ansi_content.as_bytes().into_text().unwrap(),
                sprite_x_on_screen,
                sprite_y_on_screen,
                placed_sprite.width as u16,
                placed_sprite.height as u16,
            ));
        }
    }

    drawable_elements.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    for (_, _, text, abs_x, abs_y, width, height) in drawable_elements {
        let sprite_x_relative_to_camera = abs_x;
        let sprite_y_relative_to_camera = abs_y;
        let draw_x = sprite_x_relative_to_camera.max(0) as u16;
        let draw_y = sprite_y_relative_to_camera.max(0) as u16;
        let offset_x = if sprite_x_relative_to_camera < 0 {
            -sprite_x_relative_to_camera
        } else {
            0
        };
        let offset_y = if sprite_y_relative_to_camera < 0 {
            -sprite_y_relative_to_camera
        } else {
            0
        };
        let actual_width = (width as i32 - offset_x).max(0) as u16;
        let actual_height = (height as i32 - offset_y).max(0) as u16;

        let darkened_sprite = game_state.darken_text(text, game_state.deltarune.level);
        let paragraph = Paragraph::new(darkened_sprite).scroll((offset_y as u16, offset_x as u16));

        let potential_render_rect =
            ratatui::layout::Rect::new(draw_x, draw_y, actual_width, actual_height);

        let final_render_rect = potential_render_rect.intersection(frame.area());

        if final_render_rect.is_empty() {
            continue;
        }

        let mut temp_buffer = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(
            0,
            0,
            final_render_rect.width,
            final_render_rect.height,
        ));
        paragraph.render(
            ratatui::layout::Rect::new(0, 0, final_render_rect.width, final_render_rect.height),
            &mut temp_buffer,
        );

        for y_temp in 0..final_render_rect.height {
            for x_temp in 0..final_render_rect.width {
                let cell = &temp_buffer[(x_temp, y_temp)];

                let screen_x = final_render_rect.x + x_temp;
                let screen_y = final_render_rect.y + y_temp;

                if cell.symbol() == " " && cell.bg == ratatui::style::Color::Reset {
                    continue;
                }

                let frame_cell = &mut frame.buffer_mut()[(screen_x, screen_y)];

                frame_cell.set_symbol(cell.symbol());
                frame_cell.set_fg(cell.fg);
                if cell.bg != ratatui::style::Color::Reset {
                    frame_cell.set_bg(cell.bg);
                }
                frame_cell.modifier = cell.modifier;
            }
        }
    }

    if game_state.debug_mode {
        debug::draw::draw_debug_info(frame, game_state);
    }

    if game_state.is_drawing_select_box {
        if let Some((start_x, start_y)) = game_state.select_box_start_coords {
            let current_x = game_state.player.x as u16;
            let current_y = game_state.player.y as u16;

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
                .border_style(Style::default().fg(Color::Rgb(255, 255, 0)));
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
                    let wall_paragraph =
                        Paragraph::new("W").style(Style::default().fg(Color::Rgb(255, 0, 0)));
                    let wall_rect = ratatui::layout::Rect::new(draw_x, draw_y, 1, 1);
                    frame.render_widget(wall_paragraph, wall_rect);
                }
            }
        }
    }

    if game_state.show_message {
        if game_state.animated_message_content.is_empty() {
            return; // Don't draw if message content is empty
        }
        let message_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .border_style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            )
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(0, 0, 0)),
            )
            .padding(ratatui::widgets::Padding::new(8, 8, 1, 1))
            .title("Message");

        let font = FIGfont::from_file("assets/fonts/Calvin S.flf").unwrap();
        let ascii_art = convert_and_fix_t(&font, &game_state.animated_message_content);
        let message_paragraph = Paragraph::new(ascii_art)
            .wrap(ratatui::widgets::Wrap { trim: false })
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(0, 0, 0))
                    .add_modifier(Modifier::BOLD),
            )
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
            .border_style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            )
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(0, 0, 0)),
            )
            .padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
            .title(title);

        let input_paragraph = Paragraph::new(game_state.text_input_buffer.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(0, 0, 0)),
            )
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
            .border_style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            )
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(0, 0, 0)),
            )
            .padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
            .title("Enter Event");

        let input_paragraph = Paragraph::new(game_state.text_input_buffer.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(0, 0, 0)),
            )
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
            .border_style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(255, 255, 255)),
            )
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(0, 0, 0)),
            )
            .padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
            .title("Select Map Kind");

        let current_map_key = (game_state.current_map_row, game_state.current_map_col);
        let current_map_kind = game_state
            .loaded_maps
            .get(&current_map_key)
            .map(|m| format!("{:?}", m.kind))
            .unwrap_or_else(|| "Unknown or deltarune".to_string());

        let input_paragraph = Paragraph::new(format!("Current: {}", current_map_kind))
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(0, 0, 0)),
            )
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
            format!("{}\n{}", exiting_text_lines[0], dot_line_0),
            format!("{}\n{}", exiting_text_lines[1], dot_line_1),
            format!("{}\n{}", exiting_text_lines[2], dot_line_2),
        ];
        let combined_text = combined_lines.join("\n");

        let line1_width = combined_lines[0].chars().count() as u16;
        let line2_width = combined_lines[1].chars().count() as u16;
        let line3_width = combined_lines[2].chars().count() as u16;
        let text_width = line1_width.max(line2_width).max(line3_width);
        let text_height = 3;

        let paragraph = Paragraph::new(combined_text).style(
            Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(0, 0, 0)),
        );

        let x = (size.width.saturating_sub(text_width)) / 2;
        let y = (size.height.saturating_sub(text_height)) / 2;
        let area = ratatui::layout::Rect::new(x, y, text_width, text_height);
        frame.render_widget(paragraph, area);
    }
}
