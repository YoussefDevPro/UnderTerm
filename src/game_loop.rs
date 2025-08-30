use std::{
    collections::HashMap,
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    audio::Audio,
    game::{config::FRAME_RATE, state::GameState},
    input, ui,
};

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    game_state: &mut GameState,
) -> io::Result<()> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        if let Err(e) = input::input_handler(tx) {
            eprintln!("Input handler error: {:?}", e);
        }
    });

    let mut last_frame_time = Instant::now();
    let audio = Audio::new().unwrap();

    loop {
        let elapsed_time = last_frame_time.elapsed();
        if elapsed_time >= Duration::from_millis(1000 / FRAME_RATE) {
            last_frame_time = Instant::now();

            let mut key_states: HashMap<crossterm::event::KeyCode, bool> = HashMap::new();

            if input::process_events(&rx, game_state, &mut key_states, &audio)? {
                return Ok(());
            }

            if game_state.esc_hold_dots >= 4 {
                return Ok(());
            }

            

            let current_frame_size = terminal.size()?;
            game_state.update(
                &key_states,
                ratatui::layout::Rect::new(
                    0,
                    0,
                    current_frame_size.width,
                    current_frame_size.height,
                ),
                crate::game::config::ANIMATION_FRAME_DURATION,
            );

            terminal.draw(|frame| {
                ui::draw(frame, game_state);
            })?;
        }
    }
}
