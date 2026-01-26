// Test that WindowsMut with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantWindowsMut<'a> {
    slice: &'a mut [Cell<Option<&'a String>>],
    pos: usize,
    size: usize,
}

impl<'lend> Lending<'lend> for InvariantWindowsMut<'_> {
    type Lend = &'lend mut [Cell<Option<&'lend String>>];
}

impl Lender for InvariantWindowsMut<'_> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        if self.pos + self.size <= self.slice.len() {
            let window = &mut self.slice[self.pos..self.pos + self.size];
            self.pos += 1;
            Some(unsafe { &mut *(window as *mut _) })
        } else {
            None
        }
    }
}

fn main() {}
