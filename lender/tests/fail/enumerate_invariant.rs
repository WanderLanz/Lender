// Test that Enumerate with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantEnumerate<L> {
    lender: L,
    count: usize,
}

impl<'lend, L> Lending<'lend> for InvariantEnumerate<L> {
    type Lend = (usize, &'lend Cell<Option<&'lend String>>);
}

impl<L> Lender for InvariantEnumerate<L> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
