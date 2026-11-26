use ratatui::widgets::{Block, BorderType};

#[mkutils::empty(pub)]
impl Ui {
    pub const fn rounded_bordered_block() -> Block<'static> {
        Block::bordered().border_type(BorderType::Rounded)
    }
}
