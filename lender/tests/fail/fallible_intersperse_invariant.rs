// Test that FallibleIntersperse with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending};

struct InvariantFallibleIntersperse<L> {
    lender: L,
    separator: Cell<Option<&'static String>>,
    needs_sep: bool,
}

impl<'lend, L> FallibleLending<'lend> for InvariantFallibleIntersperse<L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L> FallibleLender for InvariantFallibleIntersperse<L> {
    type Error = std::io::Error;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
