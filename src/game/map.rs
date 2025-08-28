
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum MapKind {
    #[serde(rename = "walls")]
    Walls,
    #[serde(rename = "objects")]
    Objects,
    #[serde(rename = "empty")]
    Empty,
    // Add more kinds as needed
}

impl MapKind {
    pub fn next(&self) -> Self {
        match self {
            MapKind::Walls => MapKind::Objects,
            MapKind::Objects => MapKind::Empty,
            MapKind::Empty => MapKind::Walls,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            MapKind::Walls => MapKind::Empty,
            MapKind::Objects => MapKind::Walls,
            MapKind::Empty => MapKind::Objects,
        }
    }
}

impl Default for MapKind {
    fn default() -> Self {
        MapKind::Walls // Default to Walls if not specified
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MapData {
    pub map_name: String,
    pub player_spawn: (u32, u32),
    pub walls: Vec<(u32, u32)>,
    #[serde(default)]
    pub select_object_boxes: Vec<SelectObjectBox>,
    #[serde(default)] // Use default if kind is not specified in JSON
    pub kind: MapKind,
    
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    TeleportPlayer {
        x: u32,
        y: u32,
        map_row: i32,
        map_col: i32,
    },
    // Add more event types as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectObjectBox {
    pub id: u32,
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
    pub messages: Vec<String>,
    #[serde(default)] // Use default if events are not specified
    pub events: Vec<Event>, // Added
}

use ansi_to_tui::IntoText;

#[derive(Debug, Clone, Default)]
pub struct Map {
    pub name: String,
    pub ansi_sprite: String,
    pub walls: Vec<(u32, u32)>,
    pub player_spawn: (u32, u32),
    pub select_object_boxes: Vec<SelectObjectBox>, // Added
    pub kind: MapKind,                             // Added
    
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
            select_object_boxes: map_data.select_object_boxes,
            kind: map_data.kind, // Add this line
            
            width,
            height,
        })
    }

    pub fn toggle_wall(&mut self, x: u32, y: u32) {
        let pos = (x, y);
        if let Some(index) = self.walls.iter().position(|&p| p == pos) {
            self.walls.remove(index);
        } else {
            self.walls.push(pos);
        }
    }

    pub fn save_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let base_path = Path::new("/home/youssef/UnderTerm/assets/map").join(&self.name);
        let data_path = base_path.join("data.json");

        let map_data = MapData {
            map_name: self.name.clone(),
            player_spawn: self.player_spawn,
            walls: self.walls.clone(),
            select_object_boxes: self.select_object_boxes.clone(),
            kind: self.kind.clone(),
            
        };

        let serialized = serde_json::to_string_pretty(&map_data)?;
        fs::write(&data_path, serialized)?;
        Ok(())
    }

    pub fn add_select_object_box(&mut self, select_object_box: SelectObjectBox) {
        self.select_object_boxes.push(select_object_box);
    }

    

    pub fn create_new(map_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let base_path = Path::new("/home/youssef/UnderTerm/assets/map").join(map_name);
        fs::create_dir_all(&base_path)?;

        let data_path = base_path.join("data.json");
        let sprite_path = base_path.join("sprite.ans");

        // Create a default map data
        let map_data = MapData {
            map_name: map_name.to_string(),
            player_spawn: (10, 10), // Default spawn point
            walls: vec![],
            select_object_boxes: vec![],
            kind: MapKind::Empty,
            teleport_zones: vec![],
        };

        let serialized = serde_json::to_string_pretty(&map_data)?;
        fs::write(&data_path, serialized)?;

        // Create an empty sprite file
        fs::write(&sprite_path, "")?;

        Ok(Map {
            name: map_name.to_string(),
            ansi_sprite: "".to_string(),
            walls: vec![],
            player_spawn: (10, 10),
            select_object_boxes: vec![],
            kind: MapKind::Empty,
            teleport_zones: vec![],
            width: 0,
            height: 0,
        })
    }
}
