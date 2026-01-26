// Test that Rev with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantRev<L> {
    lender: L,
}

impl<'lend, L> Lending<'lend> for InvariantRev<L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L> Lender for InvariantRev<L> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
