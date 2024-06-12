use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "command", content = "args")]
pub enum Command {
    Close,
    NextView,
    PreviousView,
    Quit,
    Save,
    ScrollDown { count: usize },
    ScrollLeft { count: usize },
    ScrollRight { count: usize },
    ScrollUp { count: usize },
    Search,
    Submit,
}
