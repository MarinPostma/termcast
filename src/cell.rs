use crate::style::Style;

#[derive(Clone)]
pub struct Cell {
    pub style: Style,
    pub symbol: char,
}

impl Cell {
    pub fn set_symbol(&mut self, symbol: char) -> &mut Self {
        self.symbol = symbol;
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.symbol = ' ';
        self.style.reset();
        self
    }
}

impl Default for Cell {
    fn default() -> Cell {
        Cell {
            symbol: ' ',
            style: Style::default(),
        }
    }
}
