use crate::debug;
use crate::game::battle::{BattleButton, BattleMode, BattleState};
use crate::game::state::{GameState, TeleportCreationState};
use ansi_to_tui::IntoText;
use rand::Rng;
use ratatui::prelude::{Line, Rect, Span, Text};

use std::time::Duration;

use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, BorderType, Clear, Gauge, Paragraph, Widget},
    Frame,
};

fn draw_battle(frame: &mut Frame, battle_state: &mut BattleState, game_state: &GameState) {
    let size = frame.area();
    let background = Block::default().bg(Color::Rgb(0, 0, 0));
    frame.render_widget(background, size);

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Enemy area
            Constraint::Percentage(40), // Player area
        ])
        .split(size);

    // Draw enemy
    if !battle_state.enemy.sprite_frames.is_empty() {
        let enemy_sprite = battle_state.enemy.sprite_frames[battle_state.enemy.current_frame].clone();
        let mut enemy_text = enemy_sprite.as_bytes().into_text().unwrap();
        let enemy_height = enemy_text.lines.len() as u16;
        let mut enemy_width = 0;
        for line in enemy_text.lines.iter() {
            let line_width = line.width() as u16;
            if line_width > enemy_width {
                enemy_width = line_width;
            }
        }

        // Clamp enemy dimensions to the available area
        let enemy_draw_width = enemy_width.min(chunks[0].width);
        let enemy_draw_height = enemy_height.min(chunks[0].height);

        let mut enemy_x = (chunks[0].x + (chunks[0].width.saturating_sub(enemy_draw_width)) / 2) as i32;
        let enemy_y = chunks[0].y + (chunks[0].height.saturating_sub(enemy_draw_height)) / 2;

        if battle_state.enemy.is_shaking {
            enemy_x += rand::thread_rng().gen_range(-1..=1);
        }

        // Apply black foreground if in Defend mode
        if battle_state.mode == BattleMode::Defend {
            enemy_text = game_state.darken_text(enemy_text, 100); // 100 means completely black
        }

        let enemy_area = ratatui::layout::Rect::new(enemy_x as u16, enemy_y, enemy_draw_width, enemy_draw_height);
        let enemy_paragraph = Paragraph::new(enemy_text);
        frame.render_widget(enemy_paragraph, enemy_area);
    }

    // Player area layout
    let player_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0), // Main box
            Constraint::Length(5), // Buttons
        ])
        .split(chunks[1]);

    let bottom_area = player_chunks[1];

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Buttons
            Constraint::Percentage(30), // HP
        ])
        .split(bottom_area);
    
    let button_area = bottom_chunks[0];
    let hp_area = bottom_chunks[1];

    // HP Gauge
    let hp_label = format!("HP {} / {}", battle_state.player_hp, battle_state.player_max_hp);
    let gauge = Gauge::default()
        .block(Block::default().title("KR"))
        .gauge_style(Style::default().fg(Color::Rgb(255, 255, 0)).bg(Color::Rgb(255, 0, 0)))
        .label(hp_label)
        .ratio((battle_state.player_hp as f64) / (battle_state.player_max_hp as f64));
    frame.render_widget(gauge, hp_area);

    let buttons_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(button_area);

    let buttons = [
        (BattleButton::Fight, "FIGHT", buttons_layout[0]),
        (BattleButton::Act, "ACT", buttons_layout[1]),
        (BattleButton::Item, "ITEM", buttons_layout[2]),
        (BattleButton::Mercy, "MERCY", buttons_layout[3]),
    ];

    for (button_type, text, area) in &buttons {
        let mut style = Style::default().fg(Color::Rgb(255, 165, 0)); // Orange
        if *button_type == BattleButton::Mercy {
            style = Style::default().fg(Color::Rgb(128, 128, 128));
        }

        let button_text = if battle_state.selected_button == *button_type {
            vec![
                Line::from(""),
                Line::from(vec![ 
                    Span::styled("❤ ", Style::default().fg(Color::Rgb(255, 0, 0))),
                    Span::styled(*text, style),
                ]),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from(vec![ 
                    Span::raw("  "),
                    Span::styled(*text, style),
                ]),
            ]
        };

        let button_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Color::Rgb(255, 165, 0)).bg(Color::Rgb(0, 0, 0))); // Orange border

        frame.render_widget(button_block, *area);
        frame.render_widget(Paragraph::new(button_text), *area);
    }

    // Main content box
    let box_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick).border_style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 0, 0)));
    let box_area = player_chunks[0];
    frame.render_widget(box_block.clone(), box_area);

    // Draw content inside the main box based on the mode
    match battle_state.mode {
        BattleMode::Menu => {
            let message = Paragraph::new(battle_state.message.join("\n"))
                .style(Style::default().fg(Color::Rgb(255, 255, 255)))
                .block(Block::default().padding(ratatui::widgets::Padding::new(2, 2, 1, 1)));
            frame.render_widget(message, box_area);
        }
        BattleMode::Act => {
            let mut act_lines = Vec::new();
            for (i, option) in battle_state.act_options.iter().enumerate() {
                if i == battle_state.selected_act_option {
                    act_lines.push(Line::from(vec![ 
                        Span::styled("❤ ", Style::default().fg(Color::Rgb(255, 0, 0))),
                        Span::styled(format!("* {}", option), Style::default().fg(Color::Rgb(255, 255, 255))),
                    ]));
                } else {
                    act_lines.push(Line::from(vec![ 
                        Span::raw("  "),
                        Span::styled(format!("* {}", option), Style::default().fg(Color::Rgb(255, 255, 255))),
                    ]));
                }
            }
            let act_list = Paragraph::new(act_lines)
                .block(Block::default().padding(ratatui::widgets::Padding::new(2, 2, 1, 1)));
            frame.render_widget(act_list, box_area);
        }
        BattleMode::Item => {
            let item_text = Paragraph::new("Item mode placeholder")
                .style(Style::default().fg(Color::Rgb(255, 255, 255)))
                .block(Block::default().padding(ratatui::widgets::Padding::new(2, 2, 1, 1)));
            frame.render_widget(item_text, box_area);
        }
        BattleMode::Attack => {
            let slider_width = 50;
            let slider_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Length(1),
                    Constraint::Percentage(40),
                ])
                .split(box_area.inner(Margin { vertical: 0, horizontal: (box_area.width.saturating_sub(slider_width)) / 2 }))[1];

            let slider_bar = "[".to_string() + &"=".repeat(slider_width as usize) + "]";
            let slider_paragraph = Paragraph::new(slider_bar).style(Style::default().fg(Color::Rgb(255, 255, 255)));
            frame.render_widget(slider_paragraph, slider_area);

            let red_center_width = slider_width / 10;
            let red_center_start = slider_area.x + (slider_width / 2) - (red_center_width / 2);
            let red_center_area = ratatui::layout::Rect::new(red_center_start, slider_area.y, red_center_width, 1);
            let red_center_paragraph = Paragraph::new("=".repeat(red_center_width as usize)).style(Style::default().fg(Color::Rgb(255, 0, 0)));
            frame.render_widget(red_center_paragraph, red_center_area);

            let stick_position = (battle_state.attack_slider.position / 100.0 * slider_width as f32) as u16;
            let stick_area = ratatui::layout::Rect::new(slider_area.x + stick_position, slider_area.y, 1, 1);
            let stick = Paragraph::new("|").style(Style::default().fg(Color::Rgb(255, 255, 0)));
            frame.render_widget(stick, stick_area);
        }
        BattleMode::Defend => {
            // The bullet board is a large, centered rectangle
            let board_area = ratatui::layout::Rect {
                x: size.width / 4,
                y: size.height / 4,
                width: size.width / 2,
                height: size.height / 2,
            };
            battle_state.bullet_board_size = (board_area.width, board_area.height);

            let board_block = Block::default().borders(Borders::ALL).border_type(BorderType::Thick).style(Style::default().bg(Color::Rgb(0, 0, 0))).border_style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 0, 0)));
            frame.render_widget(board_block, board_area);

            let heart_sprite = "  ▄ ▄  \n■██▄██■\n ▀███▀ \n   ▀   ";
            // Clamp heart position to be within the bullet board area to prevent panic
            let heart_x_f = battle_state.player_heart.x.max(0.0).min(board_area.width as f32 - battle_state.player_heart.width as f32);
            let heart_y_f = battle_state.player_heart.y.max(0.0).min(board_area.height as f32 - battle_state.player_heart.height as f32);
            let heart_x = board_area.x + heart_x_f as u16;
            let heart_y = board_area.y + heart_y_f as u16;

            let heart_area = ratatui::layout::Rect::new(heart_x, heart_y, battle_state.player_heart.width, battle_state.player_heart.height);
            let heart_paragraph = Paragraph::new(heart_sprite).style(Style::default().fg(Color::Rgb(255, 0, 0)));
            if battle_state.is_flickering && battle_state.flicker_timer.elapsed().as_millis() % 200 < 100 {
                // Don't draw the heart if flickering and in the "off" phase
            } else {
                frame.render_widget(heart_paragraph, heart_area);
            }

            // Draw bullets
            for bullet in &battle_state.bullets {
                let bullet_x = board_area.x + bullet.x as u16;
                let bullet_y = board_area.y + bullet.y as u16;
                if bullet_x < board_area.right() && bullet_y < board_area.bottom() {
                    let bullet_p = Paragraph::new(bullet.symbol.clone()).style(Style::default().fg(Color::Rgb(255, 255, 255)));
                    frame.render_widget(bullet_p, ratatui::layout::Rect::new(bullet_x, bullet_y, bullet.width, bullet.height));
                }
            }

            // Draw gun if present
            if let Some(gun_state) = &game_state.battle_state.as_ref().unwrap().gun_state {
                use crate::game::battle::GunState;
                use crate::game::battle::GunDirection;
                use ansi_to_tui::IntoText;

                let (gun_art, gun_width, gun_height) = match gun_state {
                    GunState::Charging { direction, .. } | GunState::Firing { direction, .. } => {
                        match direction {
                            GunDirection::Up => ("  ▲  \n █ \n███", 5, 3),
                            GunDirection::Down => ("███\n █ \n  ▼  ", 5, 3),
                            GunDirection::Left => (" █───\n████ \n █───", 6, 3),
                            GunDirection::Right => ("───█ \n ████\n───█ ", 6, 3),
                        }
                    }
                };

                let (gun_x, gun_y, _) = match gun_state {
                    GunState::Charging { x, y, direction, .. } => (*x, *y, direction),
                    GunState::Firing { x, y, direction, .. } => (*x, *y, direction),
                };

                let gun_text = gun_art.as_bytes().into_text().unwrap();
                let gun_paragraph = Paragraph::new(gun_text).style(Style::default().fg(Color::Rgb(255, 255, 0)));

                let draw_x = board_area.x + gun_x as u16;
                let draw_y = board_area.y + gun_y as u16;

                let gun_rect = ratatui::layout::Rect::new(draw_x, draw_y, gun_width, gun_height);
                frame.render_widget(gun_paragraph, gun_rect);
            }
        }
        BattleMode::GameOverTransition => {
            draw_game_over_transition(frame, battle_state);
        }
        BattleMode::GameOver => {
            draw_game_over(frame);
        }
        _ => { // For Narrative
            if battle_state.mode == BattleMode::Narrative || battle_state.mode == BattleMode::OpeningNarrative {
                let face_width = battle_state.narrative_face.as_ref()
                    .and_then(|name| game_state.face_manager.faces.get(name))
                    .map_or(0, |face| face.width + 1); // +1 for spacing

                let narrative_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(face_width),
                        Constraint::Min(0),
                    ])
                    .split(box_area.inner(Margin { vertical: 1, horizontal: 1 }));

                if face_width > 0 {
                    if let Some(face_name) = &battle_state.narrative_face {
                        if let Some(face) = game_state.face_manager.faces.get(face_name) {
                            let face_text = face.content.as_bytes().into_text().unwrap();
                            let face_paragraph = Paragraph::new(face_text);
                            frame.render_widget(face_paragraph, narrative_chunks[0]);
                        }
                    }
                }

                let narrative_paragraph = Paragraph::new(battle_state.animated_narrative_content.clone())
                    .wrap(ratatui::widgets::Wrap { trim: true })
                    .style(Style::default().fg(Color::Rgb(255, 255, 255)));
                frame.render_widget(narrative_paragraph, narrative_chunks[1]);
            } else {
                let text = Paragraph::new("...")
                    .style(Style::default().fg(Color::Rgb(255, 255, 255)))
                    .block(Block::default().padding(ratatui::widgets::Padding::new(2, 2, 1, 1)));
                frame.render_widget(text, box_area);
            }
        }
    }
}

fn draw_game_over_transition(frame: &mut Frame, battle_state: &mut BattleState) {
    frame.render_widget(Block::default().bg(Color::Rgb(0, 0, 0)), frame.area());
    let elapsed = battle_state.game_over_timer.elapsed();

    let heart_sprite = if elapsed < Duration::from_secs(1) {
        "  ▄ ▄  \n■██▄██■\n ▀███▀ \n   ▀   ".to_string()
    } else {
        "       \n■█ ▄█■\n ▀█ █▀ \n  ▀ ▀  ".to_string()
    };

    let heart_paragraph = Paragraph::new(heart_sprite)
        .style(Style::default().fg(Color::Rgb(255, 0, 0)))
        .alignment(ratatui::layout::Alignment::Center);
    
    frame.render_widget(heart_paragraph, frame.area());
}

fn draw_game_over(frame: &mut Frame) {
    let game_over_text = "GAME OVER";
    let p = Paragraph::new(game_over_text)
        .style(Style::default().fg(Color::Rgb(255, 255, 255)))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(p, frame.area());

    // Add replay button
    let replay_text = "Press Enter to Replay";
    let replay_p = Paragraph::new(replay_text)
        .style(Style::default().fg(Color::Rgb(255, 255, 255)))
        .alignment(ratatui::layout::Alignment::Center);

    let area = frame.area();
    let replay_area = ratatui::layout::Rect::new(area.x, area.y + area.height / 2 + 2, area.width, 1);
    frame.render_widget(replay_p, replay_area);
}

pub fn draw(frame: &mut Frame, game_state: &mut GameState) {
    let size = frame.area();

    if game_state.battle_page_active {
        let mut temp_battle_state = game_state.battle_state.take();
        if let Some(battle_state) = &mut temp_battle_state {
            draw_battle(frame, battle_state, game_state);
        } else {
            // This should not happen, but as a fallback
            frame.render_widget(Block::default().bg(Color::Rgb(0, 0, 0)), size);
            let text = Paragraph::new("Battle active, but no state found!")
                .style(Style::default().fg(Color::Rgb(255, 0, 0)));
            frame.render_widget(text, size);
        }
        game_state.battle_state = temp_battle_state; // Put it back
        return;
    } else if game_state.game_over_active {
        draw_game_over(frame);
        return;
    } else if game_state.battle_transition_timer.is_some() {
        // Draw map and player (already drawn by default)

        // Calculate transition progress
        let elapsed = game_state.battle_transition_timer.unwrap().elapsed();
        let transition_duration = Duration::from_millis(300);
        let progress = (elapsed.as_secs_f32() / transition_duration.as_secs_f32()).min(1.0);

        // Calculate shrinking rectangle (from full screen to battle box size)
        let current_x_offset = (size.width as f32 * progress / 2.0) as u16;
        let current_y_offset = (size.height as f32 * progress / 2.0) as u16;
        let _current_width_inner = size.width.saturating_sub(2 * current_x_offset);
        let current_height_inner = size.height.saturating_sub(2 * current_y_offset);

        // Draw black rectangle that grows from edges towards center
        let top_rect = Rect::new(0, 0, size.width, current_y_offset);
        let bottom_rect = Rect::new(0, size.height.saturating_sub(current_y_offset), size.width, current_y_offset);
        let left_rect = Rect::new(0, current_y_offset, current_x_offset, current_height_inner);
        let right_rect = Rect::new(size.width.saturating_sub(current_x_offset), current_y_offset, current_x_offset, current_height_inner);

        frame.render_widget(Clear, top_rect);
        frame.render_widget(Clear, bottom_rect);
        frame.render_widget(Clear, left_rect);
        frame.render_widget(Clear, right_rect);

        return; // Don't draw overworld map again
    } else if game_state.battle_transition_timer.is_some() {
        // Draw map and player (already drawn by default)

        // Calculate transition progress
        let elapsed = game_state.battle_transition_timer.unwrap().elapsed();
        let transition_duration = Duration::from_millis(300);
        let progress = (elapsed.as_secs_f32() / transition_duration.as_secs_f32()).min(1.0);

        // Calculate shrinking rectangle (from full screen to battle box size)
        let current_x_offset = (size.width as f32 * progress / 2.0) as u16;
        let current_y_offset = (size.height as f32 * progress / 2.0) as u16;
        let _current_width_inner = size.width.saturating_sub(2 * current_x_offset);
        let current_height_inner = size.height.saturating_sub(2 * current_y_offset);

        // Draw black rectangle that grows from edges towards center
        let top_rect = Rect::new(0, 0, size.width, current_y_offset);
        let bottom_rect = Rect::new(0, size.height.saturating_sub(current_y_offset), size.width, current_y_offset);
        let left_rect = Rect::new(0, current_y_offset, current_x_offset, current_height_inner);
        let right_rect = Rect::new(size.width.saturating_sub(current_x_offset), current_y_offset, current_x_offset, current_height_inner);

        frame.render_widget(Clear, top_rect);
        frame.render_widget(Clear, bottom_rect);
        frame.render_widget(Clear, left_rect);
        frame.render_widget(Clear, right_rect);

        return; // Don't draw overworld map again
    } else if game_state.battle_transition_timer.is_some() {
        // Draw map and player (already drawn by default)

        // Calculate transition progress
        let elapsed = game_state.battle_transition_timer.unwrap().elapsed();
        let transition_duration = Duration::from_millis(300);
        let progress = (elapsed.as_secs_f32() / transition_duration.as_secs_f32()).min(1.0);

        // Calculate shrinking rectangle (from full screen to battle box size)
        let current_x_offset = (size.width as f32 * progress / 2.0) as u16;
        let current_y_offset = (size.height as f32 * progress / 2.0) as u16;
        let _current_width_inner = size.width.saturating_sub(2 * current_x_offset);
        let current_height_inner = size.height.saturating_sub(2 * current_y_offset);

        // Draw black rectangle that grows from edges towards center
        let top_rect = Rect::new(0, 0, size.width, current_y_offset);
        let bottom_rect = Rect::new(0, size.height.saturating_sub(current_y_offset), size.width, current_y_offset);
        let left_rect = Rect::new(0, current_y_offset, current_x_offset, current_height_inner);
        let right_rect = Rect::new(size.width.saturating_sub(current_x_offset), current_y_offset, current_x_offset, current_height_inner);

        frame.render_widget(Clear, top_rect);
        frame.render_widget(Clear, bottom_rect);
        frame.render_widget(Clear, left_rect);
        frame.render_widget(Clear, right_rect);

        return; // Don't draw overworld map again
    } else if game_state.battle_transition_timer.is_some() {
        // Draw map and player (already drawn by default)

        // Calculate transition progress
        let elapsed = game_state.battle_transition_timer.unwrap().elapsed();
        let transition_duration = Duration::from_millis(300);
        let progress = (elapsed.as_secs_f32() / transition_duration.as_secs_f32()).min(1.0);

        // Calculate shrinking rectangle (from full screen to battle box size)
        let current_x_offset = (size.width as f32 * progress / 2.0) as u16;
        let current_y_offset = (size.height as f32 * progress / 2.0) as u16;
        let _current_width_inner = size.width.saturating_sub(2 * current_x_offset);
        let current_height_inner = size.height.saturating_sub(2 * current_y_offset);

        // Draw black rectangle that grows from edges towards center
        let top_rect = Rect::new(0, 0, size.width, current_y_offset);
        let bottom_rect = Rect::new(0, size.height.saturating_sub(current_y_offset), size.width, current_y_offset);
        let left_rect = Rect::new(0, current_y_offset, current_x_offset, current_height_inner);
        let right_rect = Rect::new(size.width.saturating_sub(current_x_offset), current_y_offset, current_x_offset, current_height_inner);

        frame.render_widget(Clear, top_rect);
        frame.render_widget(Clear, bottom_rect);
        frame.render_widget(Clear, left_rect);
        frame.render_widget(Clear, right_rect);

        return; // Don't draw overworld map again
    } else if game_state.battle_transition_timer.is_some() {
        // Draw map and player (already drawn by default)

        // Calculate transition progress
        let elapsed = game_state.battle_transition_timer.unwrap().elapsed();
        let transition_duration = Duration::from_millis(300);
        let progress = (elapsed.as_secs_f32() / transition_duration.as_secs_f32()).min(1.0);

        // Calculate shrinking rectangle (from full screen to battle box size)
        let current_x_offset = (size.width as f32 * progress / 2.0) as u16;
        let current_y_offset = (size.height as f32 * progress / 2.0) as u16;
        let _current_width_inner = size.width.saturating_sub(2 * current_x_offset);
        let current_height_inner = size.height.saturating_sub(2 * current_y_offset);

        // Draw black rectangle that grows from edges towards center
        let top_rect = Rect::new(0, 0, size.width, current_y_offset);
        let bottom_rect = Rect::new(0, size.height.saturating_sub(current_y_offset), size.width, current_y_offset);
        let left_rect = Rect::new(0, current_y_offset, current_x_offset, current_height_inner);
        let right_rect = Rect::new(size.width.saturating_sub(current_x_offset), current_y_offset, current_x_offset, current_height_inner);

        frame.render_widget(Clear, top_rect);
        frame.render_widget(Clear, bottom_rect);
        frame.render_widget(Clear, left_rect);
        frame.render_widget(Clear, right_rect);

        return; // Don't draw overworld map again
    } else if game_state.battle_transition_timer.is_some() {
        // Draw map and player (already drawn by default)

        // Calculate transition progress
        let elapsed = game_state.battle_transition_timer.unwrap().elapsed();
        let transition_duration = Duration::from_millis(300);
        let progress = (elapsed.as_secs_f32() / transition_duration.as_secs_f32()).min(1.0);

        // Calculate shrinking rectangle (from full screen to battle box size)
        let current_x_offset = (size.width as f32 * progress / 2.0) as u16;
        let current_y_offset = (size.height as f32 * progress / 2.0) as u16;
        let _current_width_inner = size.width.saturating_sub(2 * current_x_offset);
        let current_height_inner = size.height.saturating_sub(2 * current_y_offset);

        // Draw black rectangle that grows from edges towards center
        let top_rect = Rect::new(0, 0, size.width, current_y_offset);
        let bottom_rect = Rect::new(0, size.height.saturating_sub(current_y_offset), size.width, current_y_offset);
        let left_rect = Rect::new(0, current_y_offset, current_x_offset, current_height_inner);
        let right_rect = Rect::new(size.width.saturating_sub(current_x_offset), current_y_offset, current_x_offset, current_height_inner);

        frame.render_widget(Clear, top_rect);
        frame.render_widget(Clear, bottom_rect);
        frame.render_widget(Clear, left_rect);
        frame.render_widget(Clear, right_rect);

        return; // Don't draw overworld map again
    } else if game_state.battle_transition_timer.is_some() {
        // Draw map and player (already drawn by default)

        // Calculate transition progress
        let elapsed = game_state.battle_transition_timer.unwrap().elapsed();
        let transition_duration = Duration::from_millis(300);
        let progress = (elapsed.as_secs_f32() / transition_duration.as_secs_f32()).min(1.0);

        // Calculate shrinking rectangle (from full screen to battle box size)
        let current_x_offset = (size.width as f32 * progress / 2.0) as u16;
        let current_y_offset = (size.height as f32 * progress / 2.0) as u16;
        let _current_width_inner = size.width.saturating_sub(2 * current_x_offset);
        let current_height_inner = size.height.saturating_sub(2 * current_y_offset);

        // Draw black rectangle that grows from edges towards center
        let top_rect = Rect::new(0, 0, size.width, current_y_offset);
        let bottom_rect = Rect::new(0, size.height.saturating_sub(current_y_offset), size.width, current_y_offset);
        let left_rect = Rect::new(0, current_y_offset, current_x_offset, current_height_inner);
        let right_rect = Rect::new(size.width.saturating_sub(current_x_offset), current_y_offset, current_x_offset, current_height_inner);

        frame.render_widget(Clear, top_rect);
        frame.render_widget(Clear, bottom_rect);
        frame.render_widget(Clear, left_rect);
        frame.render_widget(Clear, right_rect);

        return; // Don't draw overworld map again
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
                    let wall_paragraph = Paragraph::new("W").style(Style::default().fg(Color::Rgb(255, 0, 0)));
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
            .border_style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(255, 255, 255)))
            .style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 0, 0)))
            .padding(ratatui::widgets::Padding::new(8, 8, 1, 1))
            .title("Message");

        let message_paragraph = Paragraph::new(game_state.animated_message_content.clone())
            .style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 0, 0)))
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
            .border_style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(255, 255, 255)))
            .style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 0, 0)))
            .padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
            .title(title);

        let input_paragraph = Paragraph::new(game_state.text_input_buffer.clone())
            .style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 0, 0)))
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
            .border_style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(255, 255, 255)))
            .style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 0, 0)))
            .padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
            .title("Enter Event");

        let input_paragraph = Paragraph::new(game_state.text_input_buffer.clone())
            .style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 0, 0)))
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
            .border_style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(255, 255, 255)))
            .style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 0, 0)))
            .padding(ratatui::widgets::Padding::new(1, 1, 1, 1))
            .title("Select Map Kind");

        let current_map_key = (game_state.current_map_row, game_state.current_map_col);
        let current_map_kind = game_state
            .loaded_maps
            .get(&current_map_key)
            .map(|m| format!("{:?}", m.kind))
            .unwrap_or_else(|| "Unknown or deltarune".to_string());

        let input_paragraph = Paragraph::new(format!("Current: {}", current_map_kind))
            .style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 0, 0)))
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
            .style(Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 0, 0)));

        let x = (size.width.saturating_sub(text_width)) / 2;
        let y = (size.height.saturating_sub(text_height)) / 2;
        let area = ratatui::layout::Rect::new(x, y, text_width, text_height);
        frame.render_widget(paragraph, area);
    }
}