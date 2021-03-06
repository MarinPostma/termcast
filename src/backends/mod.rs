mod termion;

use async_trait::async_trait;
use tokio::io;

use crate::cell::Cell;

pub use self::termion::TermionBackend;

#[async_trait(?Send)]
pub trait Backend {
    async fn draw<I>(&mut self, content: I) -> io::Result<()>
        where
            I: Iterator<Item = (usize, usize, Cell)> + Sync + Send;

    async fn clear(&mut self) -> io::Result<()>;

    async fn hide_cursor(&mut self) -> io::Result<()>;

    async fn show_cursor(&mut self) -> io::Result<()>;

    async fn cursor_goto(&mut self, cols: usize, rows: usize) -> io::Result<()>;

    async fn flush(&mut self) -> io::Result<()>;
}
