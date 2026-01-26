// Test that Chunky with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantChunky<L> {
    lender: L,
    size: usize,
}

impl<'lend, L> Lending<'lend> for InvariantChunky<L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L> Lender for InvariantChunky<L> {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
