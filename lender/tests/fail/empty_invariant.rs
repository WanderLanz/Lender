// Test that Empty with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantEmpty;

impl<'lend> Lending<'lend> for InvariantEmpty {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl Lender for InvariantEmpty {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
