// Test that MapWhile with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantMapWhile<L, P> {
    lender: L,
    predicate: P,
}

impl<'lend, L, P> Lending<'lend> for InvariantMapWhile<L, P> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, P> Lender for InvariantMapWhile<L, P> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
