// Test that MapWhile with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantMapWhile<L, P>(L, P);

impl<'lend, L, P> Lending<'lend> for InvariantMapWhile<L, P> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, P> Lender for InvariantMapWhile<L, P> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleMapWhile<L, P, E>(L, P, std::marker::PhantomData<E>);

impl<'lend, L, P, E> FallibleLending<'lend> for InvariantFallibleMapWhile<L, P, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, P, E> FallibleLender for InvariantFallibleMapWhile<L, P, E> {
    type Error = E;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
