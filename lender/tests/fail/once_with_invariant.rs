// Test that OnceWith with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantOnceWith<F> {
    f: Option<F>,
}

impl<'lend, F> Lending<'lend> for InvariantOnceWith<F>
where
    F: FnOnce() -> &'lend Cell<Option<&'lend String>>,
{
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<F> Lender for InvariantOnceWith<F>
where
    F: for<'lend> FnOnce() -> &'lend Cell<Option<&'lend String>>,
{
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.f.take().map(|f| f())
    }
}

struct InvariantFallibleOnceWith<E>(std::marker::PhantomData<E>);

impl<'lend, E> FallibleLending<'lend> for InvariantFallibleOnceWith<E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<E> FallibleLender for InvariantFallibleOnceWith<E> {
    type Error = E;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
