use crate::utils::any::Any;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use serde::{de::Error, Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

#[derive(Deserialize)]
pub struct KeyBinding {
    #[serde(deserialize_with = "KeyBinding::deserialize_events")]
    events: Vec<Event>,

    command: Command,
}

impl KeyBinding {
    const MISSING_KEY_ERROR_MESSAGE: &'static str = "No key was provided";
    const UNKNOWN_KEY_ERROR_MESSAGE: &'static str = "Unknown key was provided";

    fn deserialize_event<'de, D: Deserializer<'de>>(text: &str) -> Result<Event, D::Error> {
        let mut modifiers = KeyModifiers::NONE;
        let mut substrs = text.split('+').peekable();

        if let Some(&"ctrl") = substrs.peek() {
            modifiers.insert(KeyModifiers::CONTROL);
            substrs.next();
        }

        if let Some(&"shift") = substrs.peek() {
            modifiers.insert(KeyModifiers::SHIFT);
            substrs.next();
        }

        if let Some(&"alt") = substrs.peek() {
            modifiers.insert(KeyModifiers::ALT);
            substrs.next();
        }

        let Some(substr) = substrs.next() else {
            return D::Error::custom(Self::MISSING_KEY_ERROR_MESSAGE).err();
        };
        let code = match substr {
            "backspace" => KeyCode::Backspace,
            "enter" => KeyCode::Enter,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "home" => KeyCode::Home,
            "tab" => KeyCode::Tab,
            "delete" => KeyCode::Delete,
            "esc" => KeyCode::Esc,
            _ => {
                let mut chars = substr.chars();
                let Some(chr) = chars.next() else {
                    return D::Error::custom(Self::MISSING_KEY_ERROR_MESSAGE).err();
                };
                let None = chars.next() else {
                    return D::Error::custom(Self::UNKNOWN_KEY_ERROR_MESSAGE).err();
                };

                KeyCode::Char(chr)
            }
        };
        let key_event = KeyEvent::new(code, modifiers);
        let event = Event::Key(key_event);

        event.ok()
    }

    fn deserialize_events<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Event>, D::Error> {
        let texts = <Vec<String> as Deserialize>::deserialize(deserializer)?;
        let events: Vec<Event> = texts
            .iter()
            .map(String::as_str)
            .map(Self::deserialize_event::<'de, D>)
            .collect::<Result<_, _>>()?;

        events.ok()
    }
}

pub struct Keymap {
    value: HashMap<Vec<Event>, Command>,
}

impl Keymap {
    pub fn new(key_bindings: Vec<KeyBinding>) -> Self {
        let value = key_bindings
            .into_iter()
            .map(|key_binding| (key_binding.events, key_binding.command))
            .collect();

        Self { value }
    }

    pub fn get<'a>(&'a self, events: &'a [Event]) -> Result<&'a Command, &'a [Event]> {
        match self.value.get(events) {
            Some(key_binding) => key_binding.ok(),
            None => events.err(),
        }
    }
}
