// Test that repeat_with() and fallible_repeat_with() reject
// Lending/FallibleLending types that don't implement
// CovariantLend/CovariantFallibleLend.
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

fn test_repeat_with<'a>(f: impl FnMut() -> &'a Cell<Option<&'a String>>) {
    let _ = lender::repeat_with::<Invariant, _>(f);
}

fn test_fallible_repeat_with<'a>(
    f: impl FnMut() -> Result<&'a Cell<Option<&'a String>>, std::io::Error>,
) {
    let _ = lender::fallible_repeat_with::<FallibleInvariant, std::io::Error, _>(f);
}

fn main() {}
