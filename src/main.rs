use std::{
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
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Terminal,
    prelude::Widget,
};
mod game_state;
mod player;
mod utils;



use crate::game_state::GameState;

const PLAYER_SPEED: u16 = 1; // Pixels per frame
const FRAME_RATE: u64 = 120; // Frames per second
const ANIMATION_FRAME_DURATION: Duration = Duration::from_millis(200);

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
    game_state.player.x = game_state.spawn_x;
    game_state.player.y = game_state.spawn_y;
    game_state.player.is_walking = false;
    game_state.player.animation_frame = 0;
    let mut last_frame_time = Instant::now();
    let mut last_animation_update = Instant::now();

    loop {
        let elapsed_time = last_frame_time.elapsed();
        if elapsed_time >= Duration::from_millis(1000 / FRAME_RATE) {
            last_frame_time = Instant::now();

            let mut key_code = None;
            while let Ok(Event::Key(key)) = rx.try_recv() {
                if key.code == KeyCode::Char('q') {
                    game_state.save_game_state()?;
                    return Ok(());
                }
                key_code = Some(key.code);
            }

            let current_frame_size = terminal.size()?;

            game_state.update(
                key_code,
                ratatui::layout::Rect::new(
                    0,
                    0,
                    current_frame_size.width,
                    current_frame_size.height,
                ),
            );

            if game_state.player.is_walking
                && last_animation_update.elapsed() >= ANIMATION_FRAME_DURATION
            {
                game_state.player.update_animation();
                last_animation_update = Instant::now();
            } else if !game_state.player.is_walking {
                game_state.player.animation_frame = 0; // Reset animation frame when not walking
            }

            terminal.draw(|frame| {
                let size = frame.area();

                let player_x_on_screen = game_state.player.x.saturating_sub(game_state.camera_x);
                let player_y_on_screen = game_state.player.y.saturating_sub(game_state.camera_y);

                let mut temp_buffer = ratatui::buffer::Buffer::empty(size);

                let mut processed_map_text = Text::default();
                let original_map_text = game_state.map_raw_data.as_bytes().into_text().unwrap();

                for line in original_map_text.lines {
                    let mut new_spans = Vec::new();
                    for span in line.spans {
                        if span.content == "█" {
                            new_spans.push(Span::styled(" ".to_string(), span.style.bg(Color::Black)));
                        } else {
                            new_spans.push(span.clone());
                        }
                    }
                    processed_map_text.lines.push(Line::from(new_spans));
                }

                let map_paragraph = Paragraph::new(processed_map_text)
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .scroll((game_state.camera_y, game_state.camera_x));
                map_paragraph.render(size, &mut temp_buffer);

                let (player_sprite_content, player_sprite_width, player_sprite_height) = game_state.player.get_sprite_content();
                let player_paragraph = Paragraph::new(player_sprite_content);
                player_paragraph.render(
                    ratatui::layout::Rect::new(player_x_on_screen, player_y_on_screen, player_sprite_width, player_sprite_height),
                    &mut temp_buffer,
                );

                let mut final_map_text = Text::default();
                for y_coord in 0..size.height {
                    let mut spans = Vec::new();
                    for x_coord in 0..size.width {
                        let cell = temp_buffer.cell(ratatui::layout::Position::new(x_coord, y_coord)).unwrap();
                        spans.push(Span::styled(cell.symbol().to_string(), cell.style()));
                    }
                    final_map_text.lines.push(Line::from(spans));
                }

                let final_map_paragraph = Paragraph::new(final_map_text)
                    .style(Style::default().fg(Color::White))
                    .scroll((game_state.camera_y, game_state.camera_x));
                frame.render_widget(final_map_paragraph, size);

                let spawn_x_on_screen = game_state.spawn_x.saturating_sub(game_state.camera_x).saturating_sub(1);
                let spawn_y_on_screen = game_state.spawn_y.saturating_sub(game_state.camera_y);

                if spawn_x_on_screen < size.width && spawn_y_on_screen < size.height {
                    let spawn_paragraph = Paragraph::new("S").style(Style::default().fg(Color::Green));
                    frame.render_widget(spawn_paragraph, ratatui::layout::Rect::new(spawn_x_on_screen, spawn_y_on_screen, 1, 1));
                }

                let debug_text = format!(
                    "Player: ({}, {}) Dir: {:?} Anim: {} Walking: {} Camera: ({}, {}) Map: ({}, {}) Screen: ({}, {}) NewCamY: {} MapH: {} FrameH: {}",
                    game_state.player.x,
                    game_state.player.y,
                    game_state.player.direction,
                    game_state.player.animation_frame,
                    game_state.player.is_walking,
                    game_state.camera_x,
                    game_state.camera_y,
                    game_state.map_width,
                    game_state.map_height,
                    player_x_on_screen,
                    player_y_on_screen,
                    game_state.camera_y,
                    game_state.map_height,
                    size.height
                );
                let debug_paragraph = Paragraph::new(debug_text)
                    .block(Block::default().borders(Borders::ALL).title("Debug"))
                    .style(Style::default().fg(Color::Cyan));
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(3)])
                    .split(size);
                frame.render_widget(debug_paragraph, chunks[1]);

                if game_state.show_message {
                    let message_paragraph = Paragraph::new(game_state.message.clone())
                        .block(Block::default().borders(Borders::ALL).title("Message"))
                        .style(Style::default().fg(Color::Yellow).bg(Color::Black));
                    let message_area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(0),
                            Constraint::Length(3),
                        ])
                        .split(size)[1]; // Render at the bottom
                    frame.render_widget(message_paragraph, message_area);
                }
            })?;
        }
    }
}

fn input_handler(tx: mpsc::Sender<Event>) -> io::Result<()> {
    loop {
        if event::poll(Duration::from_millis(50))? {
            tx.send(event::read()?)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }
    }
}

fn main() -> io::Result<()> {
    let result = run_app();

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}
