use std::io::{self, stdout, IsTerminal};
use under_term::{game, game_loop};

use crossterm::{
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};

fn run_app() -> io::Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    if let Err(e) = stdout().execute(PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
    )) {
        eprintln!("Could not enable keyboard enhancement flags: {:?}", e);
    }

    let mut game_state = game::state::GameState::load_game_state()?;
    game_state.player.is_walking = false;
    game_state.player.animation_frame = 0;

    

    let result = game_loop::run(&mut terminal, &mut game_state);

    if let Err(e) = stdout().execute(PopKeyboardEnhancementFlags) {
        eprintln!("Could not disable keyboard enhancement flags: {:?}", e);
    }
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

fn main() -> io::Result<()> {
    if !stdout().is_terminal() {
        eprintln!("This application must be run in a terminal.");
        return Ok(());
    }
    let result = run_app();
    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
        return Err(e);
    }
    Ok(())
}
