use std::{
    io::{self, stdout},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Clear},
    Terminal,
};
mod game_state;
mod player;
mod utils;
mod map;

use crate::game_state::GameState;

const PLAYER_SPEED: u16 = 1;
const PLAYER_HORIZONTAL_SPEED: u16 = 2;
const FRAME_RATE: u64 = 120;
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
                game_state.player.animation_frame = 0;
            }

            terminal.draw(|frame| {
                let size = frame.area();

                let player_x_on_screen = game_state.player.x.saturating_sub(game_state.camera_x);
                let player_y_on_screen = game_state.player.y.saturating_sub(game_state.camera_y);

                // Get the combined map text from GameState
                let combined_map_text = game_state.get_combined_map_text(size);

                let map_paragraph = Paragraph::new(combined_map_text)
                    .scroll((game_state.camera_y, game_state.camera_x)); // Use camera_y and camera_x
                frame.render_widget(map_paragraph, size);

                let current_map_key = (game_state.current_map_row, game_state.current_map_col);
                let current_map = game_state.loaded_maps.get(&current_map_key).unwrap();

                if game_state.debug_mode {
                    for &(wx, wy) in &current_map.walls { // Still using self.map.walls, need to adjust
                        let wall_x_on_screen = wx.saturating_sub(game_state.camera_x as u32); // Use camera_x
                        let wall_y_on_screen = wy.saturating_sub(game_state.camera_y as u32); // Use camera_y

                        // Clamp wall drawing to buffer size
                        if wall_x_on_screen < size.width as u32 && wall_y_on_screen < size.height as u32 {
                            let draw_rect = ratatui::layout::Rect::new(
                                wall_x_on_screen as u16,
                                wall_y_on_screen as u16,
                                1,
                                1,
                            );
                            // Ensure draw_rect is within size
                            let clamped_rect = draw_rect.intersection(size);
                            if !clamped_rect.is_empty() {
                                let wall_paragraph = Paragraph::new("W")
                                    .style(Style::default().fg(Color::Red).bg(Color::Black));
                                frame.render_widget(
                                    wall_paragraph,
                                    clamped_rect,
                                );
                            }
                        }
                    }
                }

                let (player_sprite_content, player_sprite_width, player_sprite_height) = game_state.player.get_sprite_content();
                let player_paragraph = Paragraph::new(player_sprite_content);

                let player_draw_rect = ratatui::layout::Rect::new(
                    player_x_on_screen,
                    player_y_on_screen,
                    player_sprite_width,
                    player_sprite_height,
                );
                // Ensure player_draw_rect is within size
                let clamped_player_rect = player_draw_rect.intersection(size);
                if !clamped_player_rect.is_empty() {
                    frame.render_widget(
                        player_paragraph,
                        clamped_player_rect,
                    );
                }

                

                if game_state.show_debug_panel {
                    let debug_text = vec![
                        format!("Player: ({}, {})", game_state.player.x, game_state.player.y),
                        format!("Direction: {:?}", game_state.player.direction),
                        format!("Animation Frame: {}", game_state.player.animation_frame),
                        format!("Walking: {}", game_state.player.is_walking),
                        format!("Camera: ({}, {})", game_state.camera_x, game_state.camera_y),
                        format!("Map: ({}, {})", current_map.width, current_map.height),
                        format!("Screen Player Pos: ({}, {})", player_x_on_screen, player_y_on_screen),
                        format!("Debug Mode: {}", game_state.debug_mode),
                        format!("Current Map: {} ({}, {})", game_state.current_map_name, game_state.current_map_row, game_state.current_map_col),
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

                    
                    let max_line_length = debug_text.iter().map(|s| s.len() as u16).max().unwrap_or(0);
                    let area = size; // Reintroduce area
                    let debug_panel_width = max_line_length + 2 + 2; // +2 for borders, +2 for horizontal padding (1 left, 1 right)
                    let debug_panel_height = debug_text.len() as u16 + 4; // +2 for borders, +2 for vertical padding

                    let margin = 2; // Define margin value
                    let x = area.width.saturating_sub(debug_panel_width + margin); // Right margin
                    let y = margin; // Top margin

                    let debug_panel_rect = ratatui::layout::Rect::new(x, y, debug_panel_width, debug_panel_height);
                    frame.render_widget(Clear, debug_panel_rect); // Clear the area before drawing the panel

                    frame.render_widget(debug_paragraph, debug_panel_rect);
                }

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
                        .split(size)[1];
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