use crate::style::Style;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Cell {
    pub style: Style,
    pub symbol: char,
}

impl Cell {
    #[inline]
    pub fn set_symbol(&mut self, symbol: char) -> &mut Self {
        self.symbol = symbol;
        self
    }

    #[inline]
    pub fn set_style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    #[inline]
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
