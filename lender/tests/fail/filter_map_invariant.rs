// Test that FilterMap with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantFilterMap<L, F>(L, F);

impl<'lend, L, F> Lending<'lend> for InvariantFilterMap<L, F> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, F> Lender for InvariantFilterMap<L, F> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleFilterMap<L, F, E>(L, F, std::marker::PhantomData<E>);

impl<'lend, L, F, E> FallibleLending<'lend> for InvariantFallibleFilterMap<L, F, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, F, E> FallibleLender for InvariantFallibleFilterMap<L, F, E> {
    type Error = E;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
