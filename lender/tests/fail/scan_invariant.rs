// Test that Scan with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantScan<L, St, F> {
    lender: L,
    state: St,
    f: F,
}

impl<'lend, L, St, F> Lending<'lend> for InvariantScan<L, St, F> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, St, F> Lender for InvariantScan<L, St, F> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
