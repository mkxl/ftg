use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(tag = "command", content = "args", rename_all = "snake_case")]
pub enum Command {
    Quit,
    MoveForward,
    MoveBackward,
    MoveUp,
    MoveDown,
    ScrollRight(usize),
    ScrollLeft(usize),
    ScrollUp(usize),
    ScrollDown(usize),
    Delete,
    ToggleStatusBar,
    ToggleGutter,
    ToggleTabs,
    ToggleLogs,
    NextTab,
    PrevTab,
}
