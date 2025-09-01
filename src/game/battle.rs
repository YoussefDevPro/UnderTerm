use crate::game::attack::{Attack, AttackType, Bullet};
use crossterm::event::KeyCode;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};

const VERTICAL_SPEED_FACTOR: f32 = 0.5;

fn default_instant() -> Instant {
    Instant::now()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BattleMode {
    OpeningNarrative,
    Menu,
    Act,
    Item,
    Attack,
    Defend,
    Narrative,
    GameOverTransition,
    GameOver,
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
pub struct Enemy {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub sprite_frames: Vec<String>,
    pub current_frame: usize,
    #[serde(skip, default = "default_instant")]
    pub last_frame_time: Instant,
    pub attacks: Vec<Attack>,
    #[serde(skip, default = "default_instant")]
    pub shake_timer: Instant,
    #[serde(skip)]
    pub is_shaking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleState {
    pub mode: BattleMode,
    pub selected_button: BattleButton,
    pub enemy: Enemy,
    pub player_hp: i32,
    pub player_max_hp: i32,
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
    pub bullets: Vec<Bullet>,
    pub bullet_board_size: (u16, u16),
    pub current_attack: Option<Attack>,
    #[serde(skip, default = "default_instant")]
    pub attack_timer: Instant,
    #[serde(skip, default = "default_instant")]
    pub game_over_timer: Instant,
    #[serde(skip)]
    pub is_flickering: bool,
    #[serde(skip, default = "default_instant")]
    pub flicker_timer: Instant,
    #[serde(skip)]
    pub flicker_duration: Duration,
    pub gun_state: Option<GunState>,
    #[serde(skip)]
    pub wave_splash_state: Option<WaveSplashState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WaveSplashState {
    Charging {
        #[serde(skip, default = "default_instant")]
        start_time: Instant,
        duration: Duration,
        x: f32,
        y: f32,
    },
    Firing {
        #[serde(skip, default = "default_instant")]
        start_time: Instant,
        duration: Duration,
        x: f32,
        y: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GunState {
    Charging {
        #[serde(skip, default = "default_instant")]
        start_time: Instant,
        duration: Duration,
        x: f32,
        y: f32,
        direction: GunDirection,
    },
    Firing {
        #[serde(skip, default = "default_instant")]
        start_time: Instant,
        duration: Duration,
        x: f32,
        y: f32,
        direction: GunDirection,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GunDirection {
    Up,
    Down,
    Left,
    Right,
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
            attacks: vec![
                Attack {
                    attack_type: AttackType::Simple,
                    duration: Duration::from_secs(5),
                    damage: 2,
                    bullet_speed: 50.0,
                    bullet_symbol: "■".to_string(),
                    spawn_rate: 0.1,
                    wave_amplitude: None,
                    wave_frequency: None,
                },
                Attack {
                    attack_type: AttackType::Bouncing,
                    duration: Duration::from_secs(7),
                    damage: 3,
                    bullet_speed: 70.0,
                    bullet_symbol: "●".to_string(),
                    spawn_rate: 0.05,
                    wave_amplitude: None,
                    wave_frequency: None,
                },
                Attack {
                    attack_type: AttackType::Wave,
                    duration: Duration::from_secs(8),
                    damage: 2,
                    bullet_speed: 40.0,
                    bullet_symbol: "~".to_string(),
                    spawn_rate: 0.08,
                    wave_amplitude: Some(10.0),
                    wave_frequency: Some(0.5),
                },
            ],
            shake_timer: Instant::now(),
            is_shaking: false,
        };

        let opening_dialogue = "* Placeholder enemy appears.".to_string();

        BattleState {
            mode: BattleMode::OpeningNarrative,
            selected_button: BattleButton::Fight,
            enemy,
            player_hp: 20,
            player_max_hp: 20,
            player_heart: PlayerHeart {
                x: 50.0,
                y: 50.0,
                width: 7,
                height: 4,
            },
            attack_slider: AttackSlider {
                position: 0.0,
                speed: 100.0,
                moving_right: true,
            },
            act_options: vec!["Check".to_string(), "Talk".to_string()],
            selected_act_option: 0,
            message: vec![opening_dialogue.clone()],
            narrative_text: opening_dialogue,
            narrative_face: None, // No face for opening dialogue, or choose one
            animated_narrative_content: String::new(),
            narrative_animation_start_time: Instant::now(),
            narrative_animation_interval: Duration::from_millis(50),
            narrative_animation_finished: false,
            previous_chars_shown: 0,
            bullets: Vec::new(),
            bullet_board_size: (100, 100), // Default size, will be updated by UI
            current_attack: None,
            attack_timer: Instant::now(),
            game_over_timer: Instant::now(),
            is_flickering: false,
            flicker_timer: Instant::now(),
            flicker_duration: Duration::from_secs(2),
            gun_state: None,
            wave_splash_state: None,
        }
    }

    pub fn update(
        &mut self,
        delta_time: std::time::Duration,
        key_states: &HashMap<KeyCode, bool>,
        audio: &mut crate::audio::Audio,
    ) {
        // Animate enemy
        if self.enemy.last_frame_time.elapsed() > Duration::from_millis(200) {
            self.enemy.current_frame =
                (self.enemy.current_frame + 1) % self.enemy.sprite_frames.len();
            self.enemy.last_frame_time = Instant::now();
        }

        if self.enemy.is_shaking {
            if self.enemy.shake_timer.elapsed() > Duration::from_millis(200) {
                self.enemy.is_shaking = false;
            }
        }

        // Animate narrative text
        if (self.mode == BattleMode::Narrative || self.mode == BattleMode::OpeningNarrative)
            && !self.narrative_animation_finished
        {
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
                        _ if self.narrative_text[next_char_index..].starts_with("...") => {
                            Duration::from_millis(300)
                        }
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
                    let mut rng = rand::thread_rng();
                    let random_attack_index = rng.gen_range(0..self.enemy.attacks.len());
                    self.current_attack = Some(self.enemy.attacks[random_attack_index].clone());
                    self.attack_timer = Instant::now();
                    self.bullets.clear(); // Clear bullets from previous attack

                    if let Some(attack) = &self.current_attack {
                        if attack.attack_type == AttackType::Wave {
                            let (x, y) = (
                                rng.gen_range(0.0..self.bullet_board_size.0 as f32),
                                rng.gen_range(0.0..self.bullet_board_size.1 as f32),
                            );
                            self.wave_splash_state = Some(WaveSplashState::Charging {
                                start_time: Instant::now(),
                                duration: Duration::from_secs(1), // 1 second charge time
                                x,
                                y,
                            });
                        }
                    }

                    
                }

                if let Some(attack) = &self.current_attack {
                    
                        if self.attack_timer.elapsed() >= attack.duration {
                        self.current_attack = None;
                        self.mode = BattleMode::Menu; // Go back to menu after attack
                        self.gun_state = None; // Clear gun state
                        self.wave_splash_state = None; // Clear wave splash state
                    } else if attack.attack_type == AttackType::Wave {
                        if let Some(wave_state) = &mut self.wave_splash_state {
                            match wave_state {
                                WaveSplashState::Charging {
                                    start_time,
                                    duration,
                                    x,
                                    y,
                                } => {
                                    if start_time.elapsed() >= *duration {
                                        *wave_state = WaveSplashState::Firing {
                                            start_time: Instant::now(),
                                            duration: Duration::from_secs(2), // Firing duration
                                            x: *x,
                                            y: *y,
                                        };
                                    }
                                }
                                WaveSplashState::Firing {
                                    start_time: _,
                                    duration: _,
                                    x,
                                    y,
                                } => {
                                    // Spawn bullets in a radial pattern
                                    if rand::thread_rng().gen_bool(attack.spawn_rate) {
                                        let num_bullets = 8; // Number of bullets in the splash
                                        for i in 0..num_bullets {
                                            let angle = (i as f32 / num_bullets as f32) * 2.0 * std::f32::consts::PI;
                                            let vx = attack.bullet_speed * angle.cos();
                                            let vy = attack.bullet_speed * angle.sin();

                                            let bullet = Bullet {
                                                x: *x,
                                                y: *y,
                                                vx,
                                                vy,
                                                width: 1,
                                                height: 1,
                                                symbol: attack.bullet_symbol.clone(),
                                                bounces_remaining: 0,
                                            };
                                            self.bullets.push(bullet);
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Spawn bullets based on attack pattern
                        if rand::thread_rng().gen_bool(attack.spawn_rate) {
                            let bullet = match attack.attack_type {
                                AttackType::Simple => {
                                    let mut rng = rand::thread_rng();
                                    let side = rng.gen_range(0..4); // 0: top, 1: bottom, 2: left, 3: right
                                    let (start_x, start_y, target_x, target_y) = match side {
                                        0 => (
                                            // Top
                                            rng.gen_range(0.0..self.bullet_board_size.0 as f32),
                                            0.0,
                                            self.player_heart.x,
                                            self.player_heart.y,
                                        ),
                                        1 => (
                                            // Bottom
                                            rng.gen_range(0.0..self.bullet_board_size.0 as f32),
                                            self.bullet_board_size.1 as f32,
                                            self.player_heart.x,
                                            self.player_heart.y,
                                        ),
                                        2 => (
                                            // Left
                                            0.0,
                                            rng.gen_range(0.0..self.bullet_board_size.1 as f32),
                                            self.player_heart.x,
                                            self.player_heart.y,
                                        ),
                                        3 => (
                                            // Right
                                            self.bullet_board_size.0 as f32,
                                            rng.gen_range(0.0..self.bullet_board_size.1 as f32),
                                            self.player_heart.x,
                                            self.player_heart.y,
                                        ),
                                        _ => unreachable!(),
                                    };

                                    let dx = target_x - start_x;
                                    let dy = target_y - start_y;
                                    let distance = (dx * dx + dy * dy).sqrt();
                                    let (vx, vy) = if distance > 0.0 {
                                        (
                                            attack.bullet_speed * dx / distance,
                                            attack.bullet_speed * dy / distance,
                                        )
                                    } else {
                                        (0.0, attack.bullet_speed) // Fallback if bullet and target are at the same spot
                                    };

                                    Bullet {
                                        x: start_x,
                                        y: start_y,
                                        vx,
                                        vy,
                                        width: 1,
                                        height: 1,
                                        symbol: attack.bullet_symbol.clone(),
                                        bounces_remaining: 0,
                                    }
                                }
                                AttackType::Bouncing => {
                                    let side = rand::thread_rng().gen_range(0..4);
                                    let (x, y, vx, vy) = match side {
                                        0 => (
                                            // Top
                                            rand::thread_rng()
                                                .gen_range(0.0..self.bullet_board_size.0 as f32),
                                            0.0,
                                            rand::thread_rng().gen_range(
                                                -attack.bullet_speed..=attack.bullet_speed,
                                            ),
                                            attack.bullet_speed,
                                        ),
                                        1 => (
                                            // Bottom
                                            rand::thread_rng()
                                                .gen_range(0.0..self.bullet_board_size.0 as f32),
                                            self.bullet_board_size.1 as f32,
                                            rand::thread_rng().gen_range(
                                                -attack.bullet_speed..=attack.bullet_speed,
                                            ),
                                            -attack.bullet_speed,
                                        ),
                                        2 => (
                                            // Left
                                            0.0,
                                            rand::thread_rng()
                                                .gen_range(0.0..self.bullet_board_size.1 as f32),
                                            attack.bullet_speed,
                                            rand::thread_rng().gen_range(
                                                -attack.bullet_speed..=attack.bullet_speed,
                                            ),
                                        ),
                                        3 => (
                                            // Right
                                            self.bullet_board_size.0 as f32,
                                            rand::thread_rng()
                                                .gen_range(0.0..self.bullet_board_size.1 as f32),
                                            -attack.bullet_speed,
                                            rand::thread_rng().gen_range(
                                                -attack.bullet_speed..=attack.bullet_speed,
                                            ),
                                        ),
                                        _ => unreachable!(),
                                    };
                                    Bullet {
                                        x,
                                        y,
                                        vx,
                                        vy,
                                        width: 1,
                                        height: 1,
                                        symbol: attack.bullet_symbol.clone(),
                                        bounces_remaining: 3,
                                    }
                                }
                                AttackType::Wave => {
                                    // This block is now handled by gun_state
                                    unreachable!("Wave attack should be handled by gun_state");
                                }
                            };
                            self.bullets.push(bullet);
                        }
                    }
                }

                // Update bullet positions and handle bouncing
                for bullet in &mut self.bullets {
                    bullet.x += bullet.vx * delta_time.as_secs_f32();
                    bullet.y += bullet.vy * delta_time.as_secs_f32();

                    // No need for a separate match here, the logic is self-contained.
                    // The _ => {} block was the source of the error.
                    // The bullet logic below already handles the bouncing logic for the different attack types.

                    // Bouncing logic
                    if bullet.bounces_remaining > 0 {
                        let mut bounced = false;
                        if bullet.x <= 0.0
                            || bullet.x >= self.bullet_board_size.0 as f32 - bullet.width as f32
                        {
                            bullet.vx *= -1.0;
                            bounced = true;
                        }
                        if bullet.y <= 0.0
                            || bullet.y >= self.bullet_board_size.1 as f32 - bullet.height as f32
                        {
                            bullet.vy *= -1.0;
                            bounced = true;
                        }
                        if bounced {
                            bullet.bounces_remaining -= 1;
                        }
                    }
                }

                // Remove bullets that are off-screen or have no bounces left
                self.bullets.retain(|b| {
                    let is_off_screen = !(b.x >= -(b.width as f32)
                        && b.x <= self.bullet_board_size.0 as f32 + b.width as f32
                        && b.y >= -(b.height as f32)
                        && b.y <= self.bullet_board_size.1 as f32 + b.height as f32);

                    match self.current_attack.as_ref().map(|a| &a.attack_type) {
                        Some(AttackType::Simple) => !is_off_screen, // Retain if not off screen
                        Some(AttackType::Bouncing) => !is_off_screen && b.bounces_remaining > 0, // Retain if not off screen and still bouncing
                        Some(AttackType::Wave) => !is_off_screen, // Retain if not off screen
                        _ => false,                               // Should not happen
                    }
                });

                let speed = 100.0;
                if *key_states.get(&KeyCode::Up).unwrap_or(&false) {
                    self.player_heart.y -= speed * VERTICAL_SPEED_FACTOR * delta_time.as_secs_f32();
                }
                if *key_states.get(&KeyCode::Down).unwrap_or(&false) {
                    self.player_heart.y += speed * VERTICAL_SPEED_FACTOR * delta_time.as_secs_f32();
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

                // Collision detection
                let heart_rect = ratatui::layout::Rect::new(
                    self.player_heart.x as u16,
                    self.player_heart.y as u16,
                    self.player_heart.width,
                    self.player_heart.height,
                );
                let damage = self.current_attack.as_ref().map_or(0, |a| a.damage);
                self.bullets.retain(|bullet| {
                    let bullet_rect = ratatui::layout::Rect::new(
                        bullet.x as u16,
                        bullet.y as u16,
                        bullet.width,
                        bullet.height,
                    );
                    if heart_rect.intersects(bullet_rect) {
                        if !self.is_flickering {
                            // Only take damage if not flickering
                            self.player_hp = (self.player_hp - damage).max(0);
                            self.is_flickering = true;
                            self.flicker_timer = Instant::now();
                        }
                        false // remove bullet on collision
                    } else {
                        true
                    }
                });

                // Update flicker state
                if self.is_flickering && self.flicker_timer.elapsed() > self.flicker_duration {
                    self.is_flickering = false;
                }

                if self.player_hp <= 0 {
                    self.mode = BattleMode::GameOverTransition;
                    self.game_over_timer = Instant::now();
                }
            }
            BattleMode::GameOverTransition => {
                if self.game_over_timer.elapsed() > Duration::from_secs(3) {
                    self.mode = BattleMode::GameOver;
                }
            }
            _ => {}
        }
    }
}
