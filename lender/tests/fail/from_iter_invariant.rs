// Test that FromIter with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantFromIter<I> {
    iter: I,
}

impl<'lend, I> Lending<'lend> for InvariantFromIter<I>
where
    I: Iterator<Item = &'lend Cell<Option<&'lend String>>>,
{
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<I> Lender for InvariantFromIter<I>
where
    I: for<'lend> Iterator<Item = &'lend Cell<Option<&'lend String>>>,
{
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.iter.next()
    }
}

fn main() {}
