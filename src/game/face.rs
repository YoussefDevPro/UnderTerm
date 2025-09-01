use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use ansi_to_tui::IntoText;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Face {
    pub name: String,
    pub content: String,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Default)]
pub struct FaceManager {
    pub faces: HashMap<String, Face>,
}

impl FaceManager {
    pub fn new() -> Self {
        let mut faces = HashMap::new();
        if let Ok(entries) = fs::read_dir("assets/sprites/faces") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let text = content.as_bytes().into_text().unwrap();
                            let height = text.lines.len() as u16;
                            let mut max_width = 0;
                            for line in text.lines.iter() {
                                let line_width = line.width() as u16;
                                if line_width > max_width {
                                    max_width = line_width;
                                }
                            }
                            faces.insert(
                                name.to_string(),
                                Face {
                                    name: name.to_string(),
                                    content,
                                    width: max_width,
                                    height,
                                },
                            );
                        }
                    }
                }
            }
        }
        FaceManager { faces }
    }
}
