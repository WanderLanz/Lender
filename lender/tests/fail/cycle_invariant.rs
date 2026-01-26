// Test that Cycle with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantCycle<L>(L);

impl<'lend, L> Lending<'lend> for InvariantCycle<L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L> Lender for InvariantCycle<L> {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleCycle<L, E>(L, std::marker::PhantomData<E>);

impl<'lend, L, E> FallibleLending<'lend> for InvariantFallibleCycle<L, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, E> FallibleLender for InvariantFallibleCycle<L, E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
