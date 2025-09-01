use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedSprite {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub ansi_content: String,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleZone {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl BattleZone {
    pub fn to_rect(&self) -> ratatui::layout::Rect {
        ratatui::layout::Rect::new(
            self.x as u16,
            self.y as u16,
            self.width as u16,
            self.height as u16,
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum MapKind {
    #[serde(rename = "walls")]
    Walls,
    #[serde(rename = "objects")]
    Objects,
    #[serde(rename = "empty")]
    Empty,
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
        MapKind::Walls
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MapData {
    pub map_name: String,
    pub player_spawn: (u32, u32),
    pub walls: Vec<(u32, u32)>,
    #[serde(default)]
    pub select_object_boxes: Vec<SelectObjectBox>,
    #[serde(default)]
    pub placed_sprites: Vec<PlacedSprite>,
    #[serde(default)]
    pub kind: MapKind,
    #[serde(default)]
    pub battle_zones: Vec<BattleZone>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    TeleportPlayer {
        map_row: i32,
        map_col: i32,
        dest_x: u32,
        dest_y: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectObjectBox {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub messages: Vec<String>,
    #[serde(default)]
    pub events: Vec<Event>,
}

impl SelectObjectBox {
    pub fn to_rect(&self) -> ratatui::layout::Rect {
        ratatui::layout::Rect::new(
            self.x as u16,
            self.y as u16,
            self.width as u16,
            self.height as u16,
        )
    }
}

use ansi_to_tui::IntoText;

#[derive(Debug, Clone, Default)]
pub struct Map {
    pub name: String,
    pub ansi_sprite: String,
    pub walls: Vec<(u32, u32)>,
    pub player_spawn: (u32, u32),
    pub select_object_boxes: Vec<SelectObjectBox>,
    pub placed_sprites: Vec<PlacedSprite>,
    pub kind: MapKind,
    pub battle_zones: Vec<BattleZone>,
    pub width: u16,
    pub height: u16,
}

impl Map {
    pub fn load(map_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let base_path = Path::new("/home/youssef/UnderTerm/assets/map").join(map_name);

        // load data
        let data_path = base_path.join("data.json");
        let data_content = fs::read_to_string(&data_path)?;
        let map_data: MapData = serde_json::from_str(&data_content)?;

        // lood sprite
        let sprite_path = base_path.join("sprite.ans");
        let ansi_sprite = fs::read_to_string(&sprite_path)?;

        // map dimension
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
            placed_sprites: map_data.placed_sprites,
            kind: map_data.kind,
            battle_zones: map_data.battle_zones,

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
            placed_sprites: self.placed_sprites.clone(),
            kind: self.kind.clone(),
            battle_zones: self.battle_zones.clone(),
        };

        let serialized = serde_json::to_string_pretty(&map_data)?;
        fs::write(&data_path, serialized)?;
        Ok(())
    }

    pub fn add_select_object_box(&mut self, select_object_box: SelectObjectBox) {
        self.select_object_boxes.push(select_object_box);
    }

    pub fn add_placed_sprite(&mut self, placed_sprite: PlacedSprite) {
        self.placed_sprites.push(placed_sprite);
    }

    pub fn add_battle_zone(&mut self, battle_zone: BattleZone) {
        self.battle_zones.push(battle_zone);
    }

    pub fn create_new(map_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let base_path = Path::new("assets/map").join(map_name);
        fs::create_dir_all(&base_path)?;

        let data_path = base_path.join("data.json");
        let sprite_path = base_path.join("sprite.ans");

        let map_data = MapData {
            map_name: map_name.to_string(),
            player_spawn: (10, 10),
            walls: vec![],
            select_object_boxes: vec![],
            placed_sprites: vec![],
            kind: MapKind::Empty,
            battle_zones: vec![],
        };

        let serialized = serde_json::to_string_pretty(&map_data)?;
        fs::write(&data_path, serialized)?;

        fs::write(&sprite_path, "")?;

        Ok(Map {
            name: map_name.to_string(),
            ansi_sprite: "".to_string(),
            walls: vec![],
            player_spawn: (10, 10),
            select_object_boxes: vec![],
            placed_sprites: vec![],
            kind: MapKind::Empty,
            battle_zones: vec![],
            width: 0,
            height: 0,
        })
    }
}
