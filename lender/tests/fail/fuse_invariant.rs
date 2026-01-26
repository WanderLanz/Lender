// Test that Fuse with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantFuse<L>(L);

impl<'lend, L> Lending<'lend> for InvariantFuse<L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L> Lender for InvariantFuse<L> {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleFuse<L, E>(L, std::marker::PhantomData<E>);

impl<'lend, L, E> FallibleLending<'lend> for InvariantFallibleFuse<L, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, E> FallibleLender for InvariantFallibleFuse<L, E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
