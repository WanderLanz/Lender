// Test that Map with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantMap<L, F>(L, F);

impl<'lend, L, F> Lending<'lend> for InvariantMap<L, F> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, F> Lender for InvariantMap<L, F> {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleMap<L, F, E>(L, F, std::marker::PhantomData<E>);

impl<'lend, L, F, E> FallibleLending<'lend> for InvariantFallibleMap<L, F, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, F, E> FallibleLender for InvariantFallibleMap<L, F, E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
