use crossterm::event::KeyCode;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};

fn default_instant() -> Instant {
    Instant::now()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BattleMode {
    Dialogue,
    ExitTransition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dialogue {
    pub enemy_sprite_ansi: String,
    pub face_link: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleState {
    pub mode: BattleMode,
    pub dialogues: Vec<Dialogue>,
    pub current_dialogue_index: usize,
    pub narrative_text: String,
    pub narrative_face: Option<String>,
    pub animated_narrative_content: String,
    #[serde(skip, default = "default_instant")]
    pub narrative_animation_start_time: Instant,
    #[serde(skip)]
    pub narrative_animation_interval: Duration,
    #[serde(skip)]
    pub narrative_animation_finished: bool,
    #[serde(skip)]
    pub previous_chars_shown: usize,
    #[serde(skip, default = "default_instant")]
    pub exit_transition_timer: Instant,
    pub deltarune: crate::game::deltarune::Deltarune,
}

impl BattleState {
    pub fn new() -> Self {
        let dialogues = vec![
            Dialogue {
                                enemy_sprite_ansi: fs::read_to_string(
                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sprites/enemy/not_a_placeholder/battle_neutral.ans"),
                )
                .unwrap_or_default(),
                face_link: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sprites/faces/face_neutral.ans").to_string()),
                text: "Hello there, human. You've stumbled into my domain...".to_string(),
            },
            Dialogue {
                                enemy_sprite_ansi: fs::read_to_string(
                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sprites/enemy/not_a_placeholder/battle_smile.ans"),
                )
                .unwrap_or_default(),
                face_link: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sprites/faces/face_smile.ans").to_string()),
                text: "Prepare for a dialogue-only encounter! Mwahaha!".to_string(),
            },
            Dialogue {
                                enemy_sprite_ansi: fs::read_to_string(
                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sprites/enemy/not_a_placeholder/battle_hehehe.ans"),
                )
                .unwrap_or_default(),
                face_link: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sprites/faces/face_hehehe.ans").to_string()),
                text: "But seriously, this is just a test of the dialogue system...".to_string(),
            },
            Dialogue {
                                enemy_sprite_ansi: fs::read_to_string(
                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sprites/enemy/not_a_placeholder/battle_more_neutral.ans"),
                )
                .unwrap_or_default(),
                face_link: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sprites/faces/face_determined.ans").to_string()),
                text: "Press Enter to advance, or Esc/X to skip the current line.".to_string(),
            },
            Dialogue {
                                enemy_sprite_ansi: fs::read_to_string(
                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sprites/enemy/not_a_placeholder/battle_3.ans"),
                )
                .unwrap_or_default(),
                face_link: Some(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sprites/faces/face_3.ans").to_string()),
                text: "Watch out for the pauses... like this, ... and this. And this.".to_string(),
            },
        ];

        let current_dialogue = dialogues.first().cloned().unwrap_or_else(|| Dialogue {
            enemy_sprite_ansi: String::new(),
            face_link: None,
            text: "No dialogues defined.".to_string(),
        });

        BattleState {
            mode: BattleMode::Dialogue,
            dialogues,
            current_dialogue_index: 0,
            narrative_text: current_dialogue.text,
            narrative_face: current_dialogue.face_link,
            animated_narrative_content: String::new(),
            narrative_animation_start_time: Instant::now(),
            narrative_animation_interval: Duration::from_millis(50),
            narrative_animation_finished: false,
            previous_chars_shown: 0,
            exit_transition_timer: Instant::now(),
            deltarune: crate::game::deltarune::Deltarune::new(),
        }
    }

    pub fn update(
        &mut self,
        _delta_time: std::time::Duration,
        key_states: &HashMap<KeyCode, bool>,
        audio: &mut crate::audio::Audio,
        game_should_exit: &mut bool,
    ) {
        match self.mode {
            BattleMode::Dialogue => {
                let current_dialogue = &self.dialogues[self.current_dialogue_index];
                self.narrative_text = current_dialogue.text.clone();
                self.narrative_face = current_dialogue.face_link.clone();

                //  animation :O
                if !self.narrative_animation_finished {
                    if self.narrative_animation_start_time.elapsed() >= self.narrative_animation_interval {
                        let current_len = self.animated_narrative_content.chars().count();
                        if current_len < self.narrative_text.chars().count() {
                            let next_char_index = self.animated_narrative_content.chars().count();
                            let next_char = self.narrative_text.chars().nth(next_char_index).unwrap();
                            self.animated_narrative_content.push(next_char);
                            audio.play_text_sound(); // we scream every time we show a fucking
                                                     // character (-_-)=b

                            self.narrative_animation_interval = match next_char {
                                ' ' => Duration::from_millis(100),
                                ',' => Duration::from_millis(250), 
                                '.' => {
                                    if self.narrative_text[next_char_index..].starts_with("...") {
                                        Duration::from_millis(400) 
                                    } else {
                                        Duration::from_millis(300) 
                                    }
                                }
                                _ => Duration::from_millis(rand::thread_rng().gen_range(10..=30)),
                            };
                            self.narrative_animation_start_time = Instant::now();
                        } else {
                            self.narrative_animation_finished = true;
                        }
                    }
                }

                if *key_states.get(&KeyCode::Enter).unwrap_or(&false) {
                    if !self.narrative_animation_finished {
                        self.animated_narrative_content = self.narrative_text.clone();
                        self.narrative_animation_finished = true;
                    } else {
                        self.current_dialogue_index += 1;
                        if self.current_dialogue_index < self.dialogues.len() {
                            let next_dialogue = &self.dialogues[self.current_dialogue_index];
                            self.narrative_text = next_dialogue.text.clone();
                            self.narrative_face = next_dialogue.face_link.clone();
                            self.animated_narrative_content.clear();
                            self.narrative_animation_finished = false;
                            self.narrative_animation_start_time = Instant::now();
                        } else {
                            self.mode = BattleMode::ExitTransition;
                            self.exit_transition_timer = Instant::now();
                        }
                    }
                } else if *key_states.get(&KeyCode::Esc).unwrap_or(&false)
                    || *key_states.get(&KeyCode::Char('x')).unwrap_or(&false)
                {
                    if !self.narrative_animation_finished {
                        self.animated_narrative_content = self.narrative_text.clone();
                        self.narrative_animation_finished = true;
                    }
                }
            }
            BattleMode::ExitTransition => {
                if self.deltarune.level < 100 {
                    self.deltarune.increase();
                } else if self.exit_transition_timer.elapsed() > Duration::from_secs(3) {
                    *game_should_exit = true;
                }
            }
        }
    }
}