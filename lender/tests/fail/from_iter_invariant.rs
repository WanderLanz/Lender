// Test that FromIter with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantFromIter<I> {
    iter: I,
}

impl<'lend, I> Lending<'lend> for InvariantFromIter<I>
where
    I: Iterator<Item = &'lend Cell<Option<&'lend String>>>,
{
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<I> Lender for InvariantFromIter<I>
where
    I: for<'lend> Iterator<Item = &'lend Cell<Option<&'lend String>>>,
{
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.iter.next()
    }
}

struct InvariantFromFallibleIter<E>(std::marker::PhantomData<E>);

impl<'lend, E> FallibleLending<'lend> for InvariantFromFallibleIter<E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<E> FallibleLender for InvariantFromFallibleIter<E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
