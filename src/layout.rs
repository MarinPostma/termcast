#[derive(Debug, Clone)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Rect {
        Rect { x, y, width, height }
    } 

    pub fn right(&self) -> u16 {
        self.x + self.width
    }

    pub fn left(&self) -> u16 {
        self.x
    }

    pub fn top(&self) -> u16 {
        self.y
    }

    pub fn bottom(&self) -> u16 {
        self.y + self.height
    }
}
