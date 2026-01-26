// Test that Zip with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantZip<A, B> {
    a: A,
    b: B,
}

impl<'lend, A, B> Lending<'lend> for InvariantZip<A, B> {
    type Lend = (&'lend Cell<Option<&'lend String>>, &'lend Cell<Option<&'lend String>>);
}

impl<A, B> Lender for InvariantZip<A, B> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
