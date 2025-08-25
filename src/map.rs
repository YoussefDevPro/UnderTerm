use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct MapData {
    pub map_name: String,
    pub player_spawn: (u32, u32),
    pub walls: Vec<(u32, u32)>,
}

use ansi_to_tui::IntoText;

#[derive(Debug, Clone, Default)]
pub struct Map {
    pub name: String,
    pub ansi_sprite: String,
    pub walls: Vec<(u32, u32)>,
    pub player_spawn: (u32, u32),
    pub width: u16,
    pub height: u16,
}

impl Map {
    pub fn load(map_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let base_path = Path::new("/home/youssef/UnderTerm/assets/map").join(map_name);

        // Load map data
        let data_path = base_path.join("data.json");
        let data_content = fs::read_to_string(&data_path)?;
        let map_data: MapData = serde_json::from_str(&data_content)?;

        // Load ANSI sprite
        let sprite_path = base_path.join("sprite.ans");
        let ansi_sprite = fs::read_to_string(&sprite_path)?;

        // Calculate map dimensions
        let map_text_for_dimensions = ansi_sprite.as_bytes().into_text().unwrap();

        let height = {
            let mut actual_height = 0;
            for (i, line) in map_text_for_dimensions.lines.iter().enumerate() {
                if !line.spans.iter().all(|span| span.content.trim().is_empty()) {
                    actual_height = i + 1;
                }
            }
            actual_height as u16
        };

        let mut width = 0;
        for line in map_text_for_dimensions.lines.iter() {
            let line_width = line.width() as u16;
            if line_width > width {
                width = line_width;
            }
        }

        Ok(Map {
            name: map_data.map_name,
            ansi_sprite,
            walls: map_data.walls,
            player_spawn: map_data.player_spawn,
            width,
            height,
        })
    }

    pub fn toggle_wall(&mut self, x: u32, y: u32) {
        let pos = (x, y);
        if let Some(index) = self.walls.iter().position(|&p| p == pos) {
            // Wall exists, remove it
            self.walls.remove(index);
        } else {
            // Wall does not exist, add it
            self.walls.push(pos);
        }
    }
}
