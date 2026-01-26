// Test that SkipWhile with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantSkipWhile<L, P> {
    lender: L,
    flag: bool,
    predicate: P,
}

impl<'lend, L, P> Lending<'lend> for InvariantSkipWhile<L, P> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, P> Lender for InvariantSkipWhile<L, P> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
