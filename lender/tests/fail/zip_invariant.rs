// Test that Zip with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantZip<A, B>(A, B);

impl<'lend, A, B> Lending<'lend> for InvariantZip<A, B> {
    type Lend = (&'lend Cell<Option<&'lend String>>, &'lend Cell<Option<&'lend String>>);
}

impl<A, B> Lender for InvariantZip<A, B> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleZip<A, B, E>(A, B, std::marker::PhantomData<E>);

impl<'lend, A, B, E> FallibleLending<'lend> for InvariantFallibleZip<A, B, E> {
    type Lend = (&'lend Cell<Option<&'lend String>>, &'lend Cell<Option<&'lend String>>);
}

impl<A, B, E> FallibleLender for InvariantFallibleZip<A, B, E> {
    type Error = E;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
