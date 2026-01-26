// Test that Once with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantOnce<'a> {
    inner: Option<&'a Cell<Option<&'a String>>>,
}

impl<'lend> Lending<'lend> for InvariantOnce<'_> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl Lender for InvariantOnce<'_> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.inner.take()
    }
}

fn main() {}
