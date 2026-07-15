use crate::command::Command;
use mkutils::{KeyMap, Rgb, Utils};
use ratatui::style::Style;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
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
pub struct TabColorScheme {
    pub tab: ColorSpec,
    pub filepath: ColorSpec,
}

#[derive(Deserialize)]
pub struct TabsColorScheme {
    pub active: TabColorScheme,
    pub inactive: TabColorScheme,
}

impl TabsColorScheme {
    pub fn get(&self, active: bool) -> &TabColorScheme {
        active.if_else(self.active.ref_immut(), self.inactive.ref_immut())
    }
}

#[derive(Deserialize)]
pub struct ColorScheme {
    pub main: ColorSpec,
    pub status_bar: ColorSpec,
    pub tabs: TabsColorScheme,
}

#[derive(Deserialize)]
pub struct Theme {
    pub gutter_lighten: f32,
    pub gutter_padding: usize,
    pub tabs_width_percentage: u16,
}

#[derive(Deserialize)]
pub struct Config {
    pub color_scheme: ColorScheme,
    pub theme: Theme,
    pub key_map: KeyMap<Command>,
}

impl Config {
    pub fn gutter_fg_rgb(&self) -> Rgb {
        self.color_scheme.main.bg.lighten_fixed(self.theme.gutter_lighten)
    }
}
