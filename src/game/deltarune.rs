use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deltarune {
    pub level: u8,
}

impl Deltarune {
    pub fn new() -> Self {
        Self { level: 0 }
    }

    pub fn increase(&mut self) {
        if self.level < 100 {
            self.level += 1;
        }
    }

    pub fn decrease(&mut self) {
        if self.level > 0 {
            self.level -= 1;
        }
    }
}