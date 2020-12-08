#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub modifier: Modifier,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fg(&mut self, color: Color) -> &mut Self {
        self.fg = color;
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.bg = Color::default();
        self.fg = Color::default();
        self.modifier = Modifier::default();
        self
    }

    pub fn bg(&mut self, color: Color) -> &mut Self {
        self.bg = color;
        self
    }

    pub fn set_italic(&mut self) -> &mut Self {
        self.modifier |= Modifier::ITALIC;
        self
    }

    pub fn unset_italic(&mut self) -> &mut Self {
        self.modifier &= !Modifier::ITALIC;
        self
    }

    pub fn set_underline(&mut self) -> &mut Self {
        self.modifier |= Modifier::UNDERLINED;
        self
    }

    pub fn unset_underline(&mut self) -> &mut Self {
        self.modifier &= !Modifier::UNDERLINED;
        self
    }

    pub fn set_bold(&mut self) -> &mut Self {
        self.modifier |= Modifier::BOLD;
        self
    }

    pub fn unset_bold(&mut self) -> &mut Self {
        self.modifier &= !Modifier::BOLD;
        self
    }

    pub fn set_color_3bits(&mut self, color: usize) {
        match color {
            // fg color
            30 => self.fg = Color::Black,
            31 => self.fg = Color::Red,
            32 => self.fg = Color::Green,
            33 => self.fg = Color::Yellow,
            34 => self.fg = Color::Blue,
            35 => self.fg = Color::Magenta,
            36 => self.fg = Color::Cyan,
            37 => self.fg = Color::White,

            // bright fg color
            90 => self.fg = { self.set_bold(); Color::Black },
            91 => self.fg = { self.set_bold(); Color::Red },
            92 => self.fg = { self.set_bold(); Color::Green },
            93 => self.fg = { self.set_bold(); Color::Yellow },
            94 => self.fg = { self.set_bold(); Color::Blue },
            95 => self.fg = { self.set_bold(); Color::Magenta },
            96 => self.fg = { self.set_bold(); Color::Cyan },
            97 => self.fg = { self.set_bold(); Color::White },

            // bg color
            40 => self.bg = Color::Black,
            41 => self.bg = Color::Red,
            42 => self.bg = Color::Green,
            43 => self.bg = Color::Yellow,
            44 => self.bg = Color::Blue,
            45 => self.bg = Color::Magenta,
            46 => self.bg = Color::Cyan,
            47 => self.bg = Color::White,

            // bright bg color
            100 => self.bg = { self.set_bold(); Color::Black },
            101 => self.bg = { self.set_bold(); Color::Red },
            102 => self.bg = { self.set_bold(); Color::Green },
            103 => self.bg = { self.set_bold(); Color::Yellow },
            104 => self.bg = { self.set_bold(); Color::Blue },
            105 => self.bg = { self.set_bold(); Color::Magenta },
            106 => self.bg = { self.set_bold(); Color::Cyan },
            107 => self.bg = { self.set_bold(); Color::White },

            _ => unreachable!()
        }
    }
}

bitflags! {
    #[derive(Default)]

    pub struct Modifier: u16 {
        const BOLD              = 0b0000_0000_0001;
        const DIM               = 0b0000_0000_0010;
        const ITALIC            = 0b0000_0000_0100;
        const UNDERLINED        = 0b0000_0000_1000;
        const SLOW_BLINK        = 0b0000_0001_0000;
        const RAPID_BLINK       = 0b0000_0010_0000;
        const REVERSED          = 0b0000_0100_0000;
        const HIDDEN            = 0b0000_1000_0000;
        const CROSSED_OUT       = 0b0001_0000_0000;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

impl Default for Color {
    fn default() -> Self {
        Color::Reset
    }
}
