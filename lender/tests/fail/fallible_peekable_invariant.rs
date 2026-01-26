// Test that FalliblePeekable with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending};

struct InvariantFalliblePeekable<L> {
    lender: L,
    peeked: Option<Option<Cell<Option<&'static String>>>>,
}

impl<'lend, L> FallibleLending<'lend> for InvariantFalliblePeekable<L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L> FallibleLender for InvariantFalliblePeekable<L> {
    type Error = std::io::Error;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
