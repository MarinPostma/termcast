use std::convert::TryFrom;
use std::io::{stdin, Stdout};
use std::os::unix::io::AsRawFd;
use std::process::Command;
use std::time::{Duration, Instant};
use std::net::SocketAddr;

use anyhow::Result;
use nix::ioctl_read_bad;
use nix::libc::TIOCGWINSZ;
use nix::pty::{forkpty, Winsize};
use nix::unistd::ForkResult;
use termion::raw::IntoRawMode;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, broadcast};
use tokio_fd::AsyncFd;

use crate::backends::{Backend, TermionBackend};
use crate::layout::Rect;
use crate::terminal::Terminal;
use crate::network::Network;

const FPS: u64 = 60;

ioctl_read_bad!(get_win_size, TIOCGWINSZ, Winsize);

pub struct Host {
    terminal: Terminal<TermionBackend<termion::raw::RawTerminal<Stdout>>>,
    parser: vte::Parser,
    master: AsyncFd,
}

impl Host {
    pub async fn new(cols: usize, rows: usize) -> Result<Self> {
        let winsize = Winsize {
            ws_row: rows as u16,
            ws_col: cols as u16,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let pty_fork_result = forkpty(Some(&winsize), None)?;
        let master_fd = pty_fork_result.master;
        let master = AsyncFd::try_from(master_fd)?;

        match pty_fork_result.fork_result {
            ForkResult::Parent { .. } => {
                let stdout = std::io::stdout().into_raw_mode()?;
                let mut master_winsize = Winsize {
                    ws_row: 0,
                    ws_col: 0,
                    ws_xpixel: 0,
                    ws_ypixel: 0,
                };
                unsafe { get_win_size(stdout.as_raw_fd(), &mut master_winsize as *mut _) }?;

                let mut backend = TermionBackend::new(stdout);
                backend.clear().await?;

                let rect = Rect::new(
                    master_winsize.ws_col as usize / 2 - cols / 2,
                    master_winsize.ws_row as usize / 2 - rows / 2,
                    cols,
                    rows,
                );
                let terminal = Terminal::new(rect, backend);

                let parser = vte::Parser::new();
                Ok(Self {
                    terminal,
                    parser,
                    master,
                })
            }
            ForkResult::Child => {
                let mut child = Command::new("bash").spawn()?;
                child.wait()?;
                std::process::exit(0);
            }
        }
    }

    pub async fn run(mut self) -> Result<()> {
        let mut buf = [0; 4096];
        let mut stdin = spawn_stdin();
        let (mut master_read, mut master_write) = split(self.master);
        let mut last_draw_time = Instant::now();
        let period = Duration::from_millis(1000 / FPS);

        let (sender, _) = broadcast::channel(100);
        let addr = SocketAddr::try_from(([0, 0, 0, 0], 9999))?;
        let network = Network::new(sender.clone(), addr);

        tokio::task::spawn(network.run());

        let mut interval = tokio::time::interval(period);

        loop {
            tokio::select! {
                result = master_read.read(&mut buf) => {
                    match result {
                        Ok(n) if n > 0 => {
                            for byte in &buf[..n] {
                                self.parser.advance(&mut self.terminal, *byte);
                            }
                            if last_draw_time.elapsed() >= period {
                                let change = self.terminal.draw().await?;
                                if !change.is_empty() {
                                    let _ = sender.send(change);
                                }
                                last_draw_time = Instant::now();
                            }
                        }
                        _ => break,
                    }
                },
                result = stdin.recv() => {
                    match result {
                        Some(byte) => {
                            master_write.write(&[byte as u8]).await?;
                            master_write.flush().await?;
                        }
                        _ => break,
                    }
                }
                _ = interval.tick() => {
                    if last_draw_time.elapsed() >= period {
                        let change = self.terminal.draw().await?;
                        if !change.is_empty() {
                            let _ = sender.send(change);
                        }
                        last_draw_time = Instant::now();
                    }
                }
            }
        }
        Ok(())
    }
}

fn spawn_stdin() -> mpsc::UnboundedReceiver<u8> {
    let (stdin_snd, stdin_recv) = mpsc::unbounded_channel();
    tokio::task::spawn_blocking(move || {
        use std::io::Read;
        let mut stdin = stdin();
        let mut buf = [0; 1];
        loop {
            match stdin.read(&mut buf) {
                Ok(1) => {
                    if stdin_snd.send(buf[0] as u8).is_err() {
                        break;
                    }
                }
                _ => break,
            }
        }
    });
    stdin_recv
}
