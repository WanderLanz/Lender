// Test that FromFn with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantFromFn<F> {
    f: F,
}

impl<'lend, F> Lending<'lend> for InvariantFromFn<F>
where
    F: FnMut() -> Option<&'lend Cell<Option<&'lend String>>>,
{
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<F> Lender for InvariantFromFn<F>
where
    F: for<'lend> FnMut() -> Option<&'lend Cell<Option<&'lend String>>>,
{
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        (self.f)()
    }
}

struct InvariantFromFallibleFn<E>(std::marker::PhantomData<E>);

impl<'lend, E> FallibleLending<'lend> for InvariantFromFallibleFn<E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<E> FallibleLender for InvariantFromFallibleFn<E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
