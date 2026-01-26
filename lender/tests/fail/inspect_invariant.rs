// Test that Inspect with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantInspect<L, F>(L, F);

impl<'lend, L, F> Lending<'lend> for InvariantInspect<L, F> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, F> Lender for InvariantInspect<L, F> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleInspect<L, F, E>(L, F, std::marker::PhantomData<E>);

impl<'lend, L, F, E> FallibleLending<'lend> for InvariantFallibleInspect<L, F, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, F, E> FallibleLender for InvariantFallibleInspect<L, F, E> {
    type Error = E;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
