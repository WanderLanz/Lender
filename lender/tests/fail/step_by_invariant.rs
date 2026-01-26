// Test that StepBy with an invariant lend type fails covariance check.

use std::cell::Cell;

use lender::{Lend, Lender, Lending};

struct InvariantStepBy<L> {
    lender: L,
    step: usize,
    first_take: bool,
}

impl<'lend, L> Lending<'lend> for InvariantStepBy<L> {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl<L> Lender for InvariantStepBy<L> {
    lender::covariance_check!();

    fn next(&mut self) -> Option<Lend<'_, Self>> {
        None
    }
}

fn main() {}
