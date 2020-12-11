use std::fs;
use std::io::stdin;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::process::Command;
use std::error::Error;

use nix::ioctl_read_bad;
use nix::libc::TIOCGWINSZ;
use nix::pty::{Winsize, forkpty};
use nix::unistd::ForkResult;
use termion::raw::IntoRawMode;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;

use crate::terminal::Terminal;
use crate::layout::Rect;
use crate::backends::{Backend, TermionBackend};

ioctl_read_bad!(get_win_size, TIOCGWINSZ, Winsize);


pub struct Host {
    cols: u16,
    rows: u16,
    master_fd: RawFd,
}

impl Host {
    pub fn new(cols: u16, rows: u16) -> Result<Self, Box<dyn Error>> {
        let winsize = Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0,  ws_ypixel: 0 };
        let pty_fork_result = forkpty(Some(&winsize), None)?;
        let master_fd = pty_fork_result.master;

        match pty_fork_result.fork_result {
            ForkResult::Parent { .. } => Ok(Self { cols, rows, master_fd }),
            ForkResult::Child => {
                let mut child = Command::new("bash").spawn()?;
                child.wait()?;
                println!("process over");
                std::process::exit(0);
            }
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        let mut master_winsize = Winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0,  ws_ypixel: 0 };
        let stdout = std::io::stdout().into_raw_mode().unwrap();
        unsafe { get_win_size(stdout.as_raw_fd(), &mut master_winsize as *mut _) }?;
        let stdout_file = unsafe { fs::File::from_raw_fd(stdout.as_raw_fd()) };
        let stdout = File::from_std(stdout_file);
        let mut buf = [0; 4096];
        let mut parser = vte::Parser::new();
        let rect = Rect::new(master_winsize.ws_col / 2 - 40, master_winsize.ws_row / 2 - 20, self.cols, self.rows);
        let mut backend = TermionBackend::new(stdout);
        backend.clear().await?;
        let mut terminal = Terminal::new(rect, backend);

        let mut stdin = spawn_stdin();

        let master_writer_file = unsafe { fs::File::from_raw_fd(self.master_fd) };
        let mut master_writer = File::from_std(master_writer_file);

        let master_reader_file = unsafe { fs::File::from_raw_fd(self.master_fd) };
        let mut master_reader = File::from_std(master_reader_file);

        loop {
            tokio::select! {
                result = master_reader.read(&mut buf) => {
                    match result {
                        Ok(n) if n > 0 => {
                            for byte in &buf[..n] {
                                parser.advance(&mut terminal, *byte);
                            }
                            terminal.draw().await?;
                        }
                        e => {
                            println!("exited read with: {:?}", e);
                            break
                        }
                    }
                },
                result = stdin.recv() => {
                    match result {
                        Some(byte) => {
                            master_writer.write(&[byte as u8]).await?;
                            master_writer.flush().await?;
                        }
                        e => {
                            println!("exited recv with: {:?}", e);
                            break
                        }
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
