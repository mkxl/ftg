use crate::command::Command;
use mkutils::{KeyMap, Rgb};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ColorSpec {
    pub fg: Rgb,
    pub bg: Rgb,
}

#[derive(Deserialize)]
pub struct ColorScheme {
    pub main: ColorSpec,
}

#[derive(Deserialize)]
pub struct Theme {
    pub buffer_border_lighten: f32,
    pub buffer_title_lighten: f32,
}

#[derive(Deserialize)]
pub struct Config {
    pub color_scheme: ColorScheme,
    pub theme: Theme,
    pub key_map: KeyMap<Command>,
}
