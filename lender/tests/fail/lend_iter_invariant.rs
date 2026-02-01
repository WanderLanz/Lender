// Test that lend_iter and lend_fallible_iter reject Lending/FallibleLending
// types that don't implement CovariantLend/CovariantFallibleLend.
// An invariant type could cause UB through the lifetime transmute in next().

use std::cell::Cell;

struct Invariant;

impl<'lend> lender::Lending<'lend> for Invariant {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

struct FallibleInvariant;

impl<'lend> lender::FallibleLending<'lend> for FallibleInvariant {
    type Lend = &'lend Cell<Option<&'lend String>>;
}

fn test_lend_iter<'a>(iter: impl Iterator<Item = &'a Cell<Option<&'a String>>>) {
    let _ = lender::lend_iter::<Invariant, _>(iter);
}

fn test_lend_fallible_iter<'a>(
    iter: impl fallible_iterator::FallibleIterator<
        Item = &'a Cell<Option<&'a String>>,
        Error = std::io::Error,
    >,
) {
    let _ = lender::lend_fallible_iter::<FallibleInvariant, _>(iter);
}

fn main() {}
