// Test that Repeat with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantRepeat<'a> {
    value: &'a Cell<Option<&'a String>>,
}

impl<'lend> Lending<'lend> for InvariantRepeat<'_> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl Lender for InvariantRepeat<'_> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        Some(self.value)
    }
}

fn main() {}
