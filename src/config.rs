use crate::editor::keymap::KeyBinding;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub keymap: Vec<KeyBinding>,
}
