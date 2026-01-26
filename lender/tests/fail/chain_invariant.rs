// Test that Chain with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantChain<A, B> {
    a: Option<A>,
    b: B,
}

impl<'lend, A, B> Lending<'lend> for InvariantChain<A, B> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<A, B> Lender for InvariantChain<A, B> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
