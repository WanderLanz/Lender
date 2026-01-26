// Test that Peekable with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantPeekable<L> {
    lender: L,
    peeked: Option<Option<Cell<Option<&'static String>>>>,
}

impl<'lend, L> Lending<'lend> for InvariantPeekable<L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L> Lender for InvariantPeekable<L> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
