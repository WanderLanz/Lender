// Test that FallibleOnce with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending};

struct InvariantFallibleOnce<'a, E> {
    inner: Option<Result<&'a Cell<Option<&'a String>>, E>>,
}

impl<'lend, E> FallibleLending<'lend> for InvariantFallibleOnce<'_, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<E> FallibleLender for InvariantFallibleOnce<'_, E> {
    type Error = E;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        match self.inner.take() {
            None => Ok(None),
            Some(inner) => inner.map(Some),
        }
    }
}

fn main() {}
