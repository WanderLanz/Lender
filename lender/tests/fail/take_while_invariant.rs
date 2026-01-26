// Test that TakeWhile with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantTakeWhile<L, P>(L, P);

impl<'lend, L, P> Lending<'lend> for InvariantTakeWhile<L, P> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, P> Lender for InvariantTakeWhile<L, P> {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleTakeWhile<L, P, E>(L, P, std::marker::PhantomData<E>);

impl<'lend, L, P, E> FallibleLending<'lend> for InvariantFallibleTakeWhile<L, P, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, P, E> FallibleLender for InvariantFallibleTakeWhile<L, P, E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
