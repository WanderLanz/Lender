// Test that Scan with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantScan<L, St, F>(L, St, F);

impl<'lend, L, St, F> Lending<'lend> for InvariantScan<L, St, F> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, St, F> Lender for InvariantScan<L, St, F> {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

struct InvariantFallibleScan<L, St, F, E>(L, St, F, std::marker::PhantomData<E>);

impl<'lend, L, St, F, E> FallibleLending<'lend> for InvariantFallibleScan<L, St, F, E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L, St, F, E> FallibleLender for InvariantFallibleScan<L, St, F, E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
