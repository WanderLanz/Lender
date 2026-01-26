// Test that Intersperse with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantIntersperse<L> {
    lender: L,
    separator: Cell<Option<&'static String>>,
    needs_sep: bool,
}

impl<'lend, L> Lending<'lend> for InvariantIntersperse<L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L> Lender for InvariantIntersperse<L> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
