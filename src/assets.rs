#[macro_export]
macro_rules! load_sprite_asset_str {
    ($path:expr) => {
        match $path {
            "assets/sprites/animation/0.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/animation/0.ans"
            )),
            "assets/sprites/animation/1.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/animation/1.ans"
            )),
            "assets/sprites/animation/2.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/animation/2.ans"
            )),
            "assets/sprites/animation/3.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/animation/3.ans"
            )),
            "assets/sprites/animation/4.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/animation/4.ans"
            )),
            "assets/sprites/animation/5.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/animation/5.ans"
            )),
            "assets/sprites/faces/face_3.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/faces/face_3.ans"
            )),
            "assets/sprites/faces/face_determined_smile.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/faces/face_determined_smile.ans"
            )),
            "assets/sprites/faces/face_determined.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/faces/face_determined.ans"
            )),
            "assets/sprites/faces/face_hehehe.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/faces/face_hehehe.ans"
            )),
            "assets/sprites/faces/face_huh.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/faces/face_huh.ans"
            )),
            "assets/sprites/faces/face_meh.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/faces/face_meh.ans"
            )),
            "assets/sprites/faces/face_neutral.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/faces/face_neutral.ans"
            )),
            "assets/sprites/faces/face_sight.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/faces/face_sight.ans"
            )),
            "assets/sprites/faces/face_smile.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/faces/face_smile.ans"
            )),
            "assets/sprites/enemy/not_a_placeholder/battle_3.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/enemy/not_a_placeholder/battle_3.ans"
            )),
            "assets/sprites/enemy/not_a_placeholder/battle_hehehe.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/enemy/not_a_placeholder/battle_hehehe.ans"
            )),
            "assets/sprites/enemy/not_a_placeholder/battle_meh.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/enemy/not_a_placeholder/battle_meh.ans"
            )),
            "assets/sprites/enemy/not_a_placeholder/battle_more_neutral.ans" => {
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/sprites/enemy/not_a_placeholder/battle_more_neutral.ans"
                ))
            }
            "assets/sprites/enemy/not_a_placeholder/battle_neutral.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/enemy/not_a_placeholder/battle_neutral.ans"
            )),
            "assets/sprites/enemy/not_a_placeholder/battle_smile.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/enemy/not_a_placeholder/battle_smile.ans"
            )),
            "assets/sprites/ME/idle/insanly_dead.ans" => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/sprites/ME/idle/insanly_dead.ans"
            )),
            _ => "",
        }
    };
}
#[macro_export]
macro_rules! load_map_asset_str {
    ($map_name:expr, $file_name:expr) => {
        match ($map_name, $file_name) {
            ("map_0_0", "data.json") => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/map/map_0_0/data.json"
            )),
            ("map_0_0", "sprite.ans") => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/map/map_0_0/sprite.ans"
            )),
            ("map_1_2", "data.json") => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/map/map_1_2/data.json"
            )),
            ("map_1_2", "sprite.ans") => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/map/map_1_2/sprite.ans"
            )),
            _ => "",
        }
    };
}
