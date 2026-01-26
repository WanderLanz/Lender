// Test that Cycle with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantCycle<L> {
    orig: L,
    lender: L,
}

impl<'lend, L> Lending<'lend> for InvariantCycle<L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L: Clone> Lender for InvariantCycle<L> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
