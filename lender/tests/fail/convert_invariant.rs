// Test that Convert (fallible-only adapter) with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending};

struct InvariantConvert<E, I>(I, std::marker::PhantomData<E>);

impl<'lend, E, I> FallibleLending<'lend> for InvariantConvert<E, I> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<E, I> FallibleLender for InvariantConvert<E, I> {
    type Error = E;
    lender::fallible_covariance_check!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
