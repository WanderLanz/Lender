// Test that Enumerate with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantEnumerate<L>(L);

impl<'lend, L> Lending<'lend> for InvariantEnumerate<L> {
    type Lend = (usize, &'lend Cell<Option<&'lend String>>);
}

impl<L> Lender for InvariantEnumerate<L> {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleEnumerate<L, E>(L, std::marker::PhantomData<E>);

impl<'lend, L, E> FallibleLending<'lend> for InvariantFallibleEnumerate<L, E> {
    type Lend = (usize, &'lend Cell<Option<&'lend String>>);
}

impl<L, E> FallibleLender for InvariantFallibleEnumerate<L, E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
