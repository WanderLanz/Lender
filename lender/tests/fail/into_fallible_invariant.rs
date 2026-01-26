// Test that IntoFallible with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending};

struct InvariantIntoFallible<E, L> {
    lender: L,
    _marker: std::marker::PhantomData<E>,
}

impl<'lend, E, L> FallibleLending<'lend> for InvariantIntoFallible<E, L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<E, L> FallibleLender for InvariantIntoFallible<E, L> {
    type Error = E;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
