// Test that Map with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantMap<L, F> {
    lender: L,
    f: F,
}

impl<'lend, L, F> Lending<'lend> for InvariantMap<L, F> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, F> Lender for InvariantMap<L, F> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
