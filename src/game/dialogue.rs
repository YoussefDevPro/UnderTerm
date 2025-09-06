use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dialogue {
    pub enemy_ansi_path: String,
    pub face_ansi_path: String,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogueManager {
    pub dialogues: Vec<Dialogue>,
    pub current_dialogue_index: usize,
    pub animated_text: String,
    pub text_animation_finished: bool,
    pub visible_text_len: usize,
}

impl DialogueManager {
    pub fn new() -> Self {
        let dialogues = Self::load_dialogues().unwrap_or_default();
        DialogueManager {
            dialogues,
            ..Default::default()
        }
    }

    fn load_dialogues() -> Result<Vec<Dialogue>, Box<dyn std::error::Error>> {
        let dialogues_content = include_str!("../../dialogues.json");
        let dialogues: Vec<Dialogue> = serde_json::from_str(dialogues_content)?;
        Ok(dialogues)
    }

    pub fn current_dialogue(&self) -> Option<&Dialogue> {
        self.dialogues.get(self.current_dialogue_index)
    }

    pub fn advance_dialogue(&mut self) {
        if self.current_dialogue_index < self.dialogues.len() - 1 {
            self.current_dialogue_index += 1;
            self.animated_text.clear();
            self.text_animation_finished = false;
            self.visible_text_len = 0;
        } else {
            // No more dialogues
        }
    }

    pub fn skip_animation(&mut self) {
        if let Some(dialogue) = self.current_dialogue() {
            self.visible_text_len = dialogue.text.chars().count();
            self.text_animation_finished = true;
        }
    }
}
