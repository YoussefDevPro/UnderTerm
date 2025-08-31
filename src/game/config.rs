use std::time::Duration;

pub const PLAYER_SPEED: f32 = 40.0;
pub const PLAYER_HORIZONTAL_SPEED: f32 = 80.0;
pub const DEBUG_MOVEMENT_FRAME_INTERVAL: u8 = 5;
pub const FRAME_RATE: u64 = 120;
pub const ANIMATION_FRAME_DURATION: Duration = Duration::from_millis(200);
pub const PLAYER_INTERACTION_BOX_WIDTH: u16 = 30;
pub const PLAYER_INTERACTION_BOX_HEIGHT: u16 = 20;
pub const TELEPORT_COOLDOWN_DURATION: Duration = Duration::from_millis(500);
