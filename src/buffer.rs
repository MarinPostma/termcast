use std::ops::{Deref, DerefMut};

use crate::cell::Cell;
use crate::layout::Rect;

pub struct Buffer {
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

