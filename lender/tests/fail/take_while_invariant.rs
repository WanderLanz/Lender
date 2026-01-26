// Test that TakeWhile with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantTakeWhile<L, P> {
    lender: L,
    flag: bool,
    predicate: P,
}

impl<'lend, L, P> Lending<'lend> for InvariantTakeWhile<L, P> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, P> Lender for InvariantTakeWhile<L, P> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
