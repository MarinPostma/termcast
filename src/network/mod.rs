mod client;

use std::net::SocketAddr;

use tokio::sync::broadcast;
use tokio::net::TcpListener;
use log::{error, info};

use crate::cell::Cell;
use client::Client;

pub struct Network {
    sender: broadcast::Sender<Vec<(usize, usize, Cell)>>,
    addr: SocketAddr,
}

impl Network {
    pub fn new(
        sender: broadcast::Sender<Vec<(usize, usize, Cell)>>,
        addr: SocketAddr,
    ) -> Self {
        Self {
            sender,
            addr,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.addr).await.unwrap();
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let client = Client::new(stream, self.sender.subscribe());
                    info!("new client connected");
                    tokio::task::spawn(client.run());
                }
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
    }
}
