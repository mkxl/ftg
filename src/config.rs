use crate::command::Command;
use mkutils::KeyMap;
use ratatui::style::Color;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ColorSpec {
    pub fg: Color,
    pub bg: Color,
}

#[derive(Deserialize)]
pub struct Theme {
    pub main: ColorSpec,
}

#[derive(Deserialize)]
pub struct Config {
    pub theme: Theme,
    pub key_map: KeyMap<Command>,
}
