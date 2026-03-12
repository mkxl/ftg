use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(tag = "command", content = "args", rename_all = "snake_case")]
pub enum Command {
    Quit,
    MoveForward,
    MoveBackward,
}
