#[derive(Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}
