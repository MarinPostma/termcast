use std::convert::TryFrom;
use std::error::Error;
use std::io::{Stdout, stdin};
use std::os::unix::io::AsRawFd;
use std::process::Command;

use nix::ioctl_read_bad;
use nix::libc::TIOCGWINSZ;
use nix::pty::{Winsize, forkpty};
use nix::unistd::ForkResult;
use termion::raw::IntoRawMode;
use tokio::io::{AsyncReadExt, AsyncWriteExt, split};
use tokio::sync::mpsc;
use tokio_fd::AsyncFd;

use crate::backends::{Backend, TermionBackend};
use crate::layout::Rect;
use crate::terminal::Terminal;

ioctl_read_bad!(get_win_size, TIOCGWINSZ, Winsize);

pub struct Host {
    terminal: Terminal<TermionBackend<termion::raw::RawTerminal<Stdout>>>,
    parser: vte::Parser,
    master: AsyncFd,
}

impl Host {
    pub async fn new(cols: u16, rows: u16) -> Result<Self, Box<dyn Error>> {
        let winsize = Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0,  ws_ypixel: 0 };
        let pty_fork_result = forkpty(Some(&winsize), None)?;
        let master_fd = pty_fork_result.master;
        let master = AsyncFd::try_from(master_fd)?;

        match pty_fork_result.fork_result {
            ForkResult::Parent { .. } => {
                let stdout = std::io::stdout().into_raw_mode()?;
                let mut master_winsize = Winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0,  ws_ypixel: 0 };
                unsafe { get_win_size(stdout.as_raw_fd(), &mut master_winsize as *mut _) }?;

                let mut backend = TermionBackend::new(stdout);
                backend.clear().await?;

                let rect = Rect::new(master_winsize.ws_col / 2 - 40, master_winsize.ws_row / 2 - 20, cols, rows);
                let terminal = Terminal::new(rect, backend);

                let parser = vte::Parser::new();
                Ok(Self { terminal, parser, master })
            },
            ForkResult::Child => {
                let mut child = Command::new("bash").spawn()?;
                child.wait()?;
                std::process::exit(0);
            }
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        let mut buf = [0; 4096];
        let mut stdin = spawn_stdin();
        let (mut master_read, mut master_write) = split(self.master);

        loop {
            tokio::select! {
                result = master_read.read(&mut buf) => {
                    match result {
                        Ok(n) if n > 0 => {
                            for byte in &buf[..n] {
                                self.parser.advance(&mut self.terminal, *byte);
                            }
                            self.terminal.draw().await?;
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
                        break
                    }
                },
                _ => break
            }
        }
    });
    stdin_recv
}
