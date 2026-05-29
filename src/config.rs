use crate::command::Command;
use mkutils::{KeyMap, Rgb};
use ratatui::style::Style;
use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
pub struct ColorSpec {
    pub fg: Rgb,
    pub bg: Rgb,
}

impl From<ColorSpec> for Style {
    fn from(color_spec: ColorSpec) -> Self {
        Self::default().fg(color_spec.fg.into()).bg(color_spec.bg.into())
    }
}

#[derive(Deserialize)]
pub struct ColorScheme {
    pub main: ColorSpec,
    pub status_bar: ColorSpec,
    pub side_tab: ColorSpec,
    pub side_tab_path: ColorSpec,
    pub side_tab_active: ColorSpec,
    pub side_tab_path_active: ColorSpec,
}

#[derive(Deserialize)]
pub struct Theme {
    pub gutter_lighten: f32,
    pub gutter_padding: usize,
    pub side_tab_percentage: u16,
}

#[derive(Deserialize)]
pub struct Config {
    pub color_scheme: ColorScheme,
    pub theme: Theme,
    pub key_map: KeyMap<Command>,
}

impl Config {
    pub fn gutter_fg(&self) -> Rgb {
        self.color_scheme.main.bg.lighten_fixed(self.theme.gutter_lighten)
    }
}
