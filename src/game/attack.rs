use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub vx: f32, // velocity x
    pub vy: f32, // velocity y
    pub width: u16,
    pub height: u16,
    pub symbol: String,
    pub bounces_remaining: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackType {
    Simple,
    Bouncing,
    Wave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attack {
    pub attack_type: AttackType,
    pub duration: Duration,
    pub damage: i32,
    pub bullet_speed: f32,
    pub bullet_symbol: String,
    pub spawn_rate: f64,
    pub wave_amplitude: Option<f32>,
    pub wave_frequency: Option<f32>,
}
