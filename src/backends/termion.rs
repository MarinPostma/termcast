use std::fmt;
use std::fmt::Write;

use async_trait::async_trait;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::io;

use super::Backend;
use crate::cell::Cell;
use crate::style;

pub struct TermionBackend<W> {
    writer: W,
    buffer: String,
}

impl<W> TermionBackend<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: String::new(),
        }
    }
}

#[async_trait(?Send)]
impl<W: AsyncWrite + Unpin> Backend for TermionBackend<W> {
    async fn draw<I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, Cell)> + Sync + Send {

            for cell in content {
                write!(self.buffer, "{}", termion::cursor::Goto(cell.0, cell.1)).unwrap();
                write!(self.buffer, "{}", Bg(cell.2.style.bg)).unwrap();
                write!(self.buffer, "{}", Fg(cell.2.style.fg)).unwrap();
                write!(self.buffer, "{}", cell.2.symbol).unwrap();
                write!(self.buffer, "{}", Bg(style::Color::Reset)).unwrap();
                write!(self.buffer, "{}", Fg(style::Color::Reset)).unwrap();
            }
            self.writer.write_all(&self.buffer.as_bytes()).await?;
            self.writer.flush().await?;
            self.buffer.clear();
            Ok(())
    }

    async fn clear(&mut self) -> Result<(), io::Error> {
        write!(self.buffer, "{}", termion::clear::All).unwrap();
        self.writer.write_all(&self.buffer.as_bytes()).await?;
        self.writer.flush().await?;
        self.buffer.clear();
        Ok(())
    }

    async fn hide_cursor(&mut self) -> io::Result<()> {
        write!(self.buffer, "{}", termion::cursor::Hide).unwrap();
        self.writer.write_all(&self.buffer.as_bytes()).await?;
        self.writer.flush().await?;
        self.buffer.clear();
        Ok(())
    }

    async fn show_cursor(&mut self) -> io::Result<()> {
        write!(self.buffer, "{}", termion::cursor::Show).unwrap();
        self.writer.write_all(&self.buffer.as_bytes()).await?;
        self.writer.flush().await?;
        self.buffer.clear();
        Ok(())
    }

    async fn cursor_goto(&mut self, cols: u16, rows: u16) -> io::Result<()> {
        write!(self.buffer, "{}", termion::cursor::Goto(cols, rows)).unwrap();
        self.writer.write_all(&self.buffer.as_bytes()).await?;
        self.writer.flush().await?;
        self.buffer.clear();
        Ok(())
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
