use crate::editor::{color_scheme::ColorScheme, keymap::KeyBinding};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub color_scheme: ColorScheme,
    pub keymap: Vec<KeyBinding>,
}
