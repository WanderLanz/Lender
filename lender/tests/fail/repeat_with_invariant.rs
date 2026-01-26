// Test that RepeatWith with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantRepeatWith<F> {
    f: F,
}

impl<'lend, F> Lending<'lend> for InvariantRepeatWith<F>
where
    F: FnMut() -> &'lend Cell<Option<&'lend String>>,
{
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<F> Lender for InvariantRepeatWith<F>
where
    F: for<'lend> FnMut() -> &'lend Cell<Option<&'lend String>>,
{
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        Some((self.f)())
    }
}

struct InvariantFallibleRepeatWith<E>(std::marker::PhantomData<E>);

impl<'lend, E> FallibleLending<'lend> for InvariantFallibleRepeatWith<E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<E> FallibleLender for InvariantFallibleRepeatWith<E> {
    type Error = E;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
