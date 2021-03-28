use crate::buffer::Buffer;
use crate::cell::Cell;
use crate::layout::Rect;

use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::io::AsyncWriteExt;
use log::error;

#[allow(dead_code)]
pub struct Client {
    stream: TcpStream,
    receiver: broadcast::Receiver<Vec<(usize, usize, Cell)>>,
    buffer: Buffer,
}

impl Client {
    pub fn new(
        stream: TcpStream,
        receiver: broadcast::Receiver<Vec<(usize, usize, Cell)>>,
    ) -> Self {
        let buffer = Buffer::new(Rect::new(0, 0, 80, 40));
        Self {
            stream,
            receiver,
            buffer,
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            match self.receiver.recv().await {
                Ok(_data) => {
                    self.stream.write_all(b"hello").await?;
                }
                Err(e) => {
                    error!("client error: {}", e);
                    break
                },
            }
        }
        Ok(())
    }
}
