use crate::utils::any::Any;
use ratatui::style::Color;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct Spec {
    #[serde(deserialize_with = "Spec::deserialize_color")]
    pub fg: Color,

    #[serde(deserialize_with = "Spec::deserialize_color")]
    pub bg: Color,
}

impl Spec {
    fn deserialize_color<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Color, D::Error> {
        let color = Color::deserialize(deserializer)?;
        let Color::Rgb(r, g, b) = color else { return color.ok() };
        let index = ansi_colours::ansi256_from_rgb((r, g, b));

        Color::Indexed(index).ok()
    }
}

#[derive(Deserialize)]
pub struct Tabs {
    pub dots: Spec,
    pub active: Spec,
    pub primary: Spec,
    pub secondary: Spec,
}

#[derive(Deserialize)]
pub struct ColorScheme {
    pub title: Spec,
    pub tabs: Tabs,
    pub buffer: Spec,
}
