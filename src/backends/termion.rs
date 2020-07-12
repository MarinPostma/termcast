use std::io;
use std::io::Write;
use std::fmt;

use super::Backend;

use crate::cell::Cell;
use crate::style;

pub struct TermionBackend<W: Write>(W);

impl<W: Write> Write for TermionBackend<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }

}

impl<W: Write> TermionBackend<W> {
    pub fn new(writer: W) -> Self {
        Self(writer)
    }

}

impl<W: Write> Backend for TermionBackend<W> {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
        where
            I: Iterator<Item = (u16, u16, &'a Cell)> {
                use std::fmt::Write;

                let mut str_buf = String::new();
                for cell in content {
                    write!(str_buf, "{}", termion::cursor::Goto(cell.0, cell.1)).unwrap();
                    write!(str_buf, "{}", Bg(cell.2.style.bg)).unwrap();
                    write!(str_buf, "{}", Fg(cell.2.style.fg)).unwrap();
                    write!(str_buf, "{}", cell.2.symbol).unwrap();
                }
                write!(self.0, "{}{}{}", str_buf, Bg(style::Color::Reset), Fg(style::Color::Reset))?;
                self.0.flush()?;
                Ok(())
    }

    fn clear(&mut self) -> Result<(), io::Error> {
        write!(self.0, "{}", termion::clear::All)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        write!(self.0, "{}", termion::cursor::Hide)
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        write!(self.0, "{}", termion::cursor::Show)
    }

    fn cursor_goto(&mut self, cols: u16, rows: u16) -> io::Result<()> {
        write!(self.0, "{}", termion::cursor::Goto(cols, rows))
    }
}

struct Bg(style::Color);
struct Fg(style::Color);

impl fmt::Display for Fg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use termion::color::Color;
        match self.0 {
            style::Color::Reset => termion::color::Reset.write_fg(f),
            style::Color::Black => termion::color::Black.write_fg(f),
            style::Color::Red => termion::color::Red.write_fg(f),
            style::Color::Green => termion::color::Green.write_fg(f),
            style::Color::Yellow => termion::color::Yellow.write_fg(f),
            style::Color::Blue => termion::color::Blue.write_fg(f),
            style::Color::Magenta => termion::color::Magenta.write_fg(f),
            style::Color::Cyan => termion::color::Cyan.write_fg(f),
            style::Color::Gray => termion::color::White.write_fg(f),
            style::Color::DarkGray => termion::color::LightBlack.write_fg(f),
            style::Color::LightRed => termion::color::LightRed.write_fg(f),
            style::Color::LightGreen => termion::color::LightGreen.write_fg(f),
            style::Color::LightBlue => termion::color::LightBlue.write_fg(f),
            style::Color::LightYellow => termion::color::LightYellow.write_fg(f),
            style::Color::LightMagenta => termion::color::LightMagenta.write_fg(f),
            style::Color::LightCyan => termion::color::LightCyan.write_fg(f),
            style::Color::White => termion::color::LightWhite.write_fg(f),
            style::Color::Indexed(i) => termion::color::AnsiValue(i).write_fg(f),
            style::Color::Rgb(r, g, b) => termion::color::Rgb(r, g, b).write_fg(f),
        }
    }
}

impl fmt::Display for Bg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use termion::color::Color;
        match self.0 {
            style::Color::Reset => termion::color::Reset.write_bg(f),
            style::Color::Black => termion::color::Black.write_bg(f),
            style::Color::Red => termion::color::Red.write_bg(f),
            style::Color::Green => termion::color::Green.write_bg(f),
            style::Color::Yellow => termion::color::Yellow.write_bg(f),
            style::Color::Blue => termion::color::Blue.write_bg(f),
            style::Color::Magenta => termion::color::Magenta.write_bg(f),
            style::Color::Cyan => termion::color::Cyan.write_bg(f),
            style::Color::Gray => termion::color::White.write_bg(f),
            style::Color::DarkGray => termion::color::LightBlack.write_bg(f),
            style::Color::LightRed => termion::color::LightRed.write_bg(f),
            style::Color::LightGreen => termion::color::LightGreen.write_bg(f),
            style::Color::LightBlue => termion::color::LightBlue.write_bg(f),
            style::Color::LightYellow => termion::color::LightYellow.write_bg(f),
            style::Color::LightMagenta => termion::color::LightMagenta.write_bg(f),
            style::Color::LightCyan => termion::color::LightCyan.write_bg(f),
            style::Color::White => termion::color::LightWhite.write_bg(f),
            style::Color::Indexed(i) => termion::color::AnsiValue(i).write_bg(f),
            style::Color::Rgb(r, g, b) => termion::color::Rgb(r, g, b).write_bg(f),
        }
    }
}
