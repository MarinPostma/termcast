use std::io; 
use std::ops::{Deref, DerefMut, Range};

use crate::layout::Rect;
use crate::cell::Cell;
use crate::style::Style;
use crate::backends::Backend;

struct Buffer {
    cells: Vec<Cell>,
    rect: Rect,
}

impl Buffer {

    pub fn new(rect: Rect) -> Self {
        Self {
            cells: vec![Cell::default(); (rect.width * rect.height) as usize],
            rect,
        }
    }
    pub fn cells<'a>(&'a self) -> Vec<(u16, u16, &'a Cell)> {
        self
            .cells
            .iter()
            .enumerate()
            .map(|(i, c)| (i as u16 % self.rect.width + 1 + self.rect.x, i as u16 / self.rect.width + 1 + self.rect.y, c))
            .collect()
    }
}

impl Deref for Buffer {
    type Target = [Cell];

    fn deref(&self) -> &Self::Target {
        &self.cells
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cells
    }
}

pub struct Terminal<B: Backend> {
    c_style: Style,
    buffer: Buffer,
    rect: Rect,
    c_row: u16,
    c_col: u16,
    backend: B,
    scroll_range: Range<usize>,
}

impl<B: Backend> Terminal<B> {
    pub fn new(rect: Rect, backend: B) -> Terminal<B> {
        Terminal {
            scroll_range: 1..rect.height as usize,
            buffer: Buffer::new(rect.clone()),
            rect,
            c_style: Style::default(),
            c_col: 1,
            c_row: 1,
            backend,
        }
    }

    fn index_of(&self, x: u16, y: u16) -> usize {
        ((y - 1) * self.rect.width + (x - 1)) as usize
    }

    fn move_up(&mut self, n: u16) {
        self.c_row = std::cmp::max(1, self.c_row.saturating_sub(n));
    }

    fn move_down(&mut self, n: u16) {
        self.c_row = std::cmp::min(self.rect.height - 1, self.c_row + n);
    }

    fn move_left(&mut self, n: u16) {
        self.c_col = std::cmp::max(1, self.c_col.saturating_sub(n));
        println!("here");
    }

    fn move_right(&mut self, n: u16) {
        self.c_col = std::cmp::min(self.rect.width - 1, self.c_col + n);
    }

    fn move_cursor(&mut self, cols: u16, rows: u16) {
        self.c_col = cols;
        self.c_row = rows;
    }

    fn current_index(&self) -> usize {
        self.index_of(self.c_col, self.c_row)
    }

    fn delete_lines(&mut self, num: u16) {
        eprintln!("delete; {}", num);
        let start = self.scroll_range.start * self.buffer.rect.width as usize;
        let end = (self.scroll_range.end - 1) * self.buffer.rect.width as usize;
        self.buffer.cells.drain(start..start + num as usize * self.buffer.rect.width as usize);
        self.buffer.cells.splice(end..end,
            (0..num as usize * self.buffer.rect.width as usize).map(|_| Cell::default()));
    }

    fn clear_down(&mut self) {
        let index = self.current_index();
        self.buffer[index..].iter_mut().for_each(|c| { c.reset(); });
    }

    fn insert_line(&mut self, num: u16) {
        eprintln!("insert; {}", self.buffer.cells.len());
        let index = ((self.c_row - 1) * self.buffer.rect.width) as usize;
        self.buffer.cells.drain(self.buffer.cells.len() - num as usize * self.buffer.rect.width as usize..);
        self.buffer.cells.splice(index..index,
            (0.. num * self.buffer.rect.width).map(|_| Cell::default()));
    }

    fn clear_line_right(&mut self) {
        let width = self.rect.width as usize;
        let index = self.current_index();
        let end_index = (index + width - 1) / width * width;
        self.buffer[index..end_index].iter_mut().for_each(|c| { c.reset(); });
    }

    fn clear_n(&mut self, n: usize) {
        let index = self.current_index();
        self.buffer[index..index + n].iter_mut().for_each(|c| { c.reset(); });
    }

    pub fn draw(&mut self) -> io::Result<()> {
        self.backend.hide_cursor()?;
        let cells = self.buffer.cells();
        self.backend.draw(cells.into_iter())?;
        self.backend.cursor_goto(self.c_col + self.rect.x, self.c_row + self.rect.y)?;
        self.backend.show_cursor()?;
        self.backend.flush()?;
        Ok(())
    }

    #[allow(dead_code)]
    fn render_borders(&mut self) {
        // render top
        for i in 1..=self.rect.width {
            let index = self.index_of(i as u16, 1);
            self
                .buffer
                .get_mut(index)
                .expect(&format!("no value at {}", index))
                .set_symbol('─');
        }
        // render bottom
        for i in 1..=self.rect.width {
            let index = self.index_of(i as u16, self.rect.height);
            self
                .buffer
                .get_mut(index)
                .expect(&format!("no value at bottom {}", index))
                .set_symbol('─');
        }
        // render left
        for i in 1..=self.rect.height {
            let index = self.index_of(1, i as u16);
            self
                .buffer
                .get_mut(index)
                .expect(&format!("no value at bottom {}", index))
                .set_symbol('│');
        }
        // render right
        for i in 1..=self.rect.height {
            let index = self.index_of(self.rect.width, i as u16);
            self
                .buffer
                .get_mut(index)
                .expect(&format!("no value at bottom {}", index))
                .set_symbol('│');
        }

        // top right
        let index = self.index_of(1, 1);
        self
            .buffer
            .get_mut(index)
            .expect(&format!("no value at bottom {}", index))
            .set_symbol('┌');
        //
        // top left
        let index = self.index_of(self.rect.width, 1);
        self
            .buffer
            .get_mut(index)
            .expect(&format!("no value at bottom {}", index))
            .set_symbol('┐');
        //
        // bottom right
        let index = self.index_of(1, self.rect.height);
        self
            .buffer
            .get_mut(index)
            .expect(&format!("no value at bottom {}", index))
            .set_symbol('└');
        //
        // bottom left
        let index = self.index_of(self.rect.width, self.rect.height);
        self
            .buffer
            .get_mut(index)
            .expect(&format!("no value at bottom {}", index))
            .set_symbol('┘');

        // print title
        let title = " megaterm 5000 ";
        for (i, c) in title.chars().enumerate() {
            let index = self.index_of(i as u16 + 3, 1);
        self
            .buffer
            .get_mut(index)
            .expect(&format!("no value at bottom {}", index))
            .set_symbol(c);
        }
    }


    fn inc_row(&mut self) {

        if self.c_row == self.scroll_range.end as u16 {
            //row remains the same but the viewport is shifted up, ie rmove the first line
            let start = (self.scroll_range.start - 1) * self.rect.width as usize;
            let end = (self.scroll_range.end - 1) * self.rect.width as usize;
            self.buffer.cells.drain(start..start + self.rect.width as usize);
            self.buffer.cells.splice(end..end, (0..self.rect.width).map(|_| Cell::default()));
            //println!("buffer_len: {}", self.buffer.len());
        } else {
            self.c_row += 1;
        }
    }

    fn inc_col(&mut self) {
        if self.c_col == self.rect.width + 1 {
            self.c_col = 1;
            //println!("we are here!! {}", self.c_row);
            self.inc_row();
            //println!("we are here!! {}", self.c_row);
        } else {
            self.c_col += 1;
        }
    }

    fn current_cell(&mut self) -> Option<&mut Cell> {
        //println!("ccol: {}, crow: {}", self.c_col, self.c_row);
        let index = self.index_of(self.c_col, self.c_row);
        //println!("index: {}; buffer: {}", index, self.buffer.len());
        self.buffer.get_mut(index  as usize)
    }

    fn make_tab(&mut self) {
        for i in self.c_col..(std::cmp::max(self.rect.width, self.c_col + self.c_col % 4)) {
            let index = self.index_of(i, self.c_row);
            self
                .buffer
                .get_mut(index)
                .expect(&format!("no value at bottom {}", index))
                .reset();
        }
    }
}

impl<B: Backend> vte::Perform for Terminal<B> {
    /// Draw a character to the screen and update states.
    fn print(&mut self, c: char) {
        let style = self.c_style.clone();
        let c_row = self.c_row;
        let c_col = self.c_col;
        let mut cell = self.current_cell().expect(&format!("error with getting current cell: ({}, {})", c_col, c_row));
        cell.symbol = c;
        cell.style = style;
        self.inc_col();
    }

    /// Execute a C0 or C1 control function.
    fn execute(&mut self, byte: u8) {
        match byte {
            b'\r' => { self.c_col = 1; },
            b'\n' => { self.inc_row(); },
            b'\t' => { self.make_tab(); },
            // the bell
            0x7 => {
                self.backend.write(&[7]).unwrap();
                self.backend.flush().unwrap();
            },
            // backspace
            0x8 => self.c_col = self.c_col.saturating_sub(1),
            // shift in
            0xf => (),
            _ => panic!("unexpected action: {:?}", byte as char),
        }
    }

    /// Invoked when a final character arrives in first part of device control string.
    ///
    /// The control function should be determined from the private marker, final character, and
    /// execute with a parameter list. A handler should be selected for remaining characters in the
    /// string; the handler function should subsequently be called by `put` for every character in
    /// the control string.
    ///
    /// The `ignore` flag indicates that more than two intermediates arrived and
    /// subsequent characters were ignored.
    fn hook(&mut self, params: &[i64], _intermediates: &[u8], _ignore: bool, action: char) {
        eprintln!("hook: {}; params: {:?};", action, params);
    }

    /// Pass bytes as part of a device control string to the handle chosen in `hook`. C0 controls
    /// will also be passed to the handler.
    fn put(&mut self, byte: u8) {
        eprintln!("put: {};", byte as char);
    }

    /// Called when a device control string is terminated.
    ///
    /// The previously selected handler should be notified that the DCS has
    /// terminated.
    fn unhook(&mut self) {
        eprintln!("unhook");
    }

    /// Dispatch an operating system command.
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        eprintln!("osc dispat: {:?}", params);
    }

    /// A final character has arrived for a CSI sequence
    ///
    /// The `ignore` flag indicates that either more than two intermediates arrived
    /// or the number of parameters exceeded the maximum supported length,
    /// and subsequent characters were ignored.
    fn csi_dispatch(&mut self, params: &[i64], _intermediates: &[u8], _ignore: bool, action: char) {
        // we ingnore that for now
        match action {
            'A' => self.move_up(params[0] as u16),
            'B' => self.move_down(params[0] as u16),
            'C' => self.move_right(params[0] as u16),
            'D' => self.move_left(params[0] as u16),
            // show cursor
            'h' => (),
            // hide cursor
            'l' => (),
            //colors
            'm' => {
                //println!("params: {:?}, intermediates: {:?}", params, intermediates);
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
            // caps lock light on
            'q' => (),
            // CSI Ps ; Ps ; Ps t
            't' => (),
            // set scroll range
            'r' => self.scroll_range = params[0] as usize .. params[1] as usize,
            'M' => self.delete_lines(std::cmp::max(1, params[0]) as u16),
            'L' => self.insert_line(std::cmp::max(1, params[0]) as u16),
            'H' => {
                match params.len() {
                    0 | 1 => self.move_cursor(1, 1),
                    _ => self.move_cursor(params[1] as u16, params[0] as u16),
                }
            }
            'K' => self.clear_line_right(),
            // delete next n chars
            'P' => self.clear_n(params[0] as usize),
            'J' => {
                match params.get(0) {
                    None | Some(0) => self.clear_down(),
                    _ => unimplemented!("J other"),
                }
            }
            _ => {
                eprintln!("csi: {:?}; params: {:?}", params, action);
            }
        }
    }

    /// The final character of an escape sequence has arrived.
    ///
    /// The `ignore` flag indicates that more than two intermediates arrived and
    /// subsequent characters were ignored.
    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        eprintln!("esc dispatch: {:?}; params: {:?}", byte as char, intermediates);
    }
}
