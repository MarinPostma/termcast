mod termion;

use std::io;
use std::io::Write;

use crate::cell::Cell;

pub use self::termion::TermionBackend;

pub trait Backend: Write {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
        where
            I: Iterator<Item = (u16, u16, &'a Cell)>;

    fn clear(&mut self) -> io::Result<()>;

    fn hide_cursor(&mut self) -> io::Result<()>;

    fn show_cursor(&mut self) -> io::Result<()>;

    fn cursor_goto(&mut self, cols: u16, rows: u16) -> io::Result<()>;
}
