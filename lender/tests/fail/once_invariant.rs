// Test that Once with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantOnce<'a> {
    inner: Option<&'a Cell<Option<&'a String>>>,
}

impl<'lend> Lending<'lend> for InvariantOnce<'_> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl Lender for InvariantOnce<'_> {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.inner.take()
    }
}

struct InvariantFallibleOnce<E>(std::marker::PhantomData<E>);

impl<'lend, E> FallibleLending<'lend> for InvariantFallibleOnce<E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<E> FallibleLender for InvariantFallibleOnce<E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
