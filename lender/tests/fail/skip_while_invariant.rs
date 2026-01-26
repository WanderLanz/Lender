// Test that SkipWhile with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantSkipWhile<L, P>(L, P);

impl<'lend, L, P> Lending<'lend> for InvariantSkipWhile<L, P> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, P> Lender for InvariantSkipWhile<L, P> {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleSkipWhile<L, P, E>(L, P, std::marker::PhantomData<E>);

impl<'lend, L, P, E> FallibleLending<'lend> for InvariantFallibleSkipWhile<L, P, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, P, E> FallibleLender for InvariantFallibleSkipWhile<L, P, E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
