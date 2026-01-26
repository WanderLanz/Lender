// Test that Chain with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantChain<A, B>(A, B);

impl<'lend, A, B> Lending<'lend> for InvariantChain<A, B> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<A, B> Lender for InvariantChain<A, B> {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleChain<A, B, E>(A, B, std::marker::PhantomData<E>);

impl<'lend, A, B, E> FallibleLending<'lend> for InvariantFallibleChain<A, B, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<A, B, E> FallibleLender for InvariantFallibleChain<A, B, E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
