// Test that manually implementing CovariantLending / CovariantFallibleLending
// for a type with an invariant Lend fails to compile, even when providing the
// required _check_covariance method.
//
// Before the _check_covariance method was added to the traits, this would have
// compiled silently; the traits were empty markers with no enforcement.

use std::cell::Cell;

struct Invariant;

impl<'lend> lender::Lending<'lend> for Invariant {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl lender::CovariantLending for Invariant {
    fn __check_covariance<'long: 'short, 'short>(
        proof: lender::CovariantProof<<Self as lender::Lending<'long>>::Lend>,
    ) -> lender::CovariantProof<<Self as lender::Lending<'short>>::Lend> {
        proof
    }
}

struct FallibleInvariant;

impl<'lend> lender::FallibleLending<'lend> for FallibleInvariant {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl lender::CovariantFallibleLending for FallibleInvariant {
    fn __check_covariance<'long: 'short, 'short>(
        proof: lender::CovariantProof<&'short <Self as lender::FallibleLending<'long>>::Lend>,
    ) -> lender::CovariantProof<&'short <Self as lender::FallibleLending<'short>>::Lend> {
        proof
    }
}

fn main() {}
