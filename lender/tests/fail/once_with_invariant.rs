// Test that OnceWith with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantOnceWith<F> {
    f: Option<F>,
}

impl<'lend, F> Lending<'lend> for InvariantOnceWith<F>
where
    F: FnOnce() -> &'lend Cell<Option<&'lend String>>,
{
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<F> Lender for InvariantOnceWith<F>
where
    F: for<'lend> FnOnce() -> &'lend Cell<Option<&'lend String>>,
{
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.f.take().map(|f| f())
    }
}

fn main() {}
