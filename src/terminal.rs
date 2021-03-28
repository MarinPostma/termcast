use std::io;
use std::ops::{Deref, DerefMut, Range};

use log::debug;

use crate::layout::Rect;
use crate::cell::Cell;
use crate::style::Style;
use crate::backends::Backend;

struct Buffer {
    current: Vec<Cell>,
    previous: Vec<Cell>,
    rect: Rect,
}

impl Buffer {

    pub fn new(rect: Rect) -> Self {
        eprintln!("buflen: {}", rect.width * rect.height);
        let current = vec![Cell::default(); (rect.width * rect.height) as usize];
        let previous = current.clone();
        Self {
            current,
            previous,
            rect,
        }
    }

    /// returns an iterator over the cells that have changed since last draw.
    pub fn diff(&mut self) -> impl Iterator<Item = (usize, usize, Cell)> + '_ {
        let width = self.rect.width;
        let x = self.rect.x;
        let y = self.rect.y;
        std::mem::swap(&mut self.current, &mut self.previous);
        let previous = &mut self.previous;
        self.current
            .iter_mut()
            .enumerate()
            .filter_map(move |(i, c)| {
                if previous[i] != *c {
                    *c = previous[i];
                    Some((i % width + x, i / width + y, *c))
                } else {
                    None
                }
            })
    }
}

impl Deref for Buffer {
    type Target = Vec<Cell>;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.current
    }
}

pub struct Terminal<B: Backend> {
    c_style: Style,
    buffer: Buffer,
    rect: Rect,
    c_row: usize,
    c_col: usize,
    pub backend: B,
    scroll_range: Range<usize>,
}

impl<B: Backend> Terminal<B> {
    pub fn new(rect: Rect, backend: B) -> Terminal<B> {
        Terminal {
            scroll_range: 0..rect.height as usize,
            buffer: Buffer::new(rect.clone()),
            rect,
            c_style: Style::default(),
            c_col: 0,
            c_row: 0,
            backend,
        }
    }

    #[inline]
    fn width(&self) -> usize {
        self.rect.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.rect.height
    }

    #[inline]
    fn row(&self) -> usize {
        self.c_row
    }

    #[inline]
    fn set_row(&mut self, row: usize) {
        assert!(row < self.height(), format!("row out of bounds: {} >= {}", row, self.height()));
        self.c_row = row;
    }

    #[inline]
    fn col(&self) -> usize {
        self.c_col
    }

    #[inline]
    fn set_col(&mut self, col: usize) {
        assert!(col < self.width(), format!("col out of bounds: {} >= {}", col, self.width()));
        self.c_col = col;
    }

    /// index of the start of the current line
    #[inline]
    fn current_line_index(&self) -> usize {
        self.row() * self.width()
    }

    #[inline]
    fn scroll_range_start_index(&self) -> usize {
        self.scroll_range.start * self.width()
    }

    #[inline]
    fn scroll_range_end_index(&self) -> usize {
        self.scroll_range.end * self.width() - 1
    }

    fn index_of(&self, x: usize, y: usize) -> usize {
        debug!("getting position ({}, {}), width = {}, height = {}", x, y, self.rect.width, self.rect.height);
        (y * self.rect.width + x) as usize
    }

    fn move_up(&mut self, n: usize) {
        debug!("move up: {}", n);
        let n_row = self.row().saturating_sub(n);
        self.set_row(n_row);
    }

    fn move_down(&mut self, n: usize) {
        debug!("move down: {}", n);
        let n_row = std::cmp::min(self.height() - 1, self.row() + n);
        self.set_row(n_row);
    }

    fn move_backward(&mut self, n: usize) {
        debug!("move back: {}", n);
        let n_col = self.col().saturating_sub(n);
        self.set_col(n_col);
    }

    fn move_forward(&mut self, n: usize) {
        debug!("move forward: {}", n);
        let n_col = std::cmp::min(self.width() - 1, self.col() + n);
        self.set_col(n_col);
    }

    fn move_down_and_cr(&mut self, n: usize) {
        debug!("move down and cr: {}", n);
        self.move_down(n);
        self.carriage_return();
    }

    fn cursor_goto(&mut self, x: usize, y: usize) {
        debug!("cursor goto: ({}, {})", x, y);
        self.set_col(x);
        self.set_row(y);
    }

    fn current_index(&self) -> usize {
        self.row() * self.width() + self.col()
    }

    fn insert_line(&mut self, n: usize) {
        debug!("inserting {} lines", n);
        let to_remove_end = self.scroll_range_end_index();
        let to_remove_start = to_remove_end - n * self.width();
        self.buffer.drain(to_remove_start..to_remove_end);
        let to_insert_start = self.current_line_index();
        let amount = n * self.width();
        self.buffer.splice(to_insert_start..to_insert_start, (0..amount).map(|_| Cell::default()));
    }

    fn delete_lines(&mut self, num: usize) {
        debug!("delete lines: {}", num);
        let start = self.current_line_index();
        let amount = std::cmp::min(self.scroll_range_end_index() - start, num * self.width());
        let end = start + amount;
        let len_before = self.buffer.len();
        let index = self.scroll_range_end_index();
        self.buffer.splice(index..index, (0..amount).map(|_| Cell::default()));
        self.buffer.drain(start..end);
        assert_eq!(len_before, self.buffer.len());
    }

    fn clear_line(&mut self, mode: LineClearMode) {
        debug!("clearing line {:?}", mode);
        let (start, end) = match mode {
            LineClearMode::Right => {
                let start = self.current_index();
                let end = self.index_of(self.width() - 1, self.row());
                (start, end)
            }
            LineClearMode::Left => {
                let start = self.index_of(0, self.row());
                let end = self.current_index();
                (start, end)
            }
            LineClearMode::All => {
                let start = self.index_of(0, self.row());
                let end = self.index_of(self.width() - 1, self.row());
                (start, end)
            }
        };
        self.buffer[start..=end].iter_mut().for_each(|c| { c.reset(); });
    }

    fn clear_screen(&mut self, mode: ClearMode) {
        debug!("clear: {:?}", mode);
        match mode {
            ClearMode::All => {
                self.buffer.iter_mut().for_each(|cell| { cell.reset(); });
            },
            ClearMode::Above => {
                let index = self.current_index();
                self.buffer[..=index].iter_mut().for_each(|cell| { cell.reset(); });
            },
            ClearMode::Below => {
                let index = self.current_index();
                self.buffer[index..].iter_mut().for_each(|cell| { cell.reset(); });
            },
            mode => {
                debug!("unhandled clear mode: {:?}", mode);
            }
        }
    }

    fn set_scroll_range(&mut self, start: usize, end: Option<usize>) {
        debug!("set scroll range: {}..{:?}", start, end);
        self.scroll_range = start..end.unwrap_or(self.height());
    }

    fn backspace(&mut self) {
        debug!("back space");
        self.dec_col();
    }

    fn carriage_return(&mut self) {
        debug!("carriage return");
        self.set_col(0);
    }

    fn inc_row(&mut self) {
        debug!("inc row, c_row: {}, range_end: {}", self.c_row, self.scroll_range.end);
        if self.c_row < self.scroll_range.end - 1 {
            self.c_row += 1;
        } else {
            debug!("here");
            //row remains the same but the viewport is shifted up, ie rmove the first line
            let width = self.width();
            let start = (self.scroll_range.start) * width;
            let end = (self.scroll_range.end - 1) * width;
            debug!("width: {}, start: {}, end: {}", width, start, end);
            self.buffer.drain(start..start + width);
            self.buffer.splice(end..end, (0..width).map(|_| Cell::default()));
        }
    }

    fn inc_col(&mut self) {
        debug!("inc col");
        if self.c_col >= self.width() {
            self.c_col = 1;
            self.inc_row();
        } else {
            self.c_col += 1;
        }
    }

    fn dec_row(&mut self) {
        debug!("dec row");
        let mut n_row = self.row().saturating_sub(1);
        if n_row < self.scroll_range.start {
            self.scroll_up();
            n_row = self.scroll_range.start;
        }
        self.set_row(n_row);
    }

    fn scroll_up(&mut self) {
        debug!("scroll up");
        let len_before = self.buffer.len();
        let end = self.scroll_range_end_index();
        let start = end - self.width();
        self.buffer.drain(start..end);
        let index = self.scroll_range_start_index();
        let width = self.width();
        self.buffer.splice(index..index, (0..width).map(|_| Cell::default()));
        assert_eq!(len_before, self.buffer.len());
    }

    fn dec_col(&mut self) {
        if self.col() == 0 {
            let n_col = self.width() - 1;
            debug!("dec col: {}", n_col);
            self.scroll_up();
            self.set_col(n_col);
        } else {
            let n_col = self.col().wrapping_sub(1);
            debug!("dec col: {}", n_col);
            self.set_col(n_col);
        }
    }

    fn current_cell_mut(&mut self) -> Option<&mut Cell> {
        let index = self.index_of(self.col(), self.row());
        self.buffer.get_mut(index)
    }

    fn put_char(&mut self, c: char) {
        //debug!("put char: {}", c);
        let style = self.c_style.clone();
        let c_row = self.c_row;
        let c_col = self.c_col;
        let cell = self.current_cell_mut().expect(&format!("error with getting current cell: ({}, {})", c_col, c_row));
        cell.set_symbol(c);
        cell.set_style(style);
        self.inc_col();
    }

    fn put_tab(&mut self) {
        debug!("put tab");
        for i in self.c_col..(std::cmp::max(self.rect.width, self.c_col + self.c_col % 4)) {
            let index = self.index_of(i, self.c_row);
            self
                .buffer[index]
                .reset();
        }
    }

    fn linefeed(&mut self) {
        debug!("line feed");
        self.inc_row();
    }

    fn reverse_index(&mut self) {
        debug!("reverse index");
        self.dec_row();
    }

    fn bell(&mut self) {
        debug!("Bell!");
        ()
    }

    pub async fn draw(&mut self) -> io::Result<()> {
        self.backend.hide_cursor().await?;
        let cells = self.buffer.diff();
        self.backend.draw(cells).await?;
        self.backend.cursor_goto(self.c_col + self.rect.x, self.c_row + self.rect.y).await?;
        self.backend.show_cursor().await?;
        self.backend.flush().await?;
        Ok(())
    }
}

#[derive(Debug)]
enum LineClearMode {
    Right,
    Left,
    All,
}

#[derive(Debug)]
enum ClearMode {
    Below,
    Above,
    All,
    Saved,
}

impl<B: Backend> vte::Perform for Terminal<B> {
    /// Draw a character to the screen and update states.
    #[inline]
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    #[inline]
    fn execute(&mut self, byte: u8) {
        match byte {
            C0::HT => self.put_tab(),
            C0::BS => self.backspace(),
            C0::CR => self.carriage_return(),
            C0::LF | C0::VT | C0::FF => self.linefeed(),
            C0::BEL => self.bell(),
            //C0::SUB => self.handler.substitute(),
            _ => debug!("[unhandled] execute byte={:02x}", byte),
        }
    }

    #[inline]
    fn hook(&mut self, params: &vte::Params, intermediates: &[u8], ignore: bool, _c: char) {
        debug!(
            "[unhandled hook] params={:?}, ints: {:?}, ignore: {:?}",
            params, intermediates, ignore
        );
    }

    #[inline]
    fn put(&mut self, byte: u8) {
        debug!("[unhandled put] byte={:?}", byte);
    }

    #[inline]
    fn unhook(&mut self) {
        debug!("[unhandled unhook]");
    }

    // TODO replace OSC parsing with parser combinators.
    #[inline]
    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        match params[0] {
            _ => debug!("[unhandled osc dispatch] byte={:?}", params),
        }
    }

    #[allow(clippy::cognitive_complexity)]
    #[inline]
    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        intermediates: &[u8],
        has_ignored_intermediates: bool,
        action: char,
    ) {
        let mut params_iter = params.iter();
        let mut next_param_or = |default: usize| {
            params_iter.next().map(|param| param[0] as usize).filter(|&param| param != 0).unwrap_or(default)
        };

        match (action, intermediates.get(0)) {
            ('A', None) => self.move_up(next_param_or(1)),
            ('B', None) | ('e', None) => self.move_down(next_param_or(1)),
            ('C', None) | ('a', None) => self.move_forward(next_param_or(1)),
            ('D', None) => self.move_backward(next_param_or(1)),
            ('E', None) => self.move_down_and_cr(next_param_or(1)),
            ('H', None) | ('f', None) => {
                let y = next_param_or(1);
                let x = next_param_or(1);
                self.cursor_goto(x - 1, y - 1);
            },
            ('J', None) => {
                let mode = match next_param_or(0) {
                    0 => ClearMode::Below,
                    1 => ClearMode::Above,
                    2 => ClearMode::All,
                    3 => ClearMode::Saved,
                    _ => {
                        return;
                    },
                };

                self.clear_screen(mode);
            },
            ('L', None) => self.insert_line(next_param_or(1)),
            ('K', None) => {
                let mode = match next_param_or(0) {
                    0 => LineClearMode::Right,
                    1 => LineClearMode::Left,
                    2 => LineClearMode::All,
                    _ => {
                        return;
                    },
                };
                self.clear_line(mode);
            },
            ('M', None) => self.delete_lines(next_param_or(1)),
            //colors
            ('m', None) => {
                //println!("params: {:?}, intermediates: {:?}", params, intermediates);
                let params = params.iter().map(|p| p[0] as usize).collect::<Vec<_>>();
                match params[0] {
                    0 => { self.c_style.reset(); }
                    3 => { self.c_style.set_italic(); }
                    1 => {
                        self.c_style.set_bold();
                        if params.len() == 2 {
                            self.c_style.set_color_3bits(params[1] as usize);
                        }
                    }
                    23 => { self.c_style.unset_italic(); }
                    24 => { self.c_style.unset_underline(); }
                    // set foreground
                    38 => {
                        match params[1] {
                            5 => self.c_style.fg = crate::style::Color::Indexed(params[2] as u8),
                            2 => self.c_style.fg = crate::style::Color::Rgb(params[2] as u8, params[3] as u8, params[4] as u8),
                            _ => unreachable!("bad rgb color")
                        }
                    }
                    39 => { self.c_style.fg = crate::style::Color::default(); }
                    // set background
                    48 => {
                        match params[1] {
                            5 => self.c_style.bg = crate::style::Color::Indexed(params[2] as u8),
                            2 => self.c_style.bg = crate::style::Color::Rgb(params[2] as u8, params[3] as u8, params[4] as u8),
                            _ => unreachable!("bad rgb color")
                        }

                    }
                    49 => { self.c_style.bg = crate::style::Color::default(); }
                    90..=97
                        | 100..=107
                        | 30..=37
                        | 40..=47 => { self.c_style.set_color_3bits(params[0] as usize) }
                    value => unimplemented!("unimplemented color: {}", value),
                }
            },
            ('r', None) => {
                let top = next_param_or(1) as usize;
                let bottom =
                    params_iter.next().map(|param| param[0] as usize).filter(|&param| param != 0);

                self.set_scroll_range(top, bottom);
            },
            (c, intermediates) => debug!("[unhandled csi dispatch] char={}, intermediates={:?}", c as char, intermediates),
        }
    }

    #[inline]
    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        match (byte, intermediates.get(0)) {
            //(b'B', intermediate) => configure_charset!(StandardCharset::Ascii, intermediate),
            (b'D', None) => self.linefeed(),
            (b'E', None) => {
                self.linefeed();
                self.carriage_return();
            },
            //(b'H', None) => self.handler.set_horizontal_tabstop(),
            (b'M', None) => self.reverse_index(),
            //(b'Z', None) => self.handler.identify_terminal(self.writer, None),
            //(b'c', None) => self.handler.reset_state(),
            //(b'0', intermediate) => {
                //configure_charset!(StandardCharset::SpecialCharacterAndLineDrawing, intermediate)
            //},
            //(b'7', None) => self.handler.save_cursor_position(),
            //(b'8', Some(b'#')) => self.handler.decaln(),
            //(b'8', None) => self.handler.restore_cursor_position(),
            //(b'=', None) => self.handler.set_keypad_application_mode(),
            //(b'>', None) => self.handler.unset_keypad_application_mode(),
            //// String terminator, do nothing (parser handles as string terminator).
            //(b'\\', None) => (),
            (c, intermediates) => debug!("[unhandled esc dispatch] char={}, intermediates={:?}", c as char, intermediates),
        }
    }
}


// from allacritty
#[allow(non_snake_case)]
pub mod C0 {
    /// Null filler, terminal should ignore this character.
    pub const NUL: u8 = 0x00;
    /// Start of Header.
    pub const SOH: u8 = 0x01;
    /// Start of Text, implied end of header.
    pub const STX: u8 = 0x02;
    /// End of Text, causes some terminal to respond with ACK or NAK.
    pub const ETX: u8 = 0x03;
    /// End of Transmission.
    pub const EOT: u8 = 0x04;
    /// Enquiry, causes terminal to send ANSWER-BACK ID.
    pub const ENQ: u8 = 0x05;
    /// Acknowledge, usually sent by terminal in response to ETX.
    pub const ACK: u8 = 0x06;
    /// Bell, triggers the bell, buzzer, or beeper on the terminal.
    pub const BEL: u8 = 0x07;
    /// Backspace, can be used to define overstruck characters.
    pub const BS: u8 = 0x08;
    /// Horizontal Tabulation, move to next predetermined position.
    pub const HT: u8 = 0x09;
    /// Linefeed, move to same position on next line (see also NL).
    pub const LF: u8 = 0x0A;
    /// Vertical Tabulation, move to next predetermined line.
    pub const VT: u8 = 0x0B;
    /// Form Feed, move to next form or page.
    pub const FF: u8 = 0x0C;
    /// Carriage Return, move to first character of current line.
    pub const CR: u8 = 0x0D;
    /// Shift Out, switch to G1 (other half of character set).
    pub const SO: u8 = 0x0E;
    /// Shift In, switch to G0 (normal half of character set).
    pub const SI: u8 = 0x0F;
    /// Data Link Escape, interpret next control character specially.
    pub const DLE: u8 = 0x10;
    /// (DC1) Terminal is allowed to resume transmitting.
    pub const XON: u8 = 0x11;
    /// Device Control 2, causes ASR-33 to activate paper-tape reader.
    pub const DC2: u8 = 0x12;
    /// (DC2) Terminal must pause and refrain from transmitting.
    pub const XOFF: u8 = 0x13;
    /// Device Control 4, causes ASR-33 to deactivate paper-tape reader.
    pub const DC4: u8 = 0x14;
    /// Negative Acknowledge, used sometimes with ETX and ACK.
    pub const NAK: u8 = 0x15;
    /// Synchronous Idle, used to maintain timing in Sync communication.
    pub const SYN: u8 = 0x16;
    /// End of Transmission block.
    pub const ETB: u8 = 0x17;
    /// Cancel (makes VT100 abort current escape sequence if any).
    pub const CAN: u8 = 0x18;
    /// End of Medium.
    pub const EM: u8 = 0x19;
    /// Substitute (VT100 uses this to display parity errors).
    pub const SUB: u8 = 0x1A;
    /// Prefix to an escape sequence.
    pub const ESC: u8 = 0x1B;
    /// File Separator.
    pub const FS: u8 = 0x1C;
    /// Group Separator.
    pub const GS: u8 = 0x1D;
    /// Record Separator (sent by VT132 in block-transfer mode).
    pub const RS: u8 = 0x1E;
    /// Unit Separator.
    pub const US: u8 = 0x1F;
    /// Delete, should be ignored by terminal.
    pub const DEL: u8 = 0x7f;
}
