// Test that Mutate with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantMutate<L, F> {
    lender: L,
    f: F,
}

impl<'lend, L, F> Lending<'lend> for InvariantMutate<L, F> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, F> Lender for InvariantMutate<L, F> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
