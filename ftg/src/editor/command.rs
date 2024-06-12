use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "command", content = "args")]
pub enum Command {
    Close,
    MoveUp { count: usize },
    MoveDown { count: usize },
    MoveLeft,
    MoveRight,
    NextView,
    PreviousView,
    Quit,
    Save,
    Search,
    Submit,
}
