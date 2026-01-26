// Test that FallibleFlatten with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending};

struct InvariantFallibleFlatten<L> {
    lender: L,
}

impl<'lend, L> FallibleLending<'lend> for InvariantFallibleFlatten<L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L> FallibleLender for InvariantFallibleFlatten<L> {
    type Error = std::io::Error;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
