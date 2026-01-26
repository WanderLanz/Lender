// Test that Repeat with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{FallibleLend, FallibleLender, FallibleLending, Lend, Lender, Lending};

struct InvariantRepeat<'a> {
    value: &'a Cell<Option<&'a String>>,
}

impl<'lend> Lending<'lend> for InvariantRepeat<'_> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl Lender for InvariantRepeat<'_> {
    lender::check_covariance!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        Some(self.value)
    }
}

struct InvariantFallibleRepeat<E>(std::marker::PhantomData<E>);

impl<'lend, E> FallibleLending<'lend> for InvariantFallibleRepeat<E> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<E> FallibleLender for InvariantFallibleRepeat<E> {
    type Error = E;
    lender::check_covariance_fallible!();

    fn next(&mut self) -> Result<Option<FallibleLend<'_, Self>>, Self::Error> {
        Ok(None)
    }
}

fn main() {}
