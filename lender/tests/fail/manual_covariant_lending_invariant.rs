// Test that manually implementing CovariantLending / CovariantFallibleLending
// for a type with an invariant Lend fails to compile, even when providing the
// required _check_covariance method.
//
// Before the _check_covariance method was added to the traits, this would have
// compiled silently — the traits were empty markers with no enforcement.

use std::cell::Cell;

struct Invariant;

impl<'lend> lender::Lending<'lend> for Invariant {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl lender::CovariantLending for Invariant {
    fn _check_covariance<'long: 'short, 'short>(
        lend: *const &'short <Self as lender::Lending<'long>>::Lend,
        _: lender::Uncallable,
    ) -> *const &'short <Self as lender::Lending<'short>>::Lend {
        lend
    }
}

struct FallibleInvariant;

impl<'lend> lender::FallibleLending<'lend> for FallibleInvariant {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

impl lender::CovariantFallibleLending for FallibleInvariant {
    fn _check_covariance<'long: 'short, 'short>(
        lend: *const &'short <Self as lender::FallibleLending<'long>>::Lend,
        _: lender::Uncallable,
    ) -> *const &'short <Self as lender::FallibleLending<'short>>::Lend {
        lend
    }
}

fn main() {}
