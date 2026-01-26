// Test that MapErr (fallible-only adapter) with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending};

struct InvariantMapErr<E, L, F>(L, F, std::marker::PhantomData<E>);

impl<'lend, E, L, F> FallibleLending<'lend> for InvariantMapErr<E, L, F> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<E, L, F> FallibleLender for InvariantMapErr<E, L, F> {
    type Error = E;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
