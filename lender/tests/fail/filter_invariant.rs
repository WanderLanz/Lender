// Test that Filter with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantFilter<L, P> {
    lender: L,
    predicate: P,
}

impl<'lend, L, P> Lending<'lend> for InvariantFilter<L, P> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, P> Lender for InvariantFilter<L, P> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
