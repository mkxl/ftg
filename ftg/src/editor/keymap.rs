use crate::{editor::command::Command, utils::any::Any};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use serde::{de::Error, Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Context {
    Buffer,
    Search,
}

#[derive(Deserialize)]
pub struct KeyBinding {
    #[serde(deserialize_with = "KeyBinding::deserialize_keys", rename(deserialize = "keys"))]
    events: Vec<Event>,

    #[serde(flatten)]
    command: Command,

    #[serde(default = "KeyBinding::default_contexts")]
    contexts: Vec<Context>,
}

impl KeyBinding {
    const MISSING_KEY_ERROR_MESSAGE: &'static str = "No key was provided";
    const UNKNOWN_KEY_ERROR_MESSAGE: &'static str = "Unknown key was provided";

    // NOTE: each individual event_str must be of the form
    // [ctrl +] [shift +] [alt +] (<special-key> | <single-character>)
    // where <special-key> is one of the special keys listed below
    fn deserialize_key<'a, 'de, D: Deserializer<'de>>(event_str: &'a str) -> Result<Event, D::Error> {
        let mut modifiers = KeyModifiers::NONE;
        let mut substrs = event_str.split('+').peekable();

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

    fn deserialize_keys<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Event>, D::Error> {
        // TODO: figure out how to deserialize to &str
        Vec::deserialize(deserializer)?
            .iter()
            .map(String::as_str)
            .map(Self::deserialize_key::<D>)
            .collect()
    }

    fn default_contexts() -> Vec<Context> {
        std::vec![Context::Buffer]
    }
}

pub struct Keymap {
    value: HashMap<u64, Command>,
}

impl Keymap {
    pub fn new(key_bindings: Vec<KeyBinding>) -> Self {
        let mut value = HashMap::new();

        for key_binding in key_bindings {
            for context in key_binding.contexts {
                let key = Self::key(context, &key_binding.events);

                value.insert(key, key_binding.command.clone());
            }
        }

        Self { value }
    }

    fn key(context: Context, events: &[Event]) -> u64 {
        (context, events).hashcode()
    }

    pub fn get<'a>(&'a self, context: Context, events: &'a [Event]) -> (Context, Result<&'a Command, &'a [Event]>) {
        let key = Self::key(context, events);

        match self.value.get(&key) {
            Some(command) => (context, command.ok()),
            None => (context, events.err()),
        }
    }
}
