use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use rand::Rng;
use std::time::{Duration, Instant};

fn default_instant() -> Instant {
    Instant::now()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BattleMode {
    Menu,
    Act,
    Item,
    Attack,
    Defend,
    Narrative,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BattleButton {
    Fight,
    Act,
    Item,
    Mercy,
}

impl BattleButton {
    pub fn next(&self) -> Self {
        match self {
            BattleButton::Fight => BattleButton::Act,
            BattleButton::Act => BattleButton::Item,
            BattleButton::Item => BattleButton::Mercy,
            BattleButton::Mercy => BattleButton::Fight,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            BattleButton::Fight => BattleButton::Mercy,
            BattleButton::Act => BattleButton::Fight,
            BattleButton::Item => BattleButton::Act,
            BattleButton::Mercy => BattleButton::Item,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerHeart {
    pub x: f32,
    pub y: f32,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackSlider {
    pub position: f32,
    pub speed: f32,
    pub moving_right: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackType {
    Simple,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attack {
    pub attack_type: AttackType,
    pub duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub sprite_frames: Vec<String>,
    pub current_frame: usize,
    #[serde(skip, default = "default_instant")]
    pub last_frame_time: Instant,
    pub attacks: Vec<Attack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleState {
    pub mode: BattleMode,
    pub selected_button: BattleButton,
    pub enemy: Enemy,
    pub player_heart: PlayerHeart,
    pub attack_slider: AttackSlider,
    pub act_options: Vec<String>,
    pub selected_act_option: usize,
    pub message: Vec<String>,
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
    pub bullet_board_size: (u16, u16),
    pub current_attack: Option<Attack>,
    #[serde(skip, default = "default_instant")]
    pub attack_timer: Instant,
}

impl BattleState {
    pub fn new() -> Self {
        // TODO: Load enemy from battle trigger
        let mut sprite_frames = Vec::new();
        if let Ok(content) = fs::read_to_string("assets/sprites/ME/idle/default.ans") {
            sprite_frames.push(content);
        }

        let enemy = Enemy {
            name: "Placeholder".to_string(),
            hp: 100,
            max_hp: 100,
            sprite_frames,
            current_frame: 0,
            last_frame_time: Instant::now(),
            attacks: vec![Attack {
                attack_type: AttackType::Simple,
                duration: Duration::from_secs(5),
            }],
        };

        BattleState {
            mode: BattleMode::Menu,
            selected_button: BattleButton::Fight,
            enemy,
            player_heart: PlayerHeart {
                x: 50.0,
                y: 50.0,
                width: 3,
                height: 2,
            },
            attack_slider: AttackSlider {
                position: 0.0,
                speed: 100.0,
                moving_right: true,
            },
            act_options: vec!["Check".to_string(), "Talk".to_string()],
            selected_act_option: 0,
            message: vec!["* Placeholder enemy appears.".to_string()],
            narrative_text: String::new(), // Will be set by random dialogue
            narrative_face: None, // Will be set by random dialogue
            animated_narrative_content: String::new(),
            narrative_animation_start_time: Instant::now(),
            narrative_animation_interval: Duration::from_millis(50),
            narrative_animation_finished: false,
            previous_chars_shown: 0,
            bullet_board_size: (100, 100), // Default size, will be updated by UI
            current_attack: None,
            attack_timer: Instant::now(),
        }
    }

    pub fn update(&mut self, delta_time: std::time::Duration, key_states: &HashMap<KeyCode, bool>, audio: &crate::audio::Audio) {
        // Animate enemy
        if self.enemy.last_frame_time.elapsed() > Duration::from_millis(200) {
            self.enemy.current_frame = (self.enemy.current_frame + 1) % self.enemy.sprite_frames.len();
            self.enemy.last_frame_time = Instant::now();
        }

        // Animate narrative text
        if self.mode == BattleMode::Narrative && !self.narrative_animation_finished {
            if self.narrative_animation_start_time.elapsed() >= self.narrative_animation_interval {
                let current_len = self.animated_narrative_content.chars().count();
                if current_len < self.narrative_text.chars().count() {
                    let next_char_index = self.animated_narrative_content.chars().count();
                    let next_char = self.narrative_text.chars().nth(next_char_index).unwrap();
                    self.animated_narrative_content.push(next_char);
                    audio.play_text_sound();

                    self.narrative_animation_interval = match next_char {
                        ' ' => Duration::from_millis(100),
                        ',' => Duration::from_millis(150),
                        '.' => Duration::from_millis(200),
                        _ if self.narrative_text[next_char_index..].starts_with("...") => Duration::from_millis(300),
                        _ => Duration::from_millis(rand::thread_rng().gen_range(30..=70)),
                    };
                    self.narrative_animation_start_time = Instant::now();
                } else {
                    self.narrative_animation_finished = true;
                }
            }
        }

        match self.mode {
            BattleMode::Attack => {
                let speed = self.attack_slider.speed;
                if self.attack_slider.moving_right {
                    self.attack_slider.position += speed * delta_time.as_secs_f32();
                    if self.attack_slider.position >= 100.0 {
                        self.attack_slider.position = 100.0;
                        self.attack_slider.moving_right = false;
                    }
                } else {
                    self.attack_slider.position -= speed * delta_time.as_secs_f32();
                    if self.attack_slider.position <= 0.0 {
                        self.attack_slider.position = 0.0;
                        self.attack_slider.moving_right = true;
                    }
                }
            }
            BattleMode::Defend => {
                if self.current_attack.is_none() {
                    self.current_attack = Some(self.enemy.attacks[0].clone());
                    self.attack_timer = Instant::now();
                }

                if let Some(attack) = &self.current_attack {
                    if self.attack_timer.elapsed() >= attack.duration {
                        self.current_attack = None;
                        self.mode = BattleMode::Menu; // Go back to menu after attack
                    } else {
                        // Execute attack logic here
                    }
                }

                let speed = 100.0;
                if *key_states.get(&KeyCode::Up).unwrap_or(&false) {
                    self.player_heart.y -= speed * delta_time.as_secs_f32();
                }
                if *key_states.get(&KeyCode::Down).unwrap_or(&false) {
                    self.player_heart.y += speed * delta_time.as_secs_f32();
                }
                if *key_states.get(&KeyCode::Left).unwrap_or(&false) {
                    self.player_heart.x -= speed * delta_time.as_secs_f32();
                }
                if *key_states.get(&KeyCode::Right).unwrap_or(&false) {
                    self.player_heart.x += speed * delta_time.as_secs_f32();
                }

                // Clamp heart position to bullet board
                self.player_heart.x = self
                    .player_heart
                    .x
                    .max(0.0)
                    .min(self.bullet_board_size.0 as f32 - self.player_heart.width as f32);
                self.player_heart.y = self
                    .player_heart
                    .y
                    .max(0.0)
                    .min(self.bullet_board_size.1 as f32 - self.player_heart.height as f32);
            }
            _ => {}
        }
    }
}
