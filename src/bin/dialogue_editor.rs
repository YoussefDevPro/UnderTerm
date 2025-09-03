use std::io::stdout;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph, ListState},
    style::{Style, Color},
    Terminal,
};
use std::fs;
use std::path::Path;
use under_term::game::dialogue::{Dialogue, DialogueManager};
use serde_json;
use std::io;
use ansi_to_tui::IntoText;

enum EditorState {
    SelectFace,
    EnterText,
    SelectEnemy,
    ConfirmSave,
}

struct Editor {
    state: EditorState,
    faces: Vec<String>,
    enemies: Vec<String>,
    selected_face_index: usize,
    selected_enemy_index: usize,
    text: String,
    new_dialogue: Dialogue,
    should_quit: bool,
    face_list_state: ListState,
    enemy_list_state: ListState,
}

impl Editor {
    fn new() -> Self {
        let faces = find_files("assets/sprites/faces");
        let enemies = find_files("assets/sprites/enemy");

        let mut face_list_state = ListState::default();
        face_list_state.select(Some(0));
        let mut enemy_list_state = ListState::default();
        enemy_list_state.select(Some(0));

        Editor {
            state: EditorState::SelectFace,
            faces,
            enemies,
            selected_face_index: 0,
            selected_enemy_index: 0,
            text: String::new(),
            new_dialogue: Dialogue {
                face_ansi_path: String::new(),
                text: String::new(),
                enemy_ansi_path: String::new(),
            },
            should_quit: false,
            face_list_state,
            enemy_list_state,
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        while !self.should_quit {
            self.draw(terminal)?;
            self.handle_input()?;
        }
        Ok(())
    }

    fn draw(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.area());

            match self.state {
                EditorState::SelectFace => {
                    let items: Vec<ListItem> = self
                        .faces
                        .iter()
                        .map(|i| ListItem::new(Path::new(i).file_name().unwrap().to_str().unwrap()))
                        .collect();
                    let list = List::new(items)
                        .block(Block::default().title("Select Face").borders(Borders::ALL))
                        .highlight_style(Style::default().bg(Color::Yellow));
                    f.render_stateful_widget(list, chunks[0], &mut self.face_list_state);

                    if let Some(path) = self.faces.get(self.selected_face_index) {
                        let content = fs::read_to_string(path).unwrap_or_default();
                        let text = content.as_bytes().into_text().unwrap();
                        let paragraph = Paragraph::new(text)
                            .block(Block::default().title("Preview").borders(Borders::ALL));
                        f.render_widget(paragraph, chunks[1]);
                    }
                }
                EditorState::EnterText => {
                    let paragraph = Paragraph::new(self.text.as_str())
                        .block(Block::default().title("Enter Text (Press Enter when done)").borders(Borders::ALL));
                    f.render_widget(paragraph, f.area());
                }
                EditorState::SelectEnemy => {
                    let items: Vec<ListItem> = self
                        .enemies
                        .iter()
                        .map(|i| ListItem::new(Path::new(i).file_name().unwrap().to_str().unwrap()))
                        .collect();
                    let list = List::new(items)
                        .block(Block::default().title("Select Enemy").borders(Borders::ALL))
                        .highlight_style(Style::default().bg(Color::Yellow));
                    f.render_stateful_widget(list, chunks[0], &mut self.enemy_list_state);

                    if let Some(path) = self.enemies.get(self.selected_enemy_index) {
                        let content = fs::read_to_string(path).unwrap_or_default();
                        let text = content.as_bytes().into_text().unwrap();
                        let paragraph = Paragraph::new(text)
                            .block(Block::default().title("Preview").borders(Borders::ALL));
                        f.render_widget(paragraph, chunks[1]);
                    }
                }
                EditorState::ConfirmSave => {
                    let text = "Press Enter to save and create another dialogue, or X to save and exit.";
                    let paragraph = Paragraph::new(text)
                        .block(Block::default().title("Confirm").borders(Borders::ALL));
                    f.render_widget(paragraph, f.area());
                }
            }
        })?;
        Ok(())
    }

    fn handle_input(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            match self.state {
                EditorState::SelectFace => {
                    match key.code {
                        KeyCode::Up => {
                            if self.selected_face_index > 0 {
                                self.selected_face_index -= 1;
                                self.face_list_state.select(Some(self.selected_face_index));
                            }
                        }
                        KeyCode::Down => {
                            if self.selected_face_index < self.faces.len() - 1 {
                                self.selected_face_index += 1;
                                self.face_list_state.select(Some(self.selected_face_index));
                            }
                        }
                        KeyCode::Enter => {
                            self.new_dialogue.face_ansi_path = self.faces[self.selected_face_index].clone();
                            self.state = EditorState::EnterText;
                        }
                        _ => {}
                    }
                }
                EditorState::EnterText => {
                    match key.code {
                        KeyCode::Char(c) => self.text.push(c),
                        KeyCode::Backspace => {
                            self.text.pop();
                        }
                        KeyCode::Enter => {
                            self.new_dialogue.text = self.text.clone();
                            self.state = EditorState::SelectEnemy;
                        }
                        _ => {}
                    }
                }
                EditorState::SelectEnemy => {
                    match key.code {
                        KeyCode::Up => {
                            if self.selected_enemy_index > 0 {
                                self.selected_enemy_index -= 1;
                                self.enemy_list_state.select(Some(self.selected_enemy_index));
                            }
                        }
                        KeyCode::Down => {
                            if self.selected_enemy_index < self.enemies.len() - 1 {
                                self.selected_enemy_index += 1;
                                self.enemy_list_state.select(Some(self.selected_enemy_index));
                            }
                        }
                        KeyCode::Enter => {
                            self.new_dialogue.enemy_ansi_path = self.enemies[self.selected_enemy_index].clone();
                            self.state = EditorState::ConfirmSave;
                        }
                        _ => {}
                    }
                }
                EditorState::ConfirmSave => {
                    match key.code {
                        KeyCode::Enter => {
                            self.save_dialogue();
                            self.reset();
                        }
                        KeyCode::Char('x') => {
                            self.save_dialogue();
                            self.should_quit = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn save_dialogue(&self) {
        let mut manager = DialogueManager::new();
        manager.dialogues.push(self.new_dialogue.clone());
        let serialized = serde_json::to_string_pretty(&manager.dialogues).unwrap();
        fs::write("dialogues.json", serialized).unwrap();
    }

    fn reset(&mut self) {
        self.state = EditorState::SelectFace;
        self.selected_face_index = 0;
        self.selected_enemy_index = 0;
        self.text = String::new();
        self.new_dialogue = Dialogue {
            face_ansi_path: String::new(),
            text: String::new(),
            enemy_ansi_path: String::new(),
        };
        self.face_list_state.select(Some(0));
        self.enemy_list_state.select(Some(0));
    }
}

fn find_files(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "ans" {
                            files.push(path.to_str().unwrap().to_string());
                        }
                    }
                } else if path.is_dir() {
                    files.extend(find_files(path.to_str().unwrap()));
                }
            }
        }
    }
    files
}



fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;

    let mut editor = Editor::new();
    let result = editor.run(&mut terminal);

    restore_terminal(terminal)?;

    result
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()
}
