use std::time::Duration;

pub const PLAYER_SPEED: u16 = 1;
pub const PLAYER_HORIZONTAL_SPEED: u16 = 2;
pub const FRAME_RATE: u64 = 120;
pub const ANIMATION_FRAME_DURATION: Duration = Duration::from_millis(200);
pub const ESC_HOLD_DURATION: Duration = Duration::from_millis(1000); // 1 second