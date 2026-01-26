// Test that FallibleEmpty with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending};

struct InvariantFallibleEmpty<E> {
    _marker: std::marker::PhantomData<E>,
}

impl<'lend, E> FallibleLending<'lend> for InvariantFallibleEmpty<E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<E> FallibleLender for InvariantFallibleEmpty<E> {
    type Error = E;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
