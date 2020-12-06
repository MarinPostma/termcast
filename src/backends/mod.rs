mod termion;

use async_trait::async_trait;
use tokio::io;

use crate::cell::Cell;

pub use self::termion::TermionBackend;

#[async_trait(?Send)]
pub trait Backend {
    async fn draw<I>(&mut self, content: I) -> io::Result<()>
        where
            I: Iterator<Item = (u16, u16, Cell)> + Sync + Send;

    async fn clear(&mut self) -> io::Result<()>;

    async fn hide_cursor(&mut self) -> io::Result<()>;

    async fn show_cursor(&mut self) -> io::Result<()>;

    async fn cursor_goto(&mut self, cols: u16, rows: u16) -> io::Result<()>;
}
