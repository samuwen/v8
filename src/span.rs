use std::{
    cmp::{max, min},
    ops::Range,
};

#[derive(Clone, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize) -> Self {
        Self { start, end, line }
    }

    pub fn get_as_range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn concatenate(&self, other: &Self) -> Self {
        let start = min(self.start, other.start);
        let end = max(self.end, other.end);
        Self {
            start,
            end,
            line: self.line, // imperfect as hell but i don't care
        }
    }
}
