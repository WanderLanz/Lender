// Test that FromFn with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantFromFn<F> {
    f: F,
}

impl<'lend, F> Lending<'lend> for InvariantFromFn<F>
where
    F: FnMut() -> Option<&'lend Cell<Option<&'lend String>>>,
{
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<F> Lender for InvariantFromFn<F>
where
    F: for<'lend> FnMut() -> Option<&'lend Cell<Option<&'lend String>>>,
{
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        (self.f)()
    }
}

fn main() {}
