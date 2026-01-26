// Test that Take with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantTake<L>(L);

impl<'lend, L> Lending<'lend> for InvariantTake<L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L> Lender for InvariantTake<L> {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleTake<L, E>(L, std::marker::PhantomData<E>);

impl<'lend, L, E> FallibleLending<'lend> for InvariantFallibleTake<L, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, E> FallibleLender for InvariantFallibleTake<L, E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
