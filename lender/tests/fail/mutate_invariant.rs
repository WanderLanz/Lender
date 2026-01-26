// Test that Mutate with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantMutate<L, F>(L, F);

impl<'lend, L, F> Lending<'lend> for InvariantMutate<L, F> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, F> Lender for InvariantMutate<L, F> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleMutate<L, F, E>(L, F, std::marker::PhantomData<E>);

impl<'lend, L, F, E> FallibleLending<'lend> for InvariantFallibleMutate<L, F, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, F, E> FallibleLender for InvariantFallibleMutate<L, F, E> {
    type Error = E;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
