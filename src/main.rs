#![allow(dead_code)]

#[macro_use]
extern crate bitflags;

mod terminal;
mod style;
mod backends;
mod layout;
mod cell;

use std::process::Command;
use std::os::unix::io::RawFd;
use nix::pty::{Winsize, forkpty};
use nix::unistd::ForkResult;
use std::io::{stdout, stdin};
use termion::raw::IntoRawMode;
use nix::ioctl_read_bad;
use nix::libc::TIOCGWINSZ;
use std::os::unix::io::AsRawFd;
use futures::prelude::*;
use smol::{blocking, reader, writer};
use std::fs::File;
use std::os::unix::io::FromRawFd;
use futures::select;

use crate::terminal::Terminal;
use crate::layout::Rect;
use crate::backends::{Backend, TermionBackend};




#[derive(Default)]
struct State;

ioctl_read_bad!(get_win_size, TIOCGWINSZ, Winsize);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let winsize = Winsize { ws_row: 40, ws_col: 80, ws_xpixel: 0,  ws_ypixel: 0 };
    let pty_fork_result = forkpty(Some(&winsize), None)?;
    let master_fd: RawFd = pty_fork_result.master;
    match pty_fork_result.fork_result {
        ForkResult::Parent { .. } => {
            let mut master_winsize = Winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0,  ws_ypixel: 0 };
            let stdout = stdout().into_raw_mode().unwrap();
            unsafe { get_win_size(stdout.as_raw_fd(), &mut master_winsize as *mut _) }?;
            let mut buf = [0; 1024];
            //println!("master winsize: {:?}", master_winsize);
            let mut parser = vte::Parser::new();
            let rect = Rect::new(master_winsize.ws_col / 2 - 40, master_winsize.ws_row / 2 - 20, 80, 40);
            let mut backend = TermionBackend::new(stdout);
            backend.clear()?;
            let mut terminal = Terminal::new(rect, backend);
            //println!("buffer_len: {}", performer.buffer.len());

            smol::run(async {

                let stdin = blocking!(stdin());
                let mut stdin = reader(stdin);

                let master_reader = blocking!(unsafe { File::from_raw_fd(master_fd) });
                let mut master_reader = reader(master_reader);

                let master_writer = blocking!(unsafe { File::from_raw_fd(master_fd) });
                let mut master_writer = writer(master_writer);

                loop {

                    let mut mread = master_reader.read(&mut buf).fuse();
                    let mut byte = [0; 1];
                    let mut sread = stdin.read(&mut byte).fuse();

                    select! {
                        m_reader = mread => {
                            match m_reader {
                                Ok(n) => {
                                    for byte in &buf[..n] {
                                        parser.advance(&mut terminal, *byte);
                                    }
                                    terminal.draw()?;
                                }
                                Err(_) => {
                                    return Ok(())
                                }
                            }
                        }
                        s_read = sread => {
                            match s_read {
                                Ok(n) => {
                                    master_writer.write(&byte[..n]).await?;
                                    master_writer.flush().await?;
                                }
                                Err(e) => {
                                    return Ok(())
                                }
                            }
                        }
                    }
                }
            })
        }
        ForkResult::Child => {
            let mut child = Command::new("bash").spawn()?;
            child.wait()?;
            Ok(())
        }
    }
}
